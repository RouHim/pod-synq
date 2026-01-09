use async_trait::async_trait;
use sqlx::{Row, SqlitePool};

use crate::error::AppResult;
use crate::models::Podcast;

use super::traits::PodcastRepositoryTrait;

pub struct PodcastRepository {
    pool: SqlitePool,
}

impl PodcastRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PodcastRepositoryTrait for PodcastRepository {
    async fn get_by_urls(&self, urls: &[String]) -> AppResult<Vec<Podcast>> {
        if urls.is_empty() {
            return Ok(Vec::new());
        }

        // Build IN clause with placeholders
        let placeholders = urls.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            SELECT id, url, title, description, website, logo_url,
                   subscriber_count, created_at, updated_at
            FROM podcasts
            WHERE url IN ({})
            "#,
            placeholders
        );

        // Dynamic query with variable bindings - must use manual binding
        let mut query_builder = sqlx::query(&query);
        for url in urls {
            query_builder = query_builder.bind(url);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        // Map rows to Podcast - can't use query_as with dynamic IN clause
        let podcasts = rows
            .into_iter()
            .map(|row| Podcast {
                id: row.get("id"),
                url: row.get("url"),
                title: row.get("title"),
                description: row.get("description"),
                website: row.get("website"),
                logo_url: row.get("logo_url"),
                subscriber_count: row.get("subscriber_count"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            })
            .collect();

        Ok(podcasts)
    }
}
