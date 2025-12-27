use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct DeviceSyncRepository {
    pool: SqlitePool,
}

impl DeviceSyncRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create_group(&self, user_id: i64) -> Result<i64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO device_sync_groups (user_id)
            VALUES (?)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    pub async fn add_device_to_group(
        &self,
        group_id: i64,
        device_id: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO device_sync_members (sync_group_id, device_id)
            VALUES (?, ?)
            "#,
        )
        .bind(group_id)
        .bind(device_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn remove_device_from_group(&self, device_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM device_sync_members WHERE device_id = ?")
            .bind(device_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_device_group(&self, device_id: i64) -> Result<Option<i64>, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT sync_group_id
            FROM device_sync_members
            WHERE device_id = ?
            "#,
        )
        .bind(device_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| row.get(0)))
    }

    pub async fn get_group_devices(&self, group_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT device_id
            FROM device_sync_members
            WHERE sync_group_id = ?
            "#,
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn get_user_groups(&self, user_id: i64) -> Result<Vec<i64>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id
            FROM device_sync_groups
            WHERE user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }

    pub async fn delete_group(&self, group_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM device_sync_groups WHERE id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn merge_groups(
        &self,
        target_group_id: i64,
        source_group_id: i64,
    ) -> Result<(), sqlx::Error> {
        // Move all devices from source to target
        sqlx::query(
            r#"
            UPDATE device_sync_members
            SET sync_group_id = ?
            WHERE sync_group_id = ?
            "#,
        )
        .bind(target_group_id)
        .bind(source_group_id)
        .execute(&self.pool)
        .await?;

        // Delete the source group
        self.delete_group(source_group_id).await?;
        Ok(())
    }
}
