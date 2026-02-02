//! Reality Network snapshot service.
//!
//! Runs as a background task, submitting state channel snapshots
//! to the L0 node whenever state changes occur. Rate-limited to
//! avoid spamming the network.

use super::client::RealityClient;
use super::identity::NodeIdentity;
use super::snapshot;
use crate::config::NetworkConfig;
use crate::store::Store;
use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

/// Background service that submits state snapshots to Reality Network.
pub struct NetworkService {
    client: RealityClient,
    identity: NodeIdentity,
    store: Store,
    business_name: String,
    interval_secs: u64,
    /// Tracks the hash of the last submitted snapshot (for chain integrity).
    last_snapshot_hash: String,
    /// Signal that state has changed and a snapshot should be submitted.
    dirty: Arc<AtomicBool>,
    /// Notification channel to wake the service immediately.
    notify: Arc<Notify>,
}

/// Handle to notify the network service of state changes.
#[derive(Clone)]
pub struct NetworkNotifier {
    dirty: Arc<AtomicBool>,
    notify: Arc<Notify>,
    enabled: bool,
}

impl NetworkNotifier {
    /// Create a no-op notifier (when network is disabled).
    pub fn disabled() -> Self {
        Self {
            dirty: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
            enabled: false,
        }
    }

    /// Signal that state has changed and a snapshot should be submitted.
    pub fn mark_dirty(&self) {
        if self.enabled {
            self.dirty.store(true, Ordering::Release);
            self.notify.notify_one();
        }
    }
}

impl NetworkService {
    /// Initialize the network service (loads/generates identity, checks cluster health).
    pub async fn new(
        config: &NetworkConfig,
        store: Store,
        business_name: String,
        project_dir: &PathBuf,
    ) -> Result<(Self, NetworkNotifier)> {
        let client = RealityClient::new(&config.l0_url);

        // Load or generate node identity
        let identity_path = project_dir.join(&config.identity_path);
        let identity = NodeIdentity::load_or_generate(&identity_path)?;
        info!(
            "🔗 Reality Network identity: {} (peer: {}...)",
            identity.address,
            &identity.peer_id_hex[..16]
        );

        // Check cluster health
        match client.cluster_info().await {
            Ok(nodes) => {
                info!("✅ Reality cluster reachable: {} node(s)", nodes.len());
            }
            Err(e) => {
                warn!("⚠️  Reality cluster not reachable: {} — will retry", e);
            }
        }

        let dirty = Arc::new(AtomicBool::new(false));
        let notify = Arc::new(Notify::new());

        let notifier = NetworkNotifier {
            dirty: dirty.clone(),
            notify: notify.clone(),
            enabled: true,
        };

        let service = Self {
            client,
            identity,
            store,
            business_name,
            interval_secs: config.snapshot_interval_secs,
            last_snapshot_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            dirty,
            notify,
        };

        Ok((service, notifier))
    }

    /// Run the service loop (call from a spawned task).
    pub async fn run(mut self) {
        info!("🌐 Reality Network service started (interval: {}s)", self.interval_secs);

        loop {
            // Wait for either a dirty notification or the interval timeout
            tokio::select! {
                _ = self.notify.notified() => {
                    // State changed — submit soon
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(self.interval_secs)) => {
                    // Periodic check
                }
            }

            // Only submit if state actually changed
            if !self.dirty.swap(false, Ordering::AcqRel) {
                continue;
            }

            if let Err(e) = self.submit_snapshot().await {
                error!("❌ Failed to submit snapshot: {}", e);
            }
        }
    }

    /// Capture current state and submit a snapshot.
    async fn submit_snapshot(&mut self) -> Result<()> {
        // Capture state from the store
        let hive_state = snapshot::capture_state(&self.store, &self.business_name)?;

        info!(
            "📸 Capturing state: {} orders, {} delivered",
            hive_state.total_orders, hive_state.delivered_orders
        );

        // Build the state channel binary
        let sc_binary = hive_state.to_state_channel_binary(&self.last_snapshot_hash)?;

        // Sign it
        let signed = self.identity.sign_value(&sc_binary)?;

        // Submit to L0
        self.client
            .submit_state_channel_snapshot(&self.identity.address, &signed)
            .await?;

        // Update the last snapshot hash for chain integrity
        // Hash the binary we just submitted
        let hash = NodeIdentity::hash_value(&sc_binary)?;
        self.last_snapshot_hash = hash;

        info!("✅ Snapshot submitted to Reality Network");
        Ok(())
    }
}
