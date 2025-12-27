use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChanges {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}
