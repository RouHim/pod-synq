use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSession {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdminSession {
    pub user_id: i64,
    pub expires_at: i64,
}
