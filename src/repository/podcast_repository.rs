use crate::error::AppResult;
use crate::models::Podcast;
use sqlx::{Row, SqlitePool};

pub struct PodcastRepository {
    pool: SqlitePool,
}

impl PodcastRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get podcast by URL
    #[allow(dead_code)]
    pub async fn get_by_url(&self, url: &str) -> AppResult<Option<Podcast>> {
        let result = sqlx::query(
            r#"
            SELECT id, url, title, description, website, logo_url,
                   subscriber_count, created_at, updated_at
            FROM podcasts
            WHERE url = ?
            "#,
        )
        .bind(url)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| Podcast {
            id: row.get_unchecked(0),
            url: row.get_unchecked::<&str, _>(1).to_string(),
            title: row
                .get_unchecked::<Option<&str>, _>(2)
                .map(|s| s.to_string()),
            description: row
                .get_unchecked::<Option<&str>, _>(3)
                .map(|s| s.to_string()),
            website: row
                .get_unchecked::<Option<&str>, _>(4)
                .map(|s| s.to_string()),
            logo_url: row
                .get_unchecked::<Option<&str>, _>(5)
                .map(|s| s.to_string()),
            subscriber_count: row.get_unchecked(6),
            created_at: row.get_unchecked(7),
            updated_at: row.get_unchecked(8),
        }))
    }

    /// Get multiple podcasts by URLs
    pub async fn get_by_urls(&self, urls: &[String]) -> AppResult<Vec<Podcast>> {
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

        let mut query_builder = sqlx::query(&query);
        for url in urls {
            query_builder = query_builder.bind(url);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let podcasts = rows
            .into_iter()
            .map(|row| Podcast {
                id: row.get_unchecked(0),
                url: row.get_unchecked::<&str, _>(1).to_string(),
                title: row
                    .get_unchecked::<Option<&str>, _>(2)
                    .map(|s| s.to_string()),
                description: row
                    .get_unchecked::<Option<&str>, _>(3)
                    .map(|s| s.to_string()),
                website: row
                    .get_unchecked::<Option<&str>, _>(4)
                    .map(|s| s.to_string()),
                logo_url: row
                    .get_unchecked::<Option<&str>, _>(5)
                    .map(|s| s.to_string()),
                subscriber_count: row.get_unchecked(6),
                created_at: row.get_unchecked(7),
                updated_at: row.get_unchecked(8),
            })
            .collect();

        Ok(podcasts)
    }

    /// Create or update podcast metadata
    #[allow(dead_code)]
    pub async fn upsert(
        &self,
        url: &str,
        title: Option<&str>,
        description: Option<&str>,
        website: Option<&str>,
        logo_url: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO podcasts (url, title, description, website, logo_url, updated_at)
            VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'))
            ON CONFLICT(url) DO UPDATE SET
                title = COALESCE(excluded.title, title),
                description = COALESCE(excluded.description, description),
                website = COALESCE(excluded.website, website),
                logo_url = COALESCE(excluded.logo_url, logo_url),
                updated_at = strftime('%s', 'now')
            "#,
        )
        .bind(url)
        .bind(title)
        .bind(description)
        .bind(website)
        .bind(logo_url)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
