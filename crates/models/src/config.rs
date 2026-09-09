use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;

use crate::{
    env_rpc_override, managed_rpc_url, normalize_network_id, require_network, DEFAULT_NETWORK_ID,
};

/// Public / product default Solana mainnet RPC (no user setup required).
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

fn default_swap_favorite_tokens() -> Vec<crate::TokenInfo> {
    Vec::new()
}

fn default_network() -> String {
    DEFAULT_NETWORK_ID.to_string()
}

fn default_enabled_networks() -> Vec<String> {
    crate::default_enabled_network_ids()
}

fn default_rpc_urls() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppSettings {
    /// Optional user override (Advanced). Empty / None = managed default for `network`.
    /// Legacy single-URL field; treated as the Solana override when `rpc_urls` has no Solana key.
    pub rpc_url: Option<String>,
    /// Per-network RPC / Esplora overrides. Keyed by network id.
    #[serde(default = "default_rpc_urls")]
    pub rpc_urls: HashMap<String, String>,
    /// Optional Jupiter portal key (Advanced). None = keyless.
    pub jupiter_api_key: Option<String>,
    /// Active network id. Synced with `WalletFile.network` on switch.
    #[serde(default = "default_network")]
    pub network: String,
    /// Activated mainnets (Phantom-style). Testnets stay Advanced via `network`.
    #[serde(default = "default_enabled_networks")]
    pub enabled_networks: Vec<String>,
    /// Optional 0x API key for Ethereum swaps.
    #[serde(default)]
    pub zerox_api_key: Option<String>,
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
    /// User-added Swap tokens (keyword search selections). Instant on reopen.
    #[serde(default = "default_swap_favorite_tokens")]
    pub swap_favorite_tokens: Vec<crate::TokenInfo>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            rpc_url: None,
            rpc_urls: HashMap::new(),
            jupiter_api_key: None,
            network: default_network(),
            enabled_networks: default_enabled_networks(),
            zerox_api_key: None,
            auto_lock_minutes: default_auto_lock_minutes(),
            hide_balances: default_hide_balances(),
            explorer: ExplorerKind::Solscan,
            default_slippage_bps: default_slippage_bps(),
            app_view: AppViewKind::Desktop,
            window_width: None,
            window_height: None,
            swap_favorite_tokens: default_swap_favorite_tokens(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct RuntimeConfig {
    pub rpc_url: String,
    pub jupiter_api_key: Option<String>,
}

impl RuntimeConfig {
    /// Resolution: per-network map → legacy rpc_url (Solana) → env → managed default.
    pub fn resolve(settings: &AppSettings) -> Self {
        let network_id = normalize_network_id(&settings.network);
        let desc = require_network(network_id);

        let from_map = settings
            .rpc_urls
            .get(network_id)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let from_legacy = settings
            .rpc_url
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .filter(|_| desc.family == crate::ChainFamily::Solana);

        let from_env = env_rpc_override(desc.family);

        let jupiter_from_settings = settings
            .jupiter_api_key
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        let jupiter_from_env = std::env::var("TAURVIA_JUPITER_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let managed = managed_rpc_url(network_id).to_string();

        Self {
            rpc_url: from_map.or(from_legacy).or(from_env).unwrap_or(managed),
            jupiter_api_key: jupiter_from_settings.or(jupiter_from_env),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct OnboardingDraft {
    #[serde(default)]
    pub mnemonic: String,
    pub mode: String,
    /// Raw private key when `mode` is `import-key`.
    #[serde(default)]
    pub secret: String,
    #[serde(default)]
    pub import_kind: String,
    #[serde(default)]
    pub account_name: String,
}
