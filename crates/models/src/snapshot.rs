use serde::{Deserialize, Serialize};

use crate::TokenBalance;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSnapshot {
    pub exists: bool,
    pub unlocked: bool,
    pub public_key: Option<String>,
    pub sol_balance: Option<f64>,
    pub tokens: Option<Vec<TokenBalance>>,
}
