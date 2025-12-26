use crate::models::{EpisodeAction, EpisodeActionQuery};
use sqlx::{Error, Row, SqlitePool};

#[derive(Clone)]
pub struct EpisodeActionRepository {
    pool: SqlitePool,
}

impl EpisodeActionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: i64,
        device_id: i64,
        podcast_url: &str,
        episode_url: &str,
        action: &str,
        timestamp: i64,
        started: Option<i64>,
        position: Option<i64>,
        total: Option<i64>,
    ) -> Result<EpisodeAction, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO episode_actions 
            (user_id, device_id, podcast_url, episode_url, action, timestamp, started, position, total)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING 
                id, user_id, device_id, podcast_url, episode_url, action, 
                timestamp, started, position, total, created_at
            "#
        )
        .bind(user_id)
        .bind(device_id)
        .bind(podcast_url)
        .bind(episode_url)
        .bind(action)
        .bind(timestamp)
        .bind(started)
        .bind(position)
        .bind(total)
        .fetch_one(&self.pool)
        .await?;

        Ok(EpisodeAction {
            id: result.get_unchecked::<i64, _>(0),
            user_id: result.get_unchecked::<i64, _>(1),
            device_id: result.get_unchecked::<i64, _>(2),
            podcast_url: result.get_unchecked::<&str, _>(3).to_string(),
            episode_url: result.get_unchecked::<&str, _>(4).to_string(),
            action: result.get_unchecked::<&str, _>(5).to_string(),
            timestamp: result.get_unchecked::<i64, _>(6),
            started: result.get_unchecked::<Option<i64>, _>(7),
            position: result.get_unchecked::<Option<i64>, _>(8),
            total: result.get_unchecked::<Option<i64>, _>(9),
            created_at: result.get_unchecked::<i64, _>(10),
        })
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

    pub async fn count(&self, user_id: Option<i64>) -> Result<i64, Error> {
        if let Some(user_id) = user_id {
            let result =
                sqlx::query("SELECT COUNT(*) as count FROM episode_actions WHERE user_id = ?")
                    .bind(user_id)
                    .fetch_one(&self.pool)
                    .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        } else {
            let result = sqlx::query("SELECT COUNT(*) as count FROM episode_actions")
                .fetch_one(&self.pool)
                .await?;
            Ok(result.get_unchecked::<i64, _>(0))
        }
    }
}
