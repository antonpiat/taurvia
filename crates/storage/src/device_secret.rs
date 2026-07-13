//! OS credential store for Enhanced device protection secrets.
//! Production uses the platform keyring; tests use an in-memory map.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

const SERVICE_NAME: &str = "com.taurvia.wallet";
const KEY_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum DeviceSecretError {
    #[error("device secret store unavailable: {0}")]
    Unavailable(String),
    #[error("device secret not found for this wallet on this device")]
    NotFound,
    #[error("invalid device secret")]
    Invalid,
}

enum Backend {
    Keyring,
    Memory(Mutex<HashMap<String, [u8; KEY_LEN]>>),
}

/// Stores the 32-byte device wrapping secret keyed by wallet_id.
pub struct DeviceSecretStore {
    backend: Backend,
}

impl DeviceSecretStore {
    /// Platform OS credential store (Keychain / Credential Manager / Secret Service).
    pub fn os() -> Self {
        Self {
            backend: Backend::Keyring,
        }
    }

    /// In-memory store for unit tests (not persisted).
    pub fn memory() -> Self {
        Self {
            backend: Backend::Memory(Mutex::new(HashMap::new())),
        }
    }

    fn entry_user(wallet_id: &str) -> String {
        format!("taurvia.wallet.{wallet_id}.device_secret")
    }

    pub fn get(&self, wallet_id: &str) -> Result<[u8; KEY_LEN], DeviceSecretError> {
        match &self.backend {
            Backend::Memory(map) => map
                .lock()
                .unwrap()
                .get(wallet_id)
                .copied()
                .ok_or(DeviceSecretError::NotFound),
            Backend::Keyring => {
                let entry = keyring::Entry::new(SERVICE_NAME, &Self::entry_user(wallet_id))
                    .map_err(|e| DeviceSecretError::Unavailable(e.to_string()))?;
                let encoded = entry.get_password().map_err(|e| match e {
                    keyring::Error::NoEntry => DeviceSecretError::NotFound,
                    other => DeviceSecretError::Unavailable(other.to_string()),
                })?;
                let bytes = BASE64
                    .decode(encoded.trim())
                    .map_err(|_| DeviceSecretError::Invalid)?;
                if bytes.len() != KEY_LEN {
                    return Err(DeviceSecretError::Invalid);
                }
                let mut out = [0u8; KEY_LEN];
                out.copy_from_slice(&bytes);
                Ok(out)
            }
        }
    }

    pub fn set(&self, wallet_id: &str, secret: &[u8; KEY_LEN]) -> Result<(), DeviceSecretError> {
        match &self.backend {
            Backend::Memory(map) => {
                map.lock().unwrap().insert(wallet_id.to_string(), *secret);
                Ok(())
            }
            Backend::Keyring => {
                let entry = keyring::Entry::new(SERVICE_NAME, &Self::entry_user(wallet_id))
                    .map_err(|e| DeviceSecretError::Unavailable(e.to_string()))?;
                entry
                    .set_password(&BASE64.encode(secret))
                    .map_err(|e| DeviceSecretError::Unavailable(e.to_string()))
            }
        }
    }

    pub fn delete(&self, wallet_id: &str) -> Result<(), DeviceSecretError> {
        match &self.backend {
            Backend::Memory(map) => {
                map.lock().unwrap().remove(wallet_id);
                Ok(())
            }
            Backend::Keyring => {
                let entry = keyring::Entry::new(SERVICE_NAME, &Self::entry_user(wallet_id))
                    .map_err(|e| DeviceSecretError::Unavailable(e.to_string()))?;
                match entry.delete_credential() {
                    Ok(()) => Ok(()),
                    Err(keyring::Error::NoEntry) => Ok(()),
                    Err(e) => Err(DeviceSecretError::Unavailable(e.to_string())),
                }
            }
        }
    }
}
