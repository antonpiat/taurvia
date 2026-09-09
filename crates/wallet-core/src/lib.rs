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
    #[error(
        "this wallet is bound to another device; restore with your recovery phrase, or disable Enhanced device protection before exporting on the original device"
    )]
    DeviceSecretMissing,
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
    use storage::DeviceSecretStore;

    fn test_service(dir: &std::path::Path) -> WalletService {
        WalletService::with_device_secrets(
            dir,
            Some("http://localhost:8899"),
            DeviceSecretStore::memory(),
        )
    }

    #[tokio::test]
    async fn create_import_unlock_flow() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        assert!(service.wallet_exists());
        let pubkey = service.unlock("Password123!").unwrap();
        assert!(!pubkey.is_empty());
        let mut revealed = service.reveal_mnemonic("Password123!").unwrap();
        assert_eq!(revealed, mnemonic);
        assert!(service.reveal_mnemonic("WrongPass1!").is_err());
        service
            .change_password("Password123!", "Password456!")
            .unwrap();
        service.lock();
        assert!(!service.is_unlocked());
        assert!(service.reveal_mnemonic("Password456!").is_err());
        let pubkey2 = service.unlock("Password456!").unwrap();
        assert_eq!(pubkey, pubkey2);
        revealed = service.reveal_mnemonic("Password456!").unwrap();
        assert_eq!(revealed, mnemonic);
        let exported = service.export_wallet("Password456!").unwrap();
        assert!(exported.contains("crypto"));
        assert!(!exported.contains(&mnemonic));
    }

    #[tokio::test]
    async fn weak_password_rejected_on_create() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        let err = service
            .create_wallet(&mnemonic, "password123", "Account 1")
            .unwrap_err();
        assert!(err.to_string().contains("uppercase"));
    }

    #[tokio::test]
    async fn update_settings_cannot_desync_network_from_wallet_file() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        service.change_network("solana-devnet").unwrap();

        let mut settings = service.get_settings();
        settings.network = "solana-mainnet".into();
        let _ = service.update_settings(settings).unwrap();

        assert_eq!(service.get_settings().network, "solana-devnet");
        assert_eq!(service.wallet_network(), "solana-devnet");
    }

    #[tokio::test]
    async fn swap_rejected_on_devnet() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        service.unlock("Password123!").unwrap();
        service
            .set_enabled_networks(&["ethereum-mainnet".into()])
            .unwrap();

        let preview_err = service
            .preview_swap(
                "So11111111111111111111111111111111111111112",
                "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                0.01,
                50,
            )
            .await
            .unwrap_err();
        assert!(
            preview_err.to_string().to_lowercase().contains("solana")
                || preview_err.to_string().to_lowercase().contains("activate"),
            "{preview_err}"
        );
    }

    #[tokio::test]
    async fn seed_wallet_snapshot_lists_enabled_networks() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        service.unlock("Password123!").unwrap();
        let enabled = service.enabled_network_ids();
        assert!(enabled.contains(&"solana-mainnet".to_string()));
        assert!(enabled.contains(&"ethereum-mainnet".to_string()));
        assert!(enabled.contains(&"bitcoin-mainnet".to_string()));
        let snap = service.get_snapshot().await.unwrap();
        assert_eq!(snap.account_name, "Account 1");
        assert!(snap.can_reveal_mnemonic);
        assert_eq!(snap.import_kind, models::ImportKind::Mnemonic);
        assert_eq!(snap.enabled_networks, enabled);
    }

    #[tokio::test]
    async fn import_solana_key_hides_other_families() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        let seed = taurvia_hd::seed_from_mnemonic(&mnemonic).unwrap();
        let kp = taurvia_solana::derive_keypair_from_seed(seed.as_slice()).unwrap();
        let secret = taurvia_solana::keypair_to_base64(&kp);
        service
            .import_private_key(&secret, "Password123!", "Trading")
            .unwrap();
        service.unlock("Password123!").unwrap();
        assert_eq!(service.account_name(), "Trading");
        assert!(!service.import_kind().has_mnemonic());
        assert!(service.reveal_mnemonic("Password123!").is_err());
        let err = service.change_network("ethereum-mainnet").unwrap_err();
        assert!(
            err.to_string().to_lowercase().contains("ethereum") || err.to_string().contains("key")
        );
        let enable_err = service
            .set_enabled_networks(&["ethereum-mainnet".into(), "solana-mainnet".into()])
            .unwrap_err();
        assert!(
            enable_err.to_string().to_lowercase().contains("solana")
                || enable_err.to_string().contains("Ethereum")
                || enable_err.to_string().contains("only")
        );
    }

    #[tokio::test]
    async fn import_evm_hex_key_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        // secp256k1 one (not on curve check — k256 accepts this well-known test vector's complement; use a valid key)
        let secret = "0x4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318";
        service
            .import_private_key(secret, "Password123!", "ETH key")
            .unwrap();
        let addr = service.unlock("Password123!").unwrap();
        assert!(addr.starts_with("0x"));
        assert_eq!(service.import_kind(), models::ImportKind::EvmKey);
        assert_eq!(
            service.enabled_network_ids(),
            vec!["ethereum-mainnet".to_string()]
        );
        assert!(service.change_network("solana-mainnet").is_err());
    }

    #[tokio::test]
    async fn import_bitcoin_wif_key_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let secret = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
        service
            .import_private_key(secret, "Password123!", "BTC key")
            .unwrap();
        let addr = service.unlock("Password123!").unwrap();
        assert!(addr.starts_with("bc1"));
        assert_eq!(service.import_kind(), models::ImportKind::BitcoinKey);
        assert_eq!(
            service.enabled_network_ids(),
            vec!["bitcoin-mainnet".to_string()]
        );
        assert!(service.change_network("ethereum-mainnet").is_err());
        assert!(service.reveal_mnemonic("Password123!").is_err());
    }

    #[tokio::test]
    async fn device_protection_blocks_off_device_decrypt() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        service.unlock("Password123!").unwrap();
        service.enable_device_protection("Password123!").unwrap();
        assert!(service.device_protection_enabled());
        let exported = service.export_wallet("Password123!").unwrap();

        let dir2 = tempfile::tempdir().unwrap();
        let other = test_service(dir2.path());
        let err = other
            .import_wallet_backup(&exported, "Password123!")
            .unwrap_err();
        assert!(matches!(err, WalletError::DeviceSecretMissing));

        other
            .import_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        other.unlock("Password123!").unwrap();
        assert_eq!(other.reveal_mnemonic("Password123!").unwrap(), mnemonic);
    }

    #[tokio::test]
    async fn import_password_only_backup() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        let exported = service.export_wallet("Password123!").unwrap();

        let dir2 = tempfile::tempdir().unwrap();
        let other = test_service(dir2.path());
        other
            .import_wallet_backup(&exported, "Password123!")
            .unwrap();
        assert!(other.is_unlocked());
        assert_eq!(other.reveal_mnemonic("Password123!").unwrap(), mnemonic);
    }

    #[tokio::test]
    async fn disable_device_protection_makes_backup_portable() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        service.unlock("Password123!").unwrap();
        service.enable_device_protection("Password123!").unwrap();
        service.disable_device_protection("Password123!").unwrap();
        assert!(!service.device_protection_enabled());
        let exported = service.export_wallet("Password123!").unwrap();

        let dir2 = tempfile::tempdir().unwrap();
        let other = test_service(dir2.path());
        other
            .import_wallet_backup(&exported, "Password123!")
            .unwrap();
        assert_eq!(other.reveal_mnemonic("Password123!").unwrap(), mnemonic);
    }

    #[tokio::test]
    async fn reset_local_wallet_without_password() {
        let dir = tempfile::tempdir().unwrap();
        let service = test_service(dir.path());
        let mnemonic = service.generate_mnemonic().unwrap();
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        assert!(service.wallet_exists());
        service.reset_local_wallet().unwrap();
        assert!(!service.wallet_exists());
        assert!(!service.is_unlocked());
        // Can recreate after wipe.
        service
            .create_wallet(&mnemonic, "Password123!", "Account 1")
            .unwrap();
        assert!(service.wallet_exists());
    }
}
