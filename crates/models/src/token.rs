use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
}
