//! Reality Network data types.
//!
//! Mirrors the key types from org.reality.schema — just enough
//! for Hive to submit transactions and state channel snapshots.

use serde::{Deserialize, Serialize};

/// A Reality Network address (BASE58 encoded public key hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    /// Create an address directly from a string.
    pub fn new(addr: &str) -> Self {
        Address(addr.to_string())
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SHA256 hash used throughout Reality Network.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(pub String);

impl Hash {
    pub fn empty() -> Self {
        Hash(String::new())
    }
}

/// Snapshot ordinal — monotonically increasing counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SnapshotOrdinal(pub u64);

/// Transaction reference pointing to a previous transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReference {
    pub ordinal: u64,
    pub hash: String,
}

impl TransactionReference {
    pub fn empty() -> Self {
        Self {
            ordinal: 0,
            hash: String::new(),
        }
    }
}

/// A signature proof — matches Reality's SignatureProof.
/// `id` = uncompressed public key x||y (128 hex chars, no 04 prefix).
/// `signature` = DER-encoded ECDSA signature (hex).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureProof {
    pub id: String,
    pub signature: String,
}

/// A signed wrapper — matches Reality's Signed[A] envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signed<T: Serialize> {
    pub value: T,
    pub proofs: Vec<SignatureProof>,
}

/// State channel snapshot binary — the core type for submitting
/// rApp state to the L0 network.
///
/// Maps to: org.reality.statechannel.StateChannelSnapshotBinary
/// Content is serialized as a JSON array of SIGNED byte values [-128..127]
/// to match Circe's Array[Byte] encoding (Java bytes are signed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChannelSnapshotBinary {
    /// Hash of the previous snapshot (chain integrity).
    pub last_snapshot_hash: String,
    /// Opaque content bytes — Hive serializes its own state here.
    /// Uses i8 to match Java's signed Byte type in Circe serialization.
    pub content: Vec<i8>,
}

impl StateChannelSnapshotBinary {
    /// Create from unsigned bytes (converts u8 → i8 for Java compatibility).
    pub fn from_unsigned(last_snapshot_hash: String, content: Vec<u8>) -> Self {
        let signed_content: Vec<i8> = content.into_iter().map(|b| b as i8).collect();
        Self {
            last_snapshot_hash,
            content: signed_content,
        }
    }

    /// Get content as unsigned bytes.
    pub fn content_unsigned(&self) -> Vec<u8> {
        self.content.iter().map(|&b| b as u8).collect()
    }
}

/// State channel output — wraps a signed snapshot with its address.
///
/// Maps to: org.reality.statechannel.StateChannelOutput
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChannelOutput {
    pub address: Address,
    pub snapshot: Signed<StateChannelSnapshotBinary>,
}

/// Deploy app transaction — registers an rApp on the network.
///
/// Maps to: org.reality.schema.transaction.DeployAppTransaction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployAppTransaction {
    pub source: String,
    pub destination: String,
    pub binary_hash: String,
    pub app_name: String,
    pub app_version: String,
    pub app_description: String,
    #[serde(rename = "appDownloadURL")]
    pub app_download_url: String,
    pub fee: u64,
    pub amount: u64,
    pub parent: TransactionReference,
    pub salt: i64,
    pub token_ticker: String,
    pub total_supply: u64,
}

/// Info returned when querying a deployed app.
///
/// Maps to: org.reality.schema.transaction.DeployAppTransactionInfo
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployAppInfo {
    pub source: String,
    pub app_name: String,
    pub app_version: String,
    pub app_description: String,
    #[serde(rename = "appDownloadURL")]
    pub app_download_url: String,
    pub binary_hash: String,
    pub token_ticker: String,
    pub total_supply: u64,
}

/// Cluster node info returned by /cluster/info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeInfo {
    pub id: String,
    pub ip: String,
    pub state: String,
    #[serde(default)]
    pub reputation: Option<f64>,
}

/// Global snapshot info (simplified).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSnapshotOrdinal {
    pub value: u64,
}

// Note: Vec<u8> with serde's default serialization produces a JSON array
// of integers [1,2,3,...] which matches Reality's Circe Array[Byte] encoding.
