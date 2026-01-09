use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::error::AppResult;
use crate::models::Setting;

use super::traits::SettingRepositoryTrait;

#[derive(Clone)]
pub struct SettingRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct SettingKey<'a> {
    pub user_id: i64,
    pub scope: &'a str,
    pub podcast_url: Option<&'a str>,
    pub device_id: Option<i64>,
    pub episode_url: Option<&'a str>,
    pub key: &'a str,
}

impl SettingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SettingRepositoryTrait for SettingRepository {
    async fn get_settings(
        &self,
        user_id: i64,
        scope: &str,
        podcast_url: Option<&str>,
        device_id: Option<i64>,
        episode_url: Option<&str>,
    ) -> AppResult<Vec<Setting>> {
        let (podcast_condition, podcast_bind) = if let Some(url) = podcast_url {
            ("podcast_url = ?".to_string(), Some(url))
        } else {
            ("podcast_url IS NULL".to_string(), None)
        };

        let (device_condition, device_bind) = if let Some(id) = device_id {
            ("device_id = ?".to_string(), Some(id))
        } else {
            ("device_id IS NULL".to_string(), None)
        };

        let (episode_condition, episode_bind) = if let Some(url) = episode_url {
            ("episode_url = ?".to_string(), Some(url))
        } else {
            ("episode_url IS NULL".to_string(), None)
        };

        let query = format!(
            r#"
            SELECT id, user_id, scope, podcast_url, device_id, episode_url, key, value, created_at, updated_at
            FROM settings
            WHERE user_id = ? AND scope = ? AND {} AND {} AND {}
            ORDER BY key ASC
            "#,
            podcast_condition, device_condition, episode_condition
        );

        // Build query dynamically based on which parameters are present
        // We can't use query_as with dynamic SQL easily, so we keep using Row::get
        let mut q = sqlx::query(&query).bind(user_id).bind(scope);
        if let Some(url) = podcast_bind {
            q = q.bind(url);
        }
        if let Some(id) = device_bind {
            q = q.bind(id);
        }
        if let Some(url) = episode_bind {
            q = q.bind(url);
        }

        let rows = q.fetch_all(&self.pool).await?;

        Ok(rows
            .into_iter()
            .map(|row| Setting {
                id: row.get("id"),
                user_id: row.get("user_id"),
                scope: row.get("scope"),
                podcast_url: row.get("podcast_url"),
                device_id: row.get("device_id"),
                episode_url: row.get("episode_url"),
                key: row.get("key"),
                value: row.get("value"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect())
    }

    async fn upsert_setting(&self, key: SettingKey<'_>, value: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (user_id, scope, podcast_url, device_id, episode_url, key, value)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, scope, podcast_url, device_id, episode_url, key)
            DO UPDATE SET value = ?, updated_at = strftime('%s', 'now')
            "#,
        )
        .bind(key.user_id)
        .bind(key.scope)
        .bind(key.podcast_url)
        .bind(key.device_id)
        .bind(key.episode_url)
        .bind(key.key)
        .bind(value)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_setting(&self, key: SettingKey<'_>) -> AppResult<()> {
        let (podcast_condition, podcast_bind) = if let Some(url) = key.podcast_url {
            ("podcast_url = ?".to_string(), Some(url))
        } else {
            ("podcast_url IS NULL".to_string(), None)
        };

        let (device_condition, device_bind) = if let Some(id) = key.device_id {
            ("device_id = ?".to_string(), Some(id))
        } else {
            ("device_id IS NULL".to_string(), None)
        };

        let (episode_condition, episode_bind) = if let Some(url) = key.episode_url {
            ("episode_url = ?".to_string(), Some(url))
        } else {
            ("episode_url IS NULL".to_string(), None)
        };

        let query = format!(
            r#"
            DELETE FROM settings
            WHERE user_id = ? AND scope = ? AND {} AND {} AND {} AND key = ?
            "#,
            podcast_condition, device_condition, episode_condition
        );

        let mut q = sqlx::query(&query).bind(key.user_id).bind(key.scope);
        if let Some(url) = podcast_bind {
            q = q.bind(url);
        }
        if let Some(id) = device_bind {
            q = q.bind(id);
        }
        if let Some(url) = episode_bind {
            q = q.bind(url);
        }
        q = q.bind(key.key);

        q.execute(&self.pool).await?;

        Ok(())
    }
}
