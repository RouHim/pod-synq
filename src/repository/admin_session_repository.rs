#![allow(unused_variables)]
use crate::models::AdminSession;
use sqlx::{Error, Row, SqlitePool};

#[derive(Clone)]
pub struct AdminSessionRepository {
    pool: SqlitePool,
}

impl AdminSessionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: i64, expires_at: i64) -> Result<AdminSession, Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let result = sqlx::query(
            r#"
            INSERT INTO admin_sessions (id, user_id, expires_at)
            VALUES (?, ?, ?)
            RETURNING id, user_id, expires_at, created_at
            "#,
        )
        .bind(&id)
        .bind(user_id)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(AdminSession {
            id: result.get_unchecked::<&str, _>(0).to_string(),
            user_id: result.get_unchecked::<i64, _>(1),
            expires_at: result.get_unchecked::<i64, _>(2),
            created_at: result.get_unchecked::<i64, _>(3),
        })
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<AdminSession>, Error> {
        let result = sqlx::query(
            r#"
            SELECT id, user_id, expires_at, created_at
            FROM admin_sessions 
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| AdminSession {
            id: row.get_unchecked::<&str, _>(0).to_string(),
            user_id: row.get_unchecked::<i64, _>(1),
            expires_at: row.get_unchecked::<i64, _>(2),
            created_at: row.get_unchecked::<i64, _>(3),
        }))
    }

    pub async fn delete(&self, id: &str) -> Result<(), Error> {
        sqlx::query("DELETE FROM admin_sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_by_user(&self, user_id: i64) -> Result<(), Error> {
        sqlx::query("DELETE FROM admin_sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn cleanup_expired(&self) -> Result<(), Error> {
        let now = chrono::Utc::now().timestamp();
        sqlx::query("DELETE FROM admin_sessions WHERE expires_at < ?")
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn is_valid(&self, id: &str) -> Result<bool, Error> {
        let now = chrono::Utc::now().timestamp();
        let _result = sqlx::query(
            r#"
            SELECT COUNT(*) as count 
            FROM admin_sessions 
            WHERE id = ? AND expires_at > ?
            "#,
        )
        .bind(id)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(_result.get_unchecked::<i64, _>(0).as_i64() > 0)
    }
}
