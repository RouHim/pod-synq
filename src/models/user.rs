use serde::{Deserialize, Serialize};

/// Raw user row from database with i32 for is_admin
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserRow {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: i32,
    pub created_at: i64,
}

/// User model with proper boolean type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub is_admin: bool,
    pub created_at: i64,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        User {
            id: row.id,
            username: row.username,
            password_hash: row.password_hash,
            is_admin: row.is_admin != 0,
            created_at: row.created_at,
        }
    }
}
