use crate::models::Session;
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct SessionRepository {
    pool: SqlitePool,
}

impl SessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, id: &str, user_id: i64, expires_at: i64) -> Result<(), sqlx::Error> {
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

    pub async fn find_by_id(&self, id: &str) -> Result<Option<Session>, sqlx::Error> {
        sqlx::query_as::<_, Session>(
            r#"
            SELECT id, user_id, expires_at, created_at
            FROM sessions
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn delete_expired(&self, current_time: i64) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM sessions WHERE expires_at < ?")
            .bind(current_time)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
