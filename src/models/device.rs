use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Device {
    pub id: i64,
    pub user_id: i64,
    pub device_id: String,
    pub caption: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
