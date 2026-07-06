use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub signature: String,
    pub timestamp: Option<i64>,
    pub status: String,
    pub direction: String,
    pub amount_sol: Option<f64>,
    pub description: String,
}
