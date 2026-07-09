use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TokenBalance {
    pub mint: String,
    pub symbol: String,
    pub name: String,
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: f64,
}
