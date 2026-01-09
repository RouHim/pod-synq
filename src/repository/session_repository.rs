use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::error::AppResult;
use crate::models::Session;

use super::traits::SessionRepositoryTrait;

#[derive(Clone)]
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionRepositoryTrait for SessionRepository {
    async fn create(&self, id: &str, user_id: i64, expires_at: i64) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO sessions (id, user_id, expires_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<Session>> {
        let session = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, expires_at, created_at
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete_expired(&self, current_time: i64) -> AppResult<u64> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(current_time)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
