use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: i64,
    pub user_id: i64,
    pub device_id: i64,
    pub podcast_url: String,
    pub added_at: i64,
    pub removed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscription {
    pub user_id: i64,
    pub device_id: i64,
    pub podcast_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveSubscription {
    pub user_id: i64,
    pub device_id: i64,
    pub podcast_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionChanges {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}
