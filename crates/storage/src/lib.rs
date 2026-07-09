use models::WalletFile;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const DEFAULT_WALLET_FILENAME: &str = "default.json";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("wallet not found")]
    NotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct WalletStorage {
    wallet_path: PathBuf,
}

impl WalletStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let wallet_path = data_dir
            .as_ref()
            .join("wallets")
            .join(DEFAULT_WALLET_FILENAME);
        Self { wallet_path }
    }

    pub fn exists(&self) -> bool {
        self.wallet_path.exists()
    }

    pub fn save(&self, wallet: &WalletFile) -> Result<(), StorageError> {
        if let Some(parent) = self.wallet_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let serialized = serde_json::to_vec(wallet)?;
        let temp_path = self.wallet_path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&temp_path)?;
            file.write_all(&serialized)?;
            file.sync_all()?;
        }
        fs::rename(temp_path, &self.wallet_path)?;
        Ok(())
    }

    pub fn load(&self) -> Result<WalletFile, StorageError> {
        if !self.exists() {
            return Err(StorageError::NotFound);
        }
        let data = fs::read(&self.wallet_path)?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub fn delete(&self) -> Result<(), StorageError> {
        if !self.exists() {
            return Err(StorageError::NotFound);
        }
        fs::remove_file(&self.wallet_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{CryptoEnvelope, WALLET_FILE_VERSION};

    fn sample_wallet() -> WalletFile {
        WalletFile {
            version: WALLET_FILE_VERSION,
            wallet_id: "test-wallet-id".into(),
            network: "solana-mainnet".into(),
            public_key: "test".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            crypto: CryptoEnvelope {
                kdf: "argon2id".into(),
                salt: "c2FsdA==".into(),
                cipher: "aes-256-gcm".into(),
                nonce: "bm9uY2U=".into(),
                ciphertext: "Y2lwaGVydGV4dA==".into(),
            },
        }
    }

    #[test]
    fn save_and_load_wallet() {
        let dir = tempfile::tempdir().unwrap();
        let storage = WalletStorage::new(dir.path());
        let wallet = sample_wallet();
        storage.save(&wallet).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.wallet_id, wallet.wallet_id);
    }

    #[test]
    fn delete_wallet() {
        let dir = tempfile::tempdir().unwrap();
        let storage = WalletStorage::new(dir.path());
        storage.save(&sample_wallet()).unwrap();
        assert!(storage.exists());
        storage.delete().unwrap();
        assert!(!storage.exists());
    }
}
