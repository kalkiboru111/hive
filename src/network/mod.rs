//! Reality Network integration module.
//!
//! Provides the bridge between Hive's local state (SQLite) and
//! Reality Network's L0/L1 consensus layer. Hive submits state
//! channel snapshots containing serialized order/voucher state,
//! making each business instance a registered rApp on the network.
//!
//! Architecture:
//! - Hive (Rust) handles WhatsApp + local state
//! - Reality L0 node handles consensus + global snapshots
//! - State channel = Hive's serialized state as opaque bytes
//! - Each business = a state channel address on the network

pub mod client;
pub mod identity;
pub mod snapshot;
pub mod types;
