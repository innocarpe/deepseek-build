//! Prefix epoch = SHA-256 of stable_prefix_bytes (spec 10 §1.5).

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefixEpoch {
    /// Full SHA-256 hex of stable prefix bytes.
    pub sha256_hex: String,
}

impl PrefixEpoch {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let digest = hasher.finalize();
        Self {
            sha256_hex: hex::encode(digest),
        }
    }

    /// First 16 hex chars for logs (`prefix_epoch=`).
    pub fn short(&self) -> &str {
        let n = self.sha256_hex.len().min(16);
        &self.sha256_hex[..n]
    }

    pub fn log_label(&self) -> String {
        format!("prefix_epoch={}", self.short())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_stable_for_same_bytes() {
        let e1 = PrefixEpoch::from_bytes(b"abc");
        let e2 = PrefixEpoch::from_bytes(b"abc");
        assert_eq!(e1, e2);
        assert_eq!(e1.short().len(), 16);
    }

    #[test]
    fn epoch_changes_when_bytes_change() {
        let e1 = PrefixEpoch::from_bytes(b"abc");
        let e2 = PrefixEpoch::from_bytes(b"abd");
        assert_ne!(e1.sha256_hex, e2.sha256_hex);
    }
}
