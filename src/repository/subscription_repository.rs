use async_trait::async_trait;
use sqlx::SqlitePool;
use std::collections::HashSet;

use crate::error::AppResult;
use crate::models::SubscriptionChanges;

use super::traits::SubscriptionRepositoryTrait;

#[derive(Clone)]
pub struct SubscriptionRepository {
    pool: SqlitePool,
}

impl SubscriptionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubscriptionRepositoryTrait for SubscriptionRepository {
    async fn list_by_device(&self, user_id: i64, device_id: i64) -> AppResult<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT podcast_url
            FROM subscriptions
            WHERE user_id = ? AND device_id = ? AND removed_at IS NULL
            ORDER BY added_at ASC
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(url,)| url).collect())
    }

    async fn list_all_urls_by_user(&self, user_id: i64) -> AppResult<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT podcast_url
            FROM subscriptions
            WHERE user_id = ? AND removed_at IS NULL
            ORDER BY podcast_url ASC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(url,)| url).collect())
    }

    async fn get_changes_since(
        &self,
        user_id: i64,
        device_id: i64,
        since: i64,
    ) -> AppResult<(Vec<String>, Vec<String>)> {
        let added: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT podcast_url
            FROM subscriptions
            WHERE user_id = ? AND device_id = ? AND added_at > ? AND removed_at IS NULL
            ORDER BY added_at ASC
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        let removed: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT podcast_url
            FROM subscriptions
            WHERE user_id = ? AND device_id = ? AND removed_at > ? AND removed_at IS NOT NULL
            ORDER BY removed_at ASC
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .bind(since)
        .fetch_all(&self.pool)
        .await?;

        let added_urls = added.into_iter().map(|(url,)| url).collect();
        let removed_urls = removed.into_iter().map(|(url,)| url).collect();

        Ok((added_urls, removed_urls))
    }

    async fn set_subscriptions(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_urls: Vec<String>,
    ) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;

        let current: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT podcast_url
            FROM subscriptions
            WHERE user_id = ? AND device_id = ? AND removed_at IS NULL
            "#,
        )
        .bind(user_id)
        .bind(device_id)
        .fetch_all(&mut *tx)
        .await?;

        let current_urls: HashSet<String> = current.into_iter().map(|(url,)| url).collect();
        let new_urls: HashSet<String> = podcast_urls.into_iter().collect();

        for url in &new_urls {
            if !current_urls.contains(url) {
                sqlx::query(
                    r#"
                    INSERT OR IGNORE INTO subscriptions (user_id, device_id, podcast_url)
                    VALUES (?, ?, ?)
                    "#,
                )
                .bind(user_id)
                .bind(device_id)
                .bind(url)
                .execute(&mut *tx)
                .await?;
            }
        }

        for url in &current_urls {
            if !new_urls.contains(url) {
                sqlx::query(
                    r#"
                    UPDATE subscriptions
                    SET removed_at = strftime('%s', 'now')
                    WHERE user_id = ? AND device_id = ? AND podcast_url = ? AND removed_at IS NULL
                    "#,
                )
                .bind(user_id)
                .bind(device_id)
                .bind(url)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn apply_changes(
        &self,
        user_id: i64,
        device_id: i64,
        changes: SubscriptionChanges,
    ) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;

        for podcast_url in changes.add {
            sqlx::query(
                r#"
                INSERT OR IGNORE INTO subscriptions (user_id, device_id, podcast_url)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(user_id)
            .bind(device_id)
            .bind(podcast_url)
            .execute(&mut *tx)
            .await?;
        }

        for podcast_url in changes.remove {
            sqlx::query(
                r#"
                UPDATE subscriptions
                SET removed_at = strftime('%s', 'now')
                WHERE user_id = ? AND device_id = ? AND podcast_url = ? AND removed_at IS NULL
                "#,
            )
            .bind(user_id)
            .bind(device_id)
            .bind(podcast_url)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn count(&self, user_id: i64, device_id: Option<i64>) -> AppResult<i64> {
        let count: (i64,) = if let Some(device_id) = device_id {
            sqlx::query_as(
                r#"
                SELECT COUNT(*)
                FROM subscriptions
                WHERE user_id = ? AND device_id = ? AND removed_at IS NULL
                "#,
            )
            .bind(user_id)
            .bind(device_id)
            .fetch_one(&self.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
                SELECT COUNT(*)
                FROM subscriptions
                WHERE user_id = ? AND removed_at IS NULL
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?
        };

        Ok(count.0)
    }
}
