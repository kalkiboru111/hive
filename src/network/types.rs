//! Reality Network data types.
//!
//! Mirrors the key types from org.reality.schema — just enough
//! for Hive to submit transactions and state channel snapshots.

use serde::{Deserialize, Serialize};

/// A Reality Network address (BASE58 encoded public key hash).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl Address {
    /// Create an address from a hex-encoded public key.
    /// Reality addresses are derived: SHA256(pubkey) → BASE58 with "NET" prefix.
    pub fn from_public_key(pubkey_hex: &str) -> Self {
        // TODO: implement proper address derivation
        // For now, placeholder
        Address(format!("NET{}", &pubkey_hex[..40]))
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

/// A cryptographic signature (hex-encoded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub id: String,
    pub signature: String,
}

/// A signed wrapper — matches Reality's Signed[A] envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signed<T: Serialize> {
    pub value: T,
    pub proofs: Vec<Signature>,
}

/// State channel snapshot binary — the core type for submitting
/// rApp state to the L0 network.
///
/// Maps to: org.reality.statechannel.StateChannelSnapshotBinary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StateChannelSnapshotBinary {
    /// Hash of the previous snapshot (chain integrity).
    pub last_snapshot_hash: String,
    /// Opaque content bytes — Hive serializes its own state here.
    #[serde(with = "base64_bytes")]
    pub content: Vec<u8>,
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

/// Base64 encoding for binary content in JSON.
mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(&s).map_err(serde::de::Error::custom)
    }
}
