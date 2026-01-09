use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::error::AppResult;
use crate::models::{FavoriteEpisode, FavoriteMetadata};

use super::traits::FavoriteRepositoryTrait;

#[derive(Clone)]
pub struct FavoriteRepository {
    pool: SqlitePool,
}

impl FavoriteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FavoriteRepositoryTrait for FavoriteRepository {
    async fn add_favorite(&self, user_id: i64, metadata: &FavoriteMetadata<'_>) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO favorite_episodes (user_id, podcast_url, episode_url, title, podcast_title, description, website, released)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, episode_url) DO UPDATE SET
                podcast_url = excluded.podcast_url,
                title = excluded.title,
                podcast_title = excluded.podcast_title,
                description = excluded.description,
                website = excluded.website,
                released = excluded.released
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(metadata.podcast_url)
        .bind(metadata.episode_url)
        .bind(metadata.title)
        .bind(metadata.podcast_title)
        .bind(metadata.description)
        .bind(metadata.website)
        .bind(metadata.released)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get(0))
    }

    async fn remove_favorite(&self, user_id: i64, episode_url: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM favorite_episodes
            WHERE user_id = ? AND episode_url = ?
            "#,
        )
        .bind(user_id)
        .bind(episode_url)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_user_favorites(&self, user_id: i64) -> AppResult<Vec<FavoriteEpisode>> {
        let favorites = sqlx::query_as::<_, FavoriteEpisode>(
            r#"
            SELECT id, user_id, podcast_url, episode_url, title, podcast_title, description, website, released, created_at
            FROM favorite_episodes
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(favorites)
    }
}
