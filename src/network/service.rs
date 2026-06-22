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
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// All-zero hash used as the genesis "previous snapshot" reference.
const ZERO_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Maximum attempts (initial + retries) for a single snapshot submission.
const MAX_SUBMIT_ATTEMPTS: u32 = 3;

/// Background service that submits state snapshots to Reality Network.
pub struct NetworkService {
    client: RealityClient,
    identity: NodeIdentity,
    store: Store,
    business_name: String,
    interval_secs: u64,
    /// Tracks the hash of the last submitted snapshot (for chain integrity).
    last_snapshot_hash: String,
    /// File where `last_snapshot_hash` is persisted across restarts.
    hash_path: PathBuf,
    /// Content hash (timestamp-independent) of the last *successfully* submitted
    /// snapshot, used to skip redundant submissions when nothing changed.
    last_content_hash: Option<String>,
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
        project_dir: &Path,
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

        // Restore the snapshot-chain head so the chain continues across
        // restarts instead of resetting to the genesis (zero) hash.
        let hash_path = identity_path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join("last_snapshot.txt");
        let last_snapshot_hash = load_last_hash(&hash_path);

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
            last_snapshot_hash,
            hash_path,
            last_content_hash: None,
            dirty,
            notify,
        };

        Ok((service, notifier))
    }

    /// Run the service loop (call from a spawned task).
    pub async fn run(mut self) {
        info!("🌐 Reality Network service started (interval: {}s)", self.interval_secs);

        loop {
            // Wait for either a dirty notification (fast-path) or the interval
            // timeout (safety net that catches changes from any source).
            tokio::select! {
                _ = self.notify.notified() => {
                    // State changed — submit soon
                }
                _ = tokio::time::sleep(Duration::from_secs(self.interval_secs)) => {
                    // Periodic check
                }
            }

            // Consume the wake signal. Correctness no longer depends on this
            // flag: submit_snapshot() compares the captured content against the
            // last submitted snapshot, so any change (bot, dashboard, direct DB)
            // is caught on the next tick, and a failed submit is retried because
            // last_content_hash is only advanced on success.
            self.dirty.store(false, Ordering::Release);

            if let Err(e) = self.submit_snapshot().await {
                error!("❌ Failed to submit snapshot (will retry next tick): {}", e);
            }
        }
    }

    /// Capture current state and submit a snapshot if it changed.
    ///
    /// Returns `Ok(true)` if a snapshot was submitted, `Ok(false)` if the state
    /// was unchanged since the last successful submission.
    async fn submit_snapshot(&mut self) -> Result<bool> {
        // Capture state from the store
        let hive_state = snapshot::capture_state(&self.store, &self.business_name)?;

        // Skip if nothing meaningful changed since the last successful submit.
        let content_hash = hive_state.content_hash();
        if self.last_content_hash.as_deref() == Some(content_hash.as_str()) {
            return Ok(false);
        }

        info!(
            "📸 Capturing state: {} orders, {} delivered",
            hive_state.total_orders, hive_state.delivered_orders
        );

        // Build the state channel binary
        let sc_binary = hive_state.to_state_channel_binary(&self.last_snapshot_hash)?;

        // Sign it
        let signed = self.identity.sign_value(&sc_binary)?;

        // Submit to L0 with bounded exponential-backoff retry for transient
        // failures (network blips, node briefly unavailable).
        self.submit_with_retry(&signed).await?;

        // Advance the chain head only after a confirmed submission. Persist it
        // atomically *before* trusting it in-memory so a torn write can never
        // corrupt the head. A hard persist failure (disk full, permissions) is
        // logged loudly: the running process still advances so it keeps emitting
        // a correct chain, but the head may not survive a restart.
        let hash = NodeIdentity::hash_value(&sc_binary)?;
        if let Err(e) = save_last_hash(&self.hash_path, &hash) {
            error!(
                "❌ Failed to persist snapshot chain head to {} ({}): the chain may not survive a restart",
                self.hash_path.display(),
                e
            );
        }
        self.last_snapshot_hash = hash;
        self.last_content_hash = Some(content_hash);

        info!("✅ Snapshot submitted to Reality Network");
        Ok(true)
    }

    /// Submit a signed snapshot, retrying transient failures with backoff.
    ///
    /// Assumption: the L0 node is idempotent for a re-submitted *identical*
    /// signed snapshot (dedup by hash). If a request times out after L0 has
    /// already accepted it, the retry re-sends the same bytes; an idempotent
    /// node treats that as success. If the node instead rejects duplicates, an
    /// ambiguous timeout could advance the remote chain while we return `Err`
    /// and rebuild against a stale head next tick. The L0 dedup contract isn't
    /// visible from here; revisit with reconcile-on-ambiguous-error if it does
    /// not hold.
    async fn submit_with_retry(
        &self,
        signed: &super::types::Signed<super::types::StateChannelSnapshotBinary>,
    ) -> Result<()> {
        let mut attempt = 1;
        loop {
            match self
                .client
                .submit_state_channel_snapshot(&self.identity.address, signed)
                .await
            {
                Ok(()) => return Ok(()),
                Err(e) if attempt < MAX_SUBMIT_ATTEMPTS => {
                    // 1s, 2s, 4s, ...
                    let delay = Duration::from_secs(1 << (attempt - 1));
                    warn!(
                        "⚠️  Snapshot submit attempt {}/{} failed: {} — retrying in {:?}",
                        attempt, MAX_SUBMIT_ATTEMPTS, e, delay
                    );
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

/// True if `s` is a well-formed snapshot hash (64 lowercase/uppercase hex chars).
fn is_valid_hash(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Load the persisted snapshot-chain head.
///
/// Falls back to the genesis zero hash if the file is absent, empty, or does not
/// contain a well-formed 64-char hex hash — so a truncated/corrupt file can never
/// be used as the next chain head.
fn load_last_hash(path: &std::path::Path) -> String {
    match std::fs::read_to_string(path) {
        Ok(s) => {
            let trimmed = s.trim();
            if is_valid_hash(trimmed) {
                trimmed.to_string()
            } else {
                if !trimmed.is_empty() {
                    warn!(
                        "⚠️  Ignoring malformed snapshot hash in {} — resetting to genesis",
                        path.display()
                    );
                }
                ZERO_HASH.to_string()
            }
        }
        Err(_) => ZERO_HASH.to_string(),
    }
}

/// Persist the snapshot-chain head atomically (temp file + rename) so a crash
/// mid-write can never leave a partially written hash on disk.
fn save_last_hash(path: &std::path::Path, hash: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, hash)?;
    std::fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_missing_hash_file_returns_zero() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_snapshot.txt");
        assert_eq!(load_last_hash(&path), ZERO_HASH);
    }

    #[test]
    fn test_save_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_snapshot.txt");
        // A well-formed 64-char hex hash round-trips.
        let hash = "a".repeat(64);
        save_last_hash(&path, &hash).unwrap();
        assert_eq!(load_last_hash(&path), hash);
    }

    #[test]
    fn test_blank_file_returns_zero() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_snapshot.txt");
        save_last_hash(&path, "  \n ").unwrap();
        assert_eq!(load_last_hash(&path), ZERO_HASH);
    }

    #[test]
    fn test_malformed_hash_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("last_snapshot.txt");
        // Too short and non-hex values are both rejected back to genesis.
        save_last_hash(&path, "abc123def456").unwrap();
        assert_eq!(load_last_hash(&path), ZERO_HASH);
        save_last_hash(&path, &"z".repeat(64)).unwrap();
        assert_eq!(load_last_hash(&path), ZERO_HASH);
    }

    #[test]
    fn test_is_valid_hash() {
        assert!(is_valid_hash(&"0".repeat(64)));
        assert!(is_valid_hash(ZERO_HASH));
        assert!(!is_valid_hash(&"0".repeat(63)));
        assert!(!is_valid_hash(&"g".repeat(64)));
        assert!(!is_valid_hash(""));
    }
}
