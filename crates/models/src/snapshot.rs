use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{ImportKind, TokenBalance};

/// One family's balances inside a multi-chain portfolio.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ChainSnapshot {
    pub network: String,
    pub public_key: Option<String>,
    pub native_balance: Option<f64>,
    pub native_symbol: String,
    pub native_price_usd: Option<f64>,
    pub native_value_usd: Option<f64>,
    pub total_usd: Option<f64>,
    pub tokens: Option<Vec<TokenBalance>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletSnapshot {
    pub exists: bool,
    pub unlocked: bool,
    /// Last-used network for Send / Receive (not an exclusive mode).
    pub network: String,
    /// Last-used chain receive address (public).
    pub public_key: Option<String>,
    pub native_balance: Option<f64>,
    pub native_symbol: String,
    pub native_price_usd: Option<f64>,
    pub native_value_usd: Option<f64>,
    pub total_portfolio_usd: Option<f64>,
    pub tokens: Option<Vec<TokenBalance>>,
    #[serde(default)]
    pub chains: Vec<ChainSnapshot>,
    #[serde(default)]
    pub account_name: String,
    #[serde(default)]
    pub import_kind: ImportKind,
    #[serde(default)]
    pub enabled_networks: Vec<String>,
    #[serde(default)]
    pub can_reveal_mnemonic: bool,
}

impl WalletSnapshot {
    pub fn empty(network: String, native_symbol: String) -> Self {
        Self {
            exists: false,
            unlocked: false,
            network,
            public_key: None,
            native_balance: None,
            native_symbol,
            native_price_usd: None,
            native_value_usd: None,
            total_portfolio_usd: None,
            tokens: None,
            chains: Vec::new(),
            account_name: crate::DEFAULT_ACCOUNT_NAME.to_string(),
            import_kind: ImportKind::Mnemonic,
            enabled_networks: crate::default_enabled_network_ids(),
            can_reveal_mnemonic: false,
        }
    }
}
