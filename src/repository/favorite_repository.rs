use crate::models::{FavoriteEpisode, FavoriteMetadata};
use sqlx::{Row, SqlitePool};

#[derive(Clone)]
pub struct FavoriteRepository {
    pool: SqlitePool,
}

impl FavoriteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Add an episode to favorites
    pub async fn add_favorite(
        &self,
        user_id: i64,
        metadata: &FavoriteMetadata<'_>,
    ) -> Result<i64, sqlx::Error> {
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

    /// Remove an episode from favorites
    pub async fn remove_favorite(
        &self,
        user_id: i64,
        episode_url: &str,
    ) -> Result<(), sqlx::Error> {
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

    /// Get all favorites for a user
    pub async fn get_user_favorites(
        &self,
        user_id: i64,
    ) -> Result<Vec<FavoriteEpisode>, sqlx::Error> {
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

    /// Check if an episode is favorited
    #[allow(dead_code)]
    pub async fn is_favorite(&self, user_id: i64, episode_url: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM favorite_episodes
            WHERE user_id = ? AND episode_url = ?
            "#,
        )
        .bind(user_id)
        .bind(episode_url)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = result.get("count");
        Ok(count > 0)
    }
}
