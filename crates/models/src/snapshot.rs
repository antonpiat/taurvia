use serde::{Deserialize, Serialize};
use specta::Type;

use crate::TokenBalance;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WalletSnapshot {
    pub exists: bool,
    pub unlocked: bool,
    /// Wallet file network id (e.g. `solana-mainnet`). Empty wallet → default mainnet.
    pub network: String,
    pub public_key: Option<String>,
    pub sol_balance: Option<f64>,
    pub sol_price_usd: Option<f64>,
    pub sol_value_usd: Option<f64>,
    pub total_portfolio_usd: Option<f64>,
    pub tokens: Option<Vec<TokenBalance>>,
}
