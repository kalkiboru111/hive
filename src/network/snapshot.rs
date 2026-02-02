//! State channel snapshot serialization.
//!
//! Converts Hive's internal state (orders, vouchers, business info)
//! into the opaque bytes that Reality Network expects in a
//! StateChannelSnapshotBinary.

use super::types::StateChannelSnapshotBinary;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Hive-specific state that gets serialized into a state channel snapshot.
///
/// This is the rApp's view of its own state — Reality Network treats
/// it as opaque bytes, but other Hive nodes can deserialize and verify it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveStateSnapshot {
    /// Schema version for forward compatibility.
    pub version: u32,
    /// Business identifier (from config).
    pub business_name: String,
    /// Snapshot timestamp (unix millis).
    pub timestamp_ms: u64,
    /// Total orders placed (monotonic counter).
    pub total_orders: u64,
    /// Total revenue (in smallest currency unit).
    pub total_revenue_cents: i64,
    /// Active order count.
    pub active_orders: u32,
    /// Delivered order count.
    pub delivered_orders: u64,
    /// Voucher state summary.
    pub vouchers: VoucherStateSummary,
    /// Order hashes — compact proof that specific orders exist
    /// without exposing customer data on-chain.
    pub order_hashes: Vec<String>,
}

/// Summary of voucher state (no codes exposed on-chain).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoucherStateSummary {
    pub total_created: u64,
    pub total_redeemed: u64,
    pub total_value_created_cents: i64,
    pub total_value_redeemed_cents: i64,
}

impl HiveStateSnapshot {
    /// Serialize this snapshot into bytes for the state channel.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        // Use MessagePack for compact binary serialization.
        // JSON would work too but this is more efficient on-chain.
        let bytes = rmp_serde::to_vec(self)?;
        Ok(bytes)
    }

    /// Deserialize from state channel bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let snapshot: Self = rmp_serde::from_slice(bytes)?;
        Ok(snapshot)
    }

    /// Build a StateChannelSnapshotBinary ready for submission.
    pub fn to_state_channel_binary(
        &self,
        last_snapshot_hash: &str,
    ) -> Result<StateChannelSnapshotBinary> {
        let content = self.to_bytes()?;
        Ok(StateChannelSnapshotBinary::from_unsigned(
            last_snapshot_hash.to_string(),
            content,
        ))
    }
}

/// Build a HiveStateSnapshot from the current store state.
pub fn capture_state(
    store: &crate::store::Store,
    business_name: &str,
) -> Result<HiveStateSnapshot> {
    let stats = store.get_stats()?;

    // Hash each order for on-chain proof without exposing PII
    let orders = store.list_orders(None)?;
    let order_hashes: Vec<String> = orders
        .iter()
        .map(|o| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            format!("{}:{}:{}", o.id, o.total, o.customer_phone).hash(&mut hasher);
            format!("{:016x}", hasher.finish())
        })
        .collect();

    Ok(HiveStateSnapshot {
        version: 1,
        business_name: business_name.to_string(),
        timestamp_ms: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
        total_orders: stats.total_orders as u64,
        total_revenue_cents: (stats.total_revenue * 100.0) as i64,
        active_orders: stats.pending_orders as u32,
        delivered_orders: stats.delivered_orders as u64,
        vouchers: VoucherStateSummary {
            total_created: stats.total_vouchers as u64,
            total_redeemed: stats.redeemed_vouchers as u64,
            // TODO: track actual values in store
            total_value_created_cents: 0,
            total_value_redeemed_cents: 0,
        },
        order_hashes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_roundtrip() {
        let snapshot = HiveStateSnapshot {
            version: 1,
            business_name: "Test Business".to_string(),
            timestamp_ms: 1700000000000,
            total_orders: 42,
            total_revenue_cents: 123456,
            active_orders: 3,
            delivered_orders: 39,
            vouchers: VoucherStateSummary {
                total_created: 10,
                total_redeemed: 5,
                total_value_created_cents: 50000,
                total_value_redeemed_cents: 25000,
            },
            order_hashes: vec!["abc123".to_string(), "def456".to_string()],
        };

        let bytes = snapshot.to_bytes().unwrap();
        let restored = HiveStateSnapshot::from_bytes(&bytes).unwrap();

        assert_eq!(restored.version, 1);
        assert_eq!(restored.business_name, "Test Business");
        assert_eq!(restored.total_orders, 42);
        assert_eq!(restored.order_hashes.len(), 2);
    }

    #[test]
    fn test_state_channel_binary() {
        let snapshot = HiveStateSnapshot {
            version: 1,
            business_name: "Cloudy".to_string(),
            timestamp_ms: 1700000000000,
            total_orders: 1,
            total_revenue_cents: 3500,
            active_orders: 0,
            delivered_orders: 1,
            vouchers: VoucherStateSummary {
                total_created: 0,
                total_redeemed: 0,
                total_value_created_cents: 0,
                total_value_redeemed_cents: 0,
            },
            order_hashes: vec![],
        };

        let binary = snapshot.to_state_channel_binary("previous_hash_here").unwrap();
        assert_eq!(binary.last_snapshot_hash, "previous_hash_here");
        assert!(!binary.content_unsigned().is_empty());
    }
}
