use serde::{Deserialize, Serialize};
use specta::Type;

/// Public / product default Solana RPC (no user setup required).
/// Replace with a dedicated Aegis-managed endpoint when product infra is ready.
/// Never bake a personal developer `.env` URL into release builds.
pub const MANAGED_DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
pub struct AppSettings {
    /// Optional user override (Advanced). Empty / None = managed default.
    pub rpc_url: Option<String>,
    /// Optional Jupiter portal key (Advanced). None = keyless.
    pub jupiter_api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuntimeConfig {
    pub rpc_url: String,
    pub jupiter_api_key: Option<String>,
}

impl RuntimeConfig {
    /// Resolution: user Advanced settings → process env (dev) → managed default.
    pub fn resolve(settings: &AppSettings) -> Self {
        let rpc_from_settings = settings
            .rpc_url
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let rpc_from_env = std::env::var("AEGIS_RPC_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let jupiter_from_settings = settings
            .jupiter_api_key
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let jupiter_from_env = std::env::var("AEGIS_JUPITER_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        Self {
            rpc_url: rpc_from_settings
                .or(rpc_from_env)
                .unwrap_or_else(|| MANAGED_DEFAULT_RPC_URL.to_string()),
            jupiter_api_key: jupiter_from_settings.or(jupiter_from_env),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OnboardingDraft {
    pub mnemonic: String,
    pub mode: String,
}
