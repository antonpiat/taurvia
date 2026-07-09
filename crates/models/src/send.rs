use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendPreview {
    pub from: String,
    pub to: String,
    pub token: String,
    pub amount: String,
    pub estimated_fee_lamports: u64,
    pub estimated_fee_sol: f64,
    /// True when the recipient's associated token account will be created in this transfer.
    pub creates_token_account: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SendResult {
    pub signature: String,
    pub status: String,
}
