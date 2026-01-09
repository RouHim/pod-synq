use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::error::AppResult;
use crate::models::Device;

use super::traits::DeviceRepositoryTrait;

#[derive(Clone)]
pub struct DeviceRepository {
    pool: SqlitePool,
}

impl DeviceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeviceRepositoryTrait for DeviceRepository {
    async fn create(
        &self,
        user_id: i64,
        device_id: &str,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO devices (user_id, device_id, caption, type)
            VALUES (?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(caption)
        .bind(device_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get::<i64, _>("id"))
    }

    async fn find_by_device_id(&self, user_id: i64, device_id: &str) -> AppResult<Option<Device>> {
        let device = sqlx::query_as::<_, Device>(
            r#"
            SELECT 
                id, user_id, device_id, caption, type, 
                created_at, updated_at
            FROM devices 
            WHERE user_id = ? AND device_id = ?
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(device)
    }

    async fn list_by_user(&self, user_id: i64) -> AppResult<Vec<Device>> {
        let devices = sqlx::query_as::<_, Device>(
            r#"
            SELECT 
                id, user_id, device_id, caption, type, 
                created_at, updated_at
            FROM devices 
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(devices)
    }
}
