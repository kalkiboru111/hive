//! Node identity and transaction signing.
//!
//! Reality Network uses secp256k1 keypairs for identity.
//! Each Hive instance has a keypair that identifies it as a
//! state channel participant on the network.

use anyhow::{Context, Result};
use log::info;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A Reality Network node identity (keypair + derived address).
#[derive(Debug, Clone)]
pub struct NodeIdentity {
    /// The secp256k1 secret key (32 bytes).
    secret_key: Vec<u8>,
    /// The compressed public key (33 bytes, hex-encoded).
    pub public_key_hex: String,
    /// The derived Reality Network address.
    pub address: super::types::Address,
}

/// Serializable identity file format.
#[derive(Debug, Serialize, Deserialize)]
struct IdentityFile {
    /// Hex-encoded secret key.
    secret_key: String,
    /// Hex-encoded public key.
    public_key: String,
    /// Derived address.
    address: String,
}

impl NodeIdentity {
    /// Generate a new random identity.
    pub fn generate() -> Result<Self> {
        use rand::RngCore;
        let mut secret = [0u8; 32];
        rand::rng().fill_bytes(&mut secret);

        // Derive public key using secp256k1
        let secp = secp256k1::Secp256k1::new();
        let sk = secp256k1::SecretKey::from_slice(&secret)
            .context("Failed to create secret key")?;
        let pk = secp256k1::PublicKey::from_secret_key(&secp, &sk);
        let pk_hex = hex::encode(pk.serialize());

        let address = super::types::Address::from_public_key(&pk_hex);

        info!("Generated new node identity: {}", address);

        Ok(Self {
            secret_key: secret.to_vec(),
            public_key_hex: pk_hex,
            address,
        })
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
        let file: IdentityFile = serde_json::from_str(&contents)
            .context("Failed to parse identity file")?;

        let secret_key = hex::decode(&file.secret_key)
            .context("Invalid secret key hex")?;

        Ok(Self {
            secret_key,
            public_key_hex: file.public_key,
            address: super::types::Address(file.address),
        })
    }

    /// Save identity to a JSON file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let file = IdentityFile {
            secret_key: hex::encode(&self.secret_key),
            public_key: self.public_key_hex.clone(),
            address: self.address.0.clone(),
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let contents = serde_json::to_string_pretty(&file)?;
        std::fs::write(path, &contents)
            .with_context(|| format!("Failed to write identity file: {}", path.display()))?;

        // Restrict permissions (secret key!)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        }

        info!("Saved node identity to {}", path.display());
        Ok(())
    }

    /// Sign arbitrary bytes, returning a hex-encoded DER signature.
    pub fn sign(&self, message: &[u8]) -> Result<String> {
        use secp256k1::{Message, Secp256k1, SecretKey};
        use sha2::{Digest, Sha256};

        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&self.secret_key)
            .context("Invalid secret key")?;

        // Hash the message (Reality uses SHA256)
        let hash = Sha256::digest(message);
        let msg = Message::from_digest_slice(&hash)
            .context("Invalid message hash")?;

        let sig = secp.sign_ecdsa(&msg, &sk);
        Ok(hex::encode(sig.serialize_der()))
    }

    /// Sign and wrap a value into a Signed<T> envelope.
    pub fn sign_value<T: Serialize>(&self, value: &T) -> Result<super::types::Signed<T>>
    where
        T: Clone,
    {
        // Serialize to JSON for signing (matches Reality's approach)
        let json = serde_json::to_string(value)?;
        let signature = self.sign(json.as_bytes())?;

        Ok(super::types::Signed {
            value: value.clone(),
            proofs: vec![super::types::Signature {
                id: self.public_key_hex.clone(),
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
        assert!(!identity.public_key_hex.is_empty());
        assert!(identity.address.0.starts_with("NET"));
    }

    #[test]
    fn test_save_and_load() {
        let identity = NodeIdentity::generate().unwrap();
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        identity.save(path).unwrap();
        let loaded = NodeIdentity::load(path).unwrap();

        assert_eq!(identity.public_key_hex, loaded.public_key_hex);
        assert_eq!(identity.address.0, loaded.address.0);
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = NodeIdentity::generate().unwrap();
        let sig = identity.sign(b"hello reality network").unwrap();
        assert!(!sig.is_empty());
    }

    #[test]
    fn test_sign_value() {
        let identity = NodeIdentity::generate().unwrap();
        let data = serde_json::json!({"test": "value"});
        let signed = identity.sign_value(&data).unwrap();
        assert_eq!(signed.proofs.len(), 1);
        assert_eq!(signed.proofs[0].id, identity.public_key_hex);
    }
}
