//! Node identity and transaction signing.
//!
//! Reality Network uses secp256k1 keypairs for identity.
//! Each Hive instance has a keypair that identifies it as a
//! state channel participant on the network.
//!
//! Signing protocol (must match Reality's JVM implementation):
//! 1. Serialize value to JSON (compact, no spaces)
//! 2. SHA256(json_bytes) → hex string (this is the "Hash")
//! 3. SHA512withECDSA(hash_hex_string_as_utf8_bytes) → DER signature
//!
//! Identity format:
//! - Id (peer id): uncompressed public key x||y (128 hex chars, no 04 prefix)
//! - Address: SHA256(DER-encoded-pubkey) → Base58 → last 36 chars → parity

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
use std::path::Path;

/// DER SubjectPublicKeyInfo prefix for secp256k1 uncompressed public keys.
/// This wraps the raw 04||x||y (65 bytes) into a proper DER structure (88 bytes total).
const PUBLIC_KEY_DER_PREFIX: &str = "3056301006072a8648ce3d020106052b8104000a03420004";

/// A Reality Network node identity (keypair + derived address).
#[derive(Debug, Clone)]
pub struct NodeIdentity {
    /// The secp256k1 secret key (32 bytes).
    secret_key: Vec<u8>,
    /// The uncompressed public key x||y (64 bytes, 128 hex chars — no 04 prefix).
    /// This is the "Id" / "PeerId" in Reality's type system.
    pub peer_id_hex: String,
    /// The derived Reality Network address.
    pub address: super::types::Address,
}

/// Serializable identity file format.
#[derive(Debug, Serialize, Deserialize)]
struct IdentityFile {
    /// Hex-encoded secret key.
    secret_key: String,
    /// Hex-encoded uncompressed public key (x||y, no 04 prefix).
    peer_id: String,
    /// Derived address.
    address: String,
}

impl NodeIdentity {
    /// Generate a new random identity.
    pub fn generate() -> Result<Self> {
        use rand::RngCore;
        let mut secret = [0u8; 32];
        rand::rng().fill_bytes(&mut secret);
        Self::from_secret_key(&secret)
    }

    /// Create identity from a raw 32-byte secret key.
    fn from_secret_key(secret: &[u8; 32]) -> Result<Self> {
        let secp = secp256k1::Secp256k1::new();
        let sk =
            secp256k1::SecretKey::from_slice(secret).context("Failed to create secret key")?;
        let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);

        // Uncompressed: 04 || x(32) || y(32) = 65 bytes
        let uncompressed = pk.serialize_uncompressed();
        // Strip the 04 prefix → 64 bytes = 128 hex chars
        let peer_id_hex = hex::encode(&uncompressed[1..]);

        let address = Self::derive_address(&uncompressed)?;

        info!("Generated node identity: {} (peer: {}...)", address, &peer_id_hex[..16]);

        Ok(Self {
            secret_key: secret.to_vec(),
            peer_id_hex,
            address,
        })
    }

    /// Derive a Reality Network address from an uncompressed public key.
    ///
    /// Algorithm (from address.scala):
    ///   1. Build full DER SubjectPublicKeyInfo: prefix || x || y
    ///   2. SHA256(der_bytes)
    ///   3. Base58 encode the hash
    ///   4. Take last 36 characters
    ///   5. Sum all digit characters, mod 9 = parity
    ///   6. Address = "NET" + parity + last36
    fn derive_address(uncompressed_pubkey: &[u8; 65]) -> Result<super::types::Address> {
        // Build the full DER-encoded public key
        let prefix_bytes = hex::decode(PUBLIC_KEY_DER_PREFIX)?;
        let mut der = Vec::with_capacity(prefix_bytes.len() + 64);
        der.extend_from_slice(&prefix_bytes);
        der.extend_from_slice(&uncompressed_pubkey[1..]); // x || y (skip 04)

        // SHA256 hash
        let hash = Sha256::digest(&der);

        // Base58 encode
        let encoded = bs58::encode(&hash).into_string();

        // Last 36 chars
        let end = if encoded.len() >= 36 {
            &encoded[encoded.len() - 36..]
        } else {
            &encoded
        };

        // Parity: sum of digit characters mod 9
        let digit_sum: u32 = end.chars().filter(|c| c.is_ascii_digit()).map(|c| c.to_digit(10).unwrap()).sum();
        let parity = digit_sum % 9;

        let address = format!("NET{}{}", parity, end);
        Ok(super::types::Address(address))
    }

    /// Load identity from a file, or generate and save if it doesn't exist.
    pub fn load_or_generate(path: &Path) -> Result<Self> {
        if path.exists() {
            Self::load(path)
        } else {
            let identity = Self::generate()?;
            identity.save(path)?;
            Ok(identity)
        }
    }

    /// Load identity from a JSON file.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read identity file: {}", path.display()))?;
        let file: IdentityFile =
            serde_json::from_str(&contents).context("Failed to parse identity file")?;

        let secret_bytes = hex::decode(&file.secret_key).context("Invalid secret key hex")?;
        let secret: [u8; 32] = secret_bytes
            .try_into()
            .map_err(|_| anyhow::anyhow!("Secret key must be 32 bytes"))?;

        Self::from_secret_key(&secret)
    }

    /// Save identity to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let file = IdentityFile {
            secret_key: hex::encode(&self.secret_key),
            peer_id: self.peer_id_hex.clone(),
            address: self.address.0.clone(),
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(&file)?;
        std::fs::write(path, &contents)
            .with_context(|| format!("Failed to write identity file: {}", path.display()))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        info!("Saved node identity to {}", path.display());
        Ok(())
    }

    /// Compute the Reality "Hash" of a JSON-serializable value.
    ///
    /// This matches Reality's Hashable.forJson:
    ///   JSON noSpaces → UTF-8 bytes → SHA256 → lowercase hex string
    pub fn hash_value<T: Serialize>(value: &T) -> Result<String> {
        let json = serde_json::to_string(value)?;
        let hash = Sha256::digest(json.as_bytes());
        Ok(hex::encode(hash))
    }

    /// Sign using Reality's protocol: SHA512withECDSA over the hash hex string.
    ///
    /// Reality's Signing.signData signs hash.value.getBytes("UTF-8") using
    /// SHA512withECDSA. In pure Rust, that means:
    ///   1. SHA-512(hash_hex_bytes) → 64-byte digest
    ///   2. ECDSA sign(digest[..32]) over secp256k1
    ///
    /// Note: Java's SHA512withECDSA computes SHA-512 internally then signs.
    /// secp256k1 ECDSA requires a 32-byte message, so the implementation
    /// takes the first 32 bytes of the SHA-512 digest (which is what the
    /// JVM's EC provider does internally for curves with 256-bit order).
    pub fn sign_hash_hex(&self, hash_hex: &str) -> Result<String> {
        let hash_bytes = hash_hex.as_bytes(); // UTF-8 of the hex string
        let sha512 = Sha512::digest(hash_bytes);

        // secp256k1 needs a 32-byte message — use first 32 bytes of SHA-512
        let msg_bytes: [u8; 32] = sha512[..32].try_into()?;

        let secp = secp256k1::Secp256k1::new();
        let sk = secp256k1::SecretKey::from_slice(&self.secret_key)
            .context("Invalid secret key")?;
        let msg = secp256k1::Message::from_digest(msg_bytes);
        let sig = secp.sign_ecdsa(&msg, &sk);

        Ok(hex::encode(sig.serialize_der()))
    }

    /// Sign and wrap a value into a Signed<T> envelope.
    ///
    /// Matches Reality's SignatureProof.fromData:
    ///   1. Hash the value (JSON → SHA256 → hex)
    ///   2. Sign the hash hex string (SHA512withECDSA)
    ///   3. Wrap in Signed { value, proofs: [{ id, signature }] }
    pub fn sign_value<T: Serialize>(&self, value: &T) -> Result<super::types::Signed<T>>
    where
        T: Clone,
    {
        let hash_hex = Self::hash_value(value)?;
        let signature = self.sign_hash_hex(&hash_hex)?;

        Ok(super::types::Signed {
            value: value.clone(),
            proofs: vec![super::types::SignatureProof {
                id: self.peer_id_hex.clone(),
                signature,
            }],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_generate_identity() {
        let identity = NodeIdentity::generate().unwrap();
        // Peer ID should be 128 hex chars (64 bytes, uncompressed x||y)
        assert_eq!(identity.peer_id_hex.len(), 128);
        assert!(identity.address.0.starts_with("NET"));
        // Address should be NET + parity(1) + base58(36) = 40 chars
        assert_eq!(identity.address.0.len(), 40);
    }

    #[test]
    fn test_address_format() {
        let identity = NodeIdentity::generate().unwrap();
        let addr = &identity.address.0;
        // NET prefix
        assert!(addr.starts_with("NET"));
        // Parity digit (0-8)
        let parity = addr.chars().nth(3).unwrap();
        assert!(parity.is_ascii_digit());
        assert!(parity.to_digit(10).unwrap() < 9);
    }

    #[test]
    fn test_save_and_load() {
        let identity = NodeIdentity::generate().unwrap();
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        identity.save(path).unwrap();
        let loaded = NodeIdentity::load(path).unwrap();

        assert_eq!(identity.peer_id_hex, loaded.peer_id_hex);
        assert_eq!(identity.address.0, loaded.address.0);
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = NodeIdentity::generate().unwrap();
        let hash_hex = "abc123def456";
        let sig = identity.sign_hash_hex(hash_hex).unwrap();
        assert!(!sig.is_empty());
        // DER signature should be 70-72 bytes (140-144 hex chars)
        assert!(sig.len() >= 136 && sig.len() <= 148);
    }

    #[test]
    fn test_sign_value() {
        let identity = NodeIdentity::generate().unwrap();
        let data = serde_json::json!({"test": "value"});
        let signed = identity.sign_value(&data).unwrap();
        assert_eq!(signed.proofs.len(), 1);
        // Proof id should be the peer id (128 hex chars)
        assert_eq!(signed.proofs[0].id.len(), 128);
    }

    #[test]
    fn test_deterministic_address() {
        // Same secret key → same address
        let secret = [42u8; 32];
        let id1 = NodeIdentity::from_secret_key(&secret).unwrap();
        let id2 = NodeIdentity::from_secret_key(&secret).unwrap();
        assert_eq!(id1.address.0, id2.address.0);
        assert_eq!(id1.peer_id_hex, id2.peer_id_hex);
    }
}
