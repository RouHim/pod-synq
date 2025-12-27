use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub user_id: i64,
    pub device_id: String,
    pub caption: Option<String>,
    pub r#type: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
