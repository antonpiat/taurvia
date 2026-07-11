mod balances;
mod send;
mod session;
mod swap;
mod wallet_file;

pub use session::WalletService;

use storage::StorageError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("wallet already exists")]
    AlreadyExists,
    #[error("wallet not found")]
    NotFound,
    #[error("wallet is locked")]
    Locked,
    #[error("invalid password")]
    InvalidPassword,
    #[error("invalid mnemonic")]
    InvalidMnemonic,
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("crypto error: {0}")]
    Crypto(#[from] crypto::CryptoError),
    #[error("operation failed: {0}")]
    Operation(#[from] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_import_unlock_flow() {
        let dir = tempfile::tempdir().unwrap();
        let service = WalletService::new(dir.path(), Some("http://localhost:8899"));
        let mnemonic = service.generate_mnemonic().unwrap();
        service.create_wallet(&mnemonic, "password123").unwrap();
        assert!(service.wallet_exists());
        let pubkey = service.unlock("password123").unwrap();
        assert!(!pubkey.is_empty());
        let revealed = service.reveal_mnemonic("password123").unwrap();
        assert_eq!(revealed, mnemonic);
        service
            .change_password("password123", "password456")
            .unwrap();
        service.lock();
        assert!(!service.is_unlocked());
        let pubkey2 = service.unlock("password456").unwrap();
        assert_eq!(pubkey, pubkey2);
        let exported = service.export_wallet("password456").unwrap();
        assert!(exported.contains("crypto"));
        assert!(!exported.contains(&mnemonic));
    }
}
