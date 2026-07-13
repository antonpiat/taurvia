mod device_secret;

pub use device_secret::{DeviceSecretError, DeviceSecretStore};

use models::{AppSettings, WalletFile};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const DEFAULT_WALLET_FILENAME: &str = "default.json";
pub const APP_SETTINGS_FILENAME: &str = "config.json";

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("wallet not found")]
    NotFound,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Platform-agnostic wallet persistence. Desktop/mobile shells inject an impl.
pub trait WalletStore: Send + Sync {
    fn exists(&self) -> bool;
    fn save(&self, wallet: &WalletFile) -> Result<(), StorageError>;
    fn load(&self) -> Result<WalletFile, StorageError>;
    fn delete(&self) -> Result<(), StorageError>;
}

/// Restrict owner read/write only (best-effort; no-op on non-Unix).
fn restrict_owner_rw(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = path.with_extension("json.tmp");
    {
        let mut file = fs::File::create(&temp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
    }
    restrict_owner_rw(&temp_path)?;
    fs::rename(&temp_path, path)?;
    // Re-apply after rename in case the destination fs dropped mode bits.
    restrict_owner_rw(path)?;
    Ok(())
}

/// Filesystem-backed store: `{data_dir}/wallets/default.json`.
pub struct FileWalletStore {
    wallet_path: PathBuf,
}

impl FileWalletStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let wallet_path = data_dir
            .as_ref()
            .join("wallets")
            .join(DEFAULT_WALLET_FILENAME);
        Self { wallet_path }
    }

    pub fn exists(&self) -> bool {
        WalletStore::exists(self)
    }

    pub fn save(&self, wallet: &WalletFile) -> Result<(), StorageError> {
        WalletStore::save(self, wallet)
    }

    pub fn load(&self) -> Result<WalletFile, StorageError> {
        WalletStore::load(self)
    }

    pub fn delete(&self) -> Result<(), StorageError> {
        WalletStore::delete(self)
    }
}

impl WalletStore for FileWalletStore {
    fn exists(&self) -> bool {
        self.wallet_path.exists()
    }

    fn save(&self, wallet: &WalletFile) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec(wallet)?;
        atomic_write(&self.wallet_path, &serialized)
    }

    fn load(&self) -> Result<WalletFile, StorageError> {
        if !WalletStore::exists(self) {
            return Err(StorageError::NotFound);
        }
        let data = fs::read(&self.wallet_path)?;
        Ok(serde_json::from_slice(&data)?)
    }

    fn delete(&self) -> Result<(), StorageError> {
        if !WalletStore::exists(self) {
            return Err(StorageError::NotFound);
        }
        fs::remove_file(&self.wallet_path)?;
        Ok(())
    }
}

/// Persists Advanced network settings under `{data_dir}/config.json`.
pub struct AppConfigStore {
    path: PathBuf,
}

impl AppConfigStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            path: data_dir.as_ref().join(APP_SETTINGS_FILENAME),
        }
    }

    pub fn load(&self) -> Result<AppSettings, StorageError> {
        if !self.path.exists() {
            return Ok(AppSettings::default());
        }
        let data = fs::read(&self.path)?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), StorageError> {
        let serialized = serde_json::to_vec_pretty(settings)?;
        atomic_write(&self.path, &serialized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models::{CryptoEnvelope, WalletProtection, WALLET_FILE_VERSION};

    fn sample_wallet() -> WalletFile {
        WalletFile {
            version: WALLET_FILE_VERSION,
            wallet_id: "test-wallet-id".into(),
            network: "solana-mainnet".into(),
            public_key: "test".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            protection: WalletProtection::Password,
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
        let storage = FileWalletStore::new(dir.path());
        let wallet = sample_wallet();
        storage.save(&wallet).unwrap();
        let loaded = storage.load().unwrap();
        assert_eq!(loaded.wallet_id, wallet.wallet_id);
        assert_eq!(loaded.protection, WalletProtection::Password);
    }

    #[test]
    #[cfg(unix)]
    fn wallet_file_is_owner_rw_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let storage = FileWalletStore::new(dir.path());
        storage.save(&sample_wallet()).unwrap();
        let mode = fs::metadata(dir.path().join("wallets").join(DEFAULT_WALLET_FILENAME))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }

    #[test]
    fn delete_wallet() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileWalletStore::new(dir.path());
        storage.save(&sample_wallet()).unwrap();
        assert!(storage.exists());
        storage.delete().unwrap();
        assert!(!storage.exists());
    }

    #[test]
    fn save_and_load_settings() {
        let dir = tempfile::tempdir().unwrap();
        let store = AppConfigStore::new(dir.path());
        let settings = AppSettings {
            rpc_url: Some("https://example.rpc".into()),
            jupiter_api_key: None,
            ..Default::default()
        };
        store.save(&settings).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.rpc_url.as_deref(), Some("https://example.rpc"));
    }

    #[test]
    fn memory_device_secret_round_trip() {
        let store = DeviceSecretStore::memory();
        let secret = [7u8; 32];
        store.set("wid", &secret).unwrap();
        assert_eq!(store.get("wid").unwrap(), secret);
        store.delete("wid").unwrap();
        assert!(matches!(store.get("wid"), Err(DeviceSecretError::NotFound)));
    }
}
