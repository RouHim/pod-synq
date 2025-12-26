use crate::models::Device;
use sqlx::{Error, Row, SqlitePool};

#[derive(Clone)]
pub struct DeviceRepository {
    pool: SqlitePool,
}

impl DeviceRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i64,
        device_id: &str,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> Result<i64, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO devices (user_id, device_id, caption, type)
            VALUES (?, ?, ?, ?)
            RETURNING id, user_id, device_id, caption, type, created_at, updated_at
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(caption)
        .bind(device_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get_unchecked::<i64, _>(0))
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Device>, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT 
                id, user_id, device_id, caption, type, 
                created_at, updated_at
            FROM devices 
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| Device {
            id: row.get_unchecked::<i64, _>(0),
            user_id: row.get_unchecked::<i64, _>(1),
            device_id: row.get_unchecked::<&str, _>(2).to_string(),
            caption: row.get_unchecked::<Option<String>, _>(3),
            r#type: row.get_unchecked::<Option<String>, _>(4),
            created_at: row.get_unchecked(5),
            updated_at: row.get_unchecked::<i64, _>(6),
        }))
    }

    pub async fn find_by_device_id(
        &self,
        user_id: i64,
        device_id: &str,
    ) -> Result<Option<Device>, sqlx::Error> {
        let result = sqlx::query(
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

        Ok(result.map(|row| Device {
            id: row.get_unchecked::<i64, _>(0),
            user_id: row.get_unchecked::<i64, _>(1),
            device_id: row.get_unchecked::<&str, _>(2).to_string(),
            caption: row.get_unchecked::<Option<String>, _>(3),
            r#type: row.get_unchecked::<Option<String>, _>(4),
            created_at: row.get_unchecked(5),
            updated_at: row.get_unchecked::<i64, _>(6),
        }))
    }

    pub async fn list_by_user(&self, user_id: i64) -> Result<Vec<Device>, Error> {
        let rows = sqlx::query(
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

        Ok(rows
            .into_iter()
            .map(|row| Device {
                id: row.get_unchecked::<i64, _>(0),
                user_id: row.get_unchecked::<i64, _>(1),
                device_id: row.get_unchecked::<&str, _>(2).to_string(),
                caption: row.get_unchecked::<Option<String>, _>(3),
                r#type: row.get_unchecked::<Option<String>, _>(4),
                created_at: row.get_unchecked(5),
                updated_at: row.get_unchecked::<i64, _>(6),
            })
            .collect())
    }

    pub async fn update(
        &self,
        id: i64,
        caption: Option<&str>,
        device_type: Option<&str>,
    ) -> Result<Device, Error> {
        let result = sqlx::query(
            r#"
            UPDATE devices 
            SET caption = ?, type = ?, updated_at = strftime('%s', 'now')
            WHERE id = ?
            RETURNING id, user_id, device_id, caption, type, created_at, updated_at
            "#,
        )
        .bind(caption)
        .bind(device_type)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(Device {
            id: result.get_unchecked::<i64, _>(0),
            user_id: result.get_unchecked::<i64, _>(1),
            device_id: result.get_unchecked::<&str, _>(2).to_string(),
            caption: result.get_unchecked::<Option<String>, _>(3),
            r#type: result.get_unchecked::<Option<String>, _>(4),
            created_at: result.get_unchecked(5),
            updated_at: result.get_unchecked::<i64, _>(6),
        })
    }

    pub async fn delete(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM devices WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count(&self, user_id: Option<i64>) -> Result<i64, sqlx::Error> {
        if let Some(user_id) = user_id {
            let result = sqlx::query("SELECT COUNT(*) as count FROM devices WHERE user_id = ?")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        } else {
            let result = sqlx::query("SELECT COUNT(*) as count FROM devices")
                .fetch_one(&self.pool)
                .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        }
    }
}
