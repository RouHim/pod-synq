use crate::models::{EpisodeAction, EpisodeActionQuery};
use sqlx::SqlitePool;

#[derive(Clone)]
pub struct EpisodeActionRepository {
    pool: SqlitePool,
}

impl EpisodeActionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        user_id: i64,
        query: EpisodeActionQuery,
    ) -> Result<Vec<EpisodeAction>, sqlx::Error> {
        let mut sql = String::from(
            r#"
            SELECT 
                id, user_id, device_id, podcast_url, episode_url, action, 
                timestamp, started, position, total, created_at
            FROM episode_actions 
            WHERE user_id = ?
            "#,
        );

        let mut bind_count = 1;

        if let Some(ref _since) = query.since {
            bind_count += 1;
            sql.push_str(&format!("AND timestamp >= ${}", bind_count));
        }

        if let Some(ref _podcast) = query.podcast {
            bind_count += 1;
            sql.push_str(&format!("AND podcast_url = ${}", bind_count));
        }

        if let Some(ref _device) = query.device {
            bind_count += 1;
            sql.push_str(&format!("AND device_id = ${}", bind_count));
        }

        sql.push_str("ORDER BY timestamp DESC");

        let mut q = sqlx::query_as::<_, EpisodeAction>(&sql);
        q = q.bind(user_id);

        if let Some(since) = query.since {
            q = q.bind(since);
        }

        if let Some(ref podcast) = query.podcast {
            q = q.bind(podcast);
        }

        if let Some(ref device) = query.device {
            q = q.bind(device);
        }

        q.fetch_all(&self.pool).await
    }

    pub async fn upload(&self, actions: Vec<EpisodeAction>) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        for action in actions {
            sqlx::query(
                r#"
                INSERT INTO episode_actions 
                (user_id, device_id, podcast_url, episode_url, action, timestamp, started, position, total)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(action.user_id)
            .bind(action.device_id)
            .bind(&action.podcast_url)
            .bind(&action.episode_url)
            .bind(&action.action)
            .bind(action.timestamp)
            .bind(action.started)
            .bind(action.position)
            .bind(action.total)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}
