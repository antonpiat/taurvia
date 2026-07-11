use serde::{Deserialize, Serialize};
use specta::Type;

/// Public / product default Solana RPC (no user setup required).
/// Replace with a dedicated Taurvia-managed endpoint when product infra is ready.
/// Never bake a personal developer `.env` URL into release builds.
pub const MANAGED_DEFAULT_RPC_URL: &str = "https://api.mainnet-beta.solana.com";

pub const DEFAULT_AUTO_LOCK_MINUTES: u32 = 5;
pub const DEFAULT_SLIPPAGE_BPS: u16 = 50;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExplorerKind {
    #[default]
    Solscan,
    SolanaExplorer,
}

/// App chrome layout preference (synced from window size).
#[derive(Debug, Clone, Copy, Serialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum AppViewKind {
    #[default]
    Desktop,
    Compact,
    Phone,
}

impl<'de> Deserialize<'de> for AppViewKind {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "compact" => Self::Compact,
            "phone" => Self::Phone,
            // "desktop", legacy "auto", or unknown → desktop
            _ => Self::Desktop,
        })
    }
}

fn default_auto_lock_minutes() -> Option<u32> {
    Some(DEFAULT_AUTO_LOCK_MINUTES)
}

/// Missing / null → default 5 minutes. `0` disables auto-lock.
fn deserialize_auto_lock_minutes<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<u32>::deserialize(deserializer)? {
        None => Some(DEFAULT_AUTO_LOCK_MINUTES),
        Some(0) => Some(0),
        Some(minutes) => Some(minutes),
    })
}

fn default_slippage_bps() -> u16 {
    DEFAULT_SLIPPAGE_BPS
}

fn default_hide_balances() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppSettings {
    /// Optional user override (Advanced). Empty / None = managed default.
    pub rpc_url: Option<String>,
    /// Optional Jupiter portal key (Advanced). None = keyless.
    pub jupiter_api_key: Option<String>,
    /// Minutes of idle time before auto-lock. Defaults to 5. `0` disables.
    #[serde(
        default = "default_auto_lock_minutes",
        deserialize_with = "deserialize_auto_lock_minutes"
    )]
    pub auto_lock_minutes: Option<u32>,
    /// When true, mask balances in the UI. Defaults to hidden.
    #[serde(default = "default_hide_balances")]
    pub hide_balances: bool,
    #[serde(default)]
    pub explorer: ExplorerKind,
    #[serde(default = "default_slippage_bps")]
    pub default_slippage_bps: u16,
    /// Desktop / compact / phone chrome (tracks window size).
    #[serde(default)]
    pub app_view: AppViewKind,
    /// Last window width in logical pixels. Restored on launch when set.
    #[serde(default)]
    pub window_width: Option<u32>,
    /// Last window height in logical pixels. Restored on launch when set.
    #[serde(default)]
    pub window_height: Option<u32>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rpc_url: None,
            jupiter_api_key: None,
            auto_lock_minutes: default_auto_lock_minutes(),
            hide_balances: default_hide_balances(),
            explorer: ExplorerKind::Solscan,
            default_slippage_bps: default_slippage_bps(),
            app_view: AppViewKind::Desktop,
            window_width: None,
            window_height: None,
        }
    }
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
        let rpc_from_env = std::env::var("TAURVIA_RPC_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let jupiter_from_settings = settings
            .jupiter_api_key
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let jupiter_from_env = std::env::var("TAURVIA_JUPITER_API_KEY")
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
