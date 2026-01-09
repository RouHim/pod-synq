use async_trait::async_trait;

use crate::error::AppResult;
use crate::models::{User, UserRow};
use crate::repository::traits::UserRepositoryTrait;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    async fn create(&self, username: &str, password_hash: &str, is_admin: bool) -> AppResult<User> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            INSERT INTO users (username, password_hash, is_admin)
            VALUES (?, ?, ?)
            RETURNING id, username, password_hash, CAST(is_admin AS INTEGER) as is_admin, created_at
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .bind(is_admin)
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn find_by_id(&self, id: i64) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                id, username, password_hash,
                CAST(is_admin AS INTEGER) as is_admin,
                created_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let row = sqlx::query_as::<_, UserRow>(
            r#"
            SELECT
                id, username, password_hash,
                CAST(is_admin AS INTEGER) as is_admin,
                created_at
            FROM users
            WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn is_empty(&self) -> AppResult<bool> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(count.0 == 0)
    }
}
