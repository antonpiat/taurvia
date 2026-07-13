use models::ApiError;
use wallet_core::WalletError;

pub fn map_wallet_error(err: WalletError) -> ApiError {
    match err {
        WalletError::AlreadyExists => ApiError::new("wallet_exists", "A wallet already exists"),
        WalletError::NotFound => ApiError::new("wallet_not_found", "Wallet not found"),
        WalletError::Locked => ApiError::new("wallet_locked", "Wallet is locked"),
        WalletError::InvalidPassword => ApiError::new("invalid_password", "Invalid password"),
        WalletError::InvalidMnemonic => ApiError::new("invalid_mnemonic", "Invalid seed phrase"),
        WalletError::DeviceSecretMissing => ApiError::new("device_secret_missing", err.to_string()),
        WalletError::Storage(e) => ApiError::new("storage_error", e.to_string()),
        WalletError::Crypto(e) => ApiError::new("crypto_error", e.to_string()),
        WalletError::Operation(e) => ApiError::new("operation_error", e.to_string()),
    }
}

pub type CommandResult<T> = Result<T, ApiError>;
