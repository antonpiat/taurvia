use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActivityItem {
    pub signature: String,
    pub timestamp: Option<i64>,
    pub status: String,
    pub direction: String,
    pub amount_sol: Option<f64>,
    pub description: String,
}
