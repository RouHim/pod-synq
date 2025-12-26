use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EpisodeAction {
    pub id: i64,
    pub user_id: i64,
    pub device_id: i64,
    pub podcast_url: String,
    pub episode_url: String,
    pub action: String,
    pub timestamp: i64,
    pub started: Option<i64>,
    pub position: Option<i64>,
    pub total: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpisodeAction {
    pub user_id: i64,
    pub device_id: i64,
    pub podcast_url: String,
    pub episode_url: String,
    pub action: String,
    pub timestamp: i64,
    pub started: Option<i64>,
    pub position: Option<i64>,
    pub total: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeActionQuery {
    pub since: Option<i64>,
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub aggregated: Option<bool>,
}
