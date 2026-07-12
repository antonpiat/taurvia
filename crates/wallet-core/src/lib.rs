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

    #[tokio::test]
    async fn update_settings_cannot_desync_network_from_wallet_file() {
        let dir = tempfile::tempdir().unwrap();
        let service = WalletService::new(dir.path(), Some("http://localhost:8899"));
        let mnemonic = service.generate_mnemonic().unwrap();
        service.create_wallet(&mnemonic, "password123").unwrap();
        service
            .change_network(models::Network::SolanaDevnet)
            .unwrap();

        let mut settings = service.get_settings();
        settings.network = models::Network::SolanaMainnet;
        let _ = service.update_settings(settings).unwrap();

        assert_eq!(service.get_settings().network, models::Network::SolanaDevnet);
        assert_eq!(service.wallet_network(), "solana-devnet");
    }

    #[tokio::test]
    async fn swap_rejected_on_devnet() {
        let dir = tempfile::tempdir().unwrap();
        let service = WalletService::new(dir.path(), Some("http://localhost:8899"));
        let mnemonic = service.generate_mnemonic().unwrap();
        service.create_wallet(&mnemonic, "password123").unwrap();
        service.unlock("password123").unwrap();
        service
            .change_network(models::Network::SolanaDevnet)
            .unwrap();

        let err = service
            .preview_swap(
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                0.01,
                50,
            )
            .await
            .unwrap_err();
        assert!(err.to_string().contains("Mainnet"));
    }
}
