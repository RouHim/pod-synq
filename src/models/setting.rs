use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
    pub id: i64,
    pub user_id: i64,
    pub scope: String,
    pub podcast_url: Option<String>,
    pub device_id: Option<i64>,
    pub episode_url: Option<String>,
    pub key: String,
    pub value: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRequest {
    pub set: Option<serde_json::Map<String, serde_json::Value>>,
    pub remove: Option<Vec<String>>,
}
