use serde::{Deserialize, Serialize};
use specta::Type;

pub const WALLET_FILE_VERSION: u32 = 1;
pub const DEFAULT_DERIVATION_PATH: &str = "m/44'/501'/0'/0'";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum Network {
    #[default]
    SolanaMainnet,
    SolanaDevnet,
}

impl Network {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolanaMainnet => "solana-mainnet",
            Self::SolanaDevnet => "solana-devnet",
        }
    }

    /// Parse wallet-file / settings id. Unknown values → mainnet.
    pub fn parse(value: &str) -> Self {
        match value.trim() {
            "solana-devnet" | "devnet" => Self::SolanaDevnet,
            _ => Self::SolanaMainnet,
        }
    }

    pub fn is_mainnet(self) -> bool {
        matches!(self, Self::SolanaMainnet)
    }
}

impl std::str::FromStr for Network {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

/// How the wallet ciphertext is keyed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum WalletProtection {
    /// Argon2id(password) only — portable with JSON + password.
    #[default]
    Password,
    /// Argon2id(password) + OS keychain device secret — not portable off-device.
    PasswordDevice,
}

impl WalletProtection {
    pub fn is_device_bound(self) -> bool {
        matches!(self, Self::PasswordDevice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CryptoEnvelope {
    pub kdf: String,
    pub salt: String,
    pub cipher: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EncryptedPayload {
    pub mnemonic: String,
    #[serde(rename = "private_key")]
    pub private_key: String,
    pub derivation_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletFile {
    pub version: u32,
    pub wallet_id: String,
    pub network: String,
    pub public_key: String,
    pub created_at: String,
    /// Absent in older files → password-only.
    #[serde(default)]
    pub protection: WalletProtection,
    pub crypto: CryptoEnvelope,
}
