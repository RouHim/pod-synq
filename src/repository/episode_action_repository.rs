use crate::models::{EpisodeAction, EpisodeActionQuery};
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

#[derive(Clone)]
pub struct EpisodeActionRepository {
    pool: SqlitePool,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct EpisodeActionWithDevice {
    pub id: i64,
    pub user_id: i64,
    #[sqlx(rename = "device_id_fk")]
    pub device_id_fk: i64,
    pub device: String,
    pub podcast_url: String,
    pub episode_url: String,
    pub action: String,
    pub timestamp: i64,
    pub started: Option<i64>,
    pub position: Option<i64>,
    pub total: Option<i64>,
    pub created_at: i64,
}

impl EpisodeActionRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        user_id: i64,
        query: EpisodeActionQuery,
    ) -> Result<Vec<EpisodeActionWithDevice>, sqlx::Error> {
        let mut sql = String::from(
            r#"
            SELECT
                ea.id, ea.user_id, ea.device_id as device_id_fk, d.device_id as device,
                ea.podcast_url, ea.episode_url, ea.action,
                ea.timestamp, ea.started, ea.position, ea.total, ea.created_at
            FROM episode_actions ea
            INNER JOIN devices d ON ea.device_id = d.id
            WHERE ea.user_id = ?
            "#,
        );

        if query.since.is_some() {
            sql.push_str(" AND ea.timestamp >= ? ");
        }

        if query.podcast.is_some() {
            sql.push_str(" AND ea.podcast_url = ? ");
        }

        if query.device.is_some() {
            sql.push_str(" AND d.device_id = ? ");
        }

        sql.push_str(" ORDER BY ea.timestamp DESC");

        let mut q = sqlx::query_as::<_, EpisodeActionWithDevice>(&sql);
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
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
