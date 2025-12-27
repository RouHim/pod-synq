use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FavoriteEpisode {
    pub id: i64,
    pub user_id: i64,
    pub podcast_url: String,
    pub episode_url: String,
    pub title: Option<String>,
    pub podcast_title: Option<String>,
    pub description: Option<String>,
    pub website: Option<String>,
    pub released: Option<String>,
    pub created_at: i64,
}

/// Metadata for adding a favorite episode
#[derive(Debug, Clone, Default)]
pub struct FavoriteMetadata<'a> {
    pub podcast_url: &'a str,
    pub episode_url: &'a str,
    pub title: Option<&'a str>,
    pub podcast_title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub website: Option<&'a str>,
    pub released: Option<&'a str>,
}

/// Response format for favorites API
#[derive(Debug, Serialize)]
pub struct FavoriteResponse {
    pub title: String,
    pub url: String,
    pub podcast_title: String,
    pub podcast_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub released: Option<String>,
    pub mygpo_link: String,
}

impl FavoriteEpisode {
    pub fn to_response(&self, base_url: &str) -> FavoriteResponse {
        FavoriteResponse {
            title: self.title.clone().unwrap_or_default(),
            url: self.episode_url.clone(),
            podcast_title: self.podcast_title.clone().unwrap_or_default(),
            podcast_url: self.podcast_url.clone(),
            description: self.description.clone(),
            website: self.website.clone(),
            released: self.released.clone(),
            mygpo_link: format!("{}/episode/{}", base_url, self.id),
        }
    }
}
