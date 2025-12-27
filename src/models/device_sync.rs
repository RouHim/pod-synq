use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceSyncGroup {
    pub id: i64,
    pub user_id: i64,
    pub created_at: i64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DeviceSyncMember {
    pub id: i64,
    pub sync_group_id: i64,
    pub device_id: i64,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct DeviceSyncStatus {
    pub synchronized: Vec<Vec<String>>,
    #[serde(rename = "not-synchronized")]
    pub not_synchronized: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceSyncRequest {
    #[serde(default)]
    pub synchronize: Vec<Vec<String>>,
    #[serde(rename = "stop-synchronize", default)]
    pub stop_synchronize: Vec<String>,
}
