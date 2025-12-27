use sqlx::{Error, Row, SqlitePool};

#[derive(Clone)]
pub struct SubscriptionRepository {
    pool: SqlitePool,
}

impl SubscriptionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_by_device(
        &self,
        user_id: i64,
        device_id: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
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

        Ok(rows
            .into_iter()
            .map(|row| row.get_unchecked::<i64, _>(0).to_string())
            .collect())
    }

    pub async fn list_all_urls_by_user(&self, user_id: i64) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query(
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

        Ok(rows
            .into_iter()
            .map(|row| row.get_unchecked::<i64, _>(0).to_string())
            .collect())
    }

    pub async fn get_changes_since(
        &self,
        user_id: i64,
        device_id: i64,
        since: i64,
    ) -> Result<(Vec<String>, Vec<String>), sqlx::Error> {
        let added = sqlx::query(
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

        let removed = sqlx::query(
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

        let added_urls = added
            .into_iter()
            .map(|row| row.get_unchecked::<i64, _>(0).to_string())
            .collect();

        let removed_urls = removed
            .into_iter()
            .map(|row| row.get_unchecked::<i64, _>(0).to_string())
            .collect();

        Ok((added_urls, removed_urls))
    }

    pub async fn set_subscriptions(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_urls: Vec<String>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let current = sqlx::query(
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

        let current_urls: std::collections::HashSet<String> = current
            .into_iter()
            .map(|row| row.get_unchecked::<i64, _>(0).to_string())
            .collect();

        let new_urls: std::collections::HashSet<String> = podcast_urls.into_iter().collect();

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

    pub async fn apply_changes(
        &self,
        user_id: i64,
        device_id: i64,
        changes: crate::models::SubscriptionChanges,
    ) -> Result<(), sqlx::Error> {
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

    pub async fn count(&self, user_id: i64, device_id: Option<i64>) -> Result<i64, Error> {
        if let Some(device_id) = device_id {
            let result = sqlx::query(
                r#"
                SELECT COUNT(*) as count
                FROM subscriptions
                WHERE user_id = ? AND device_id = ? AND removed_at IS NULL
                "#,
            )
            .bind(user_id)
            .bind(device_id)
            .fetch_one(&self.pool)
            .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        } else {
            let result = sqlx::query(
                r#"
                SELECT COUNT(*) as count
                FROM subscriptions
                WHERE user_id = ? AND removed_at IS NULL
                "#,
            )
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        }
    }
}
