//! HTTP client for Reality Network L0/L1 nodes.
//!
//! Talks to the Reality node's REST API to submit state channel
//! snapshots and query global state.

use super::types::*;
use anyhow::{Context, Result};
use log::{debug, error, info};

/// Client for communicating with a Reality Network L0 node.
#[derive(Debug, Clone)]
pub struct RealityClient {
    /// Base URL of the L0 node (e.g., "http://localhost:9000")
    base_url: String,
    /// HTTP client
    client: reqwest::Client,
}

impl RealityClient {
    /// Create a new client pointing at an L0 node.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    // ── Cluster Info ──────────────────────────────────────────

    /// Check if the node is reachable and get cluster info.
    pub async fn cluster_info(&self) -> Result<Vec<ClusterNodeInfo>> {
        let url = format!("{}/cluster/info", self.base_url);
        debug!("GET {}", url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to reach Reality L0 node")?;

        let nodes: Vec<ClusterNodeInfo> = resp
            .json()
            .await
            .context("Failed to parse cluster info")?;

        info!("Reality cluster: {} nodes", nodes.len());
        Ok(nodes)
    }

    /// Health check — returns true if the node responds.
    pub async fn is_healthy(&self) -> bool {
        self.cluster_info().await.is_ok()
    }

    // ── Global Snapshots ──────────────────────────────────────

    /// Get the latest snapshot ordinal.
    pub async fn latest_ordinal(&self) -> Result<u64> {
        let url = format!("{}/global-snapshots/latest/ordinal", self.base_url);
        debug!("GET {}", url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch latest ordinal")?;

        let ordinal: GlobalSnapshotOrdinal = resp
            .json()
            .await
            .context("Failed to parse ordinal")?;

        Ok(ordinal.value)
    }

    /// Query a deployed app's info.
    pub async fn get_app_data(&self, app_identifier: &str) -> Result<Option<DeployAppInfo>> {
        let url = format!(
            "{}/global-snapshots/app-data/{}",
            self.base_url, app_identifier
        );
        debug!("GET {}", url);

        let resp = self.client.get(&url).send().await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let info: DeployAppInfo = resp
            .json()
            .await
            .context("Failed to parse app data")?;

        Ok(Some(info))
    }

    // ── State Channel Submission ──────────────────────────────

    /// Submit a signed state channel snapshot to L0.
    ///
    /// This is the core rApp integration point: Hive serializes its
    /// order/voucher state, signs it, and submits to the network.
    pub async fn submit_state_channel_snapshot(
        &self,
        address: &Address,
        snapshot: &Signed<StateChannelSnapshotBinary>,
    ) -> Result<()> {
        let url = format!(
            "{}/state-channels/{}/snapshot",
            self.base_url, address.0
        );
        info!("Submitting state channel snapshot to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(snapshot)
            .send()
            .await
            .context("Failed to submit state channel snapshot")?;

        if resp.status().is_success() {
            info!("✅ State channel snapshot accepted by L0");
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("❌ L0 rejected snapshot: {} — {}", status, body);
            anyhow::bail!("L0 rejected snapshot: {} — {}", status, body)
        }
    }

    // ── Transactions ──────────────────────────────────────────

    /// Submit a transaction to L0 (deploy app, record data, etc).
    pub async fn submit_transaction<T: serde::Serialize>(
        &self,
        transaction: &Signed<T>,
    ) -> Result<String> {
        let url = format!("{}/transactions", self.base_url);
        info!("Submitting transaction to {}", url);

        let resp = self
            .client
            .post(&url)
            .json(transaction)
            .send()
            .await
            .context("Failed to submit transaction")?;

        if resp.status().is_success() {
            let hash = resp.text().await.unwrap_or_default();
            info!("✅ Transaction accepted: {}", hash);
            Ok(hash)
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("❌ Transaction rejected: {} — {}", status, body);
            anyhow::bail!("Transaction rejected: {} — {}", status, body)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RealityClient::new("http://localhost:9000");
        assert_eq!(client.base_url, "http://localhost:9000");
    }

    #[test]
    fn test_trailing_slash_stripped() {
        let client = RealityClient::new("http://localhost:9000/");
        assert_eq!(client.base_url, "http://localhost:9000");
    }
}
