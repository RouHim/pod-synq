use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Podcast {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub logo_url: Option<String>,
    pub subscriber_count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PodcastMetadata {
    pub url: String,
    pub title: String,
    pub description: String,
    pub subscribers: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    pub mygpo_link: String,
}

impl Podcast {
    /// Convert to API response format with fallback values
    pub fn to_metadata(&self, base_url: &str) -> PodcastMetadata {
        PodcastMetadata {
            url: self.url.clone(),
            title: self.title.clone().unwrap_or_else(|| {
                // Fallback: use URL as title
                self.url
                    .split('/')
                    .next_back()
                    .unwrap_or("Unknown Podcast")
                    .to_string()
            }),
            description: self.description.clone().unwrap_or_default(),
            subscribers: self.subscriber_count,
            logo_url: self.logo_url.clone(),
            website: self.website.clone(),
            mygpo_link: format!("{}/podcast/{}", base_url, urlencoding::encode(&self.url)),
        }
    }
}
