use serde::Deserialize;
use std::collections::HashMap;
use warp::{reject, reply::json, Rejection, Reply};

use crate::error::AppError;
use crate::middleware::AuthContext;
use crate::models::{EpisodeAction, EpisodeActionQuery};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct EpisodeActionQueryParams {
    pub since: Option<i64>,
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub aggregated: Option<bool>,
}

pub async fn get_episode_actions(
    username: String,
    auth: AuthContext,
    params: EpisodeActionQueryParams,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

    let query = EpisodeActionQuery {
        since: params.since,
        podcast: params.podcast.clone(),
        device: params.device.clone(),
        aggregated: params.aggregated,
    };

    let actions = state
        .episode_action_service
        .get_episode_actions(auth.user_id, query)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let mut result: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

    for action in actions {
        let podcast_actions = result.entry(action.podcast_url.clone()).or_default();

        podcast_actions.push(serde_json::json!({
            "podcast": action.podcast_url,
            "episode": action.episode_url,
            "action": action.action,
            "timestamp": action.timestamp,
            "started": action.started,
            "position": action.position,
            "total": action.total,
            "device": "",
        }));
    }

    Ok(json(&serde_json::json!(result)))
}

pub async fn upload_episode_actions(
    username: String,
    auth: AuthContext,
    state: AppState,
    actions: Vec<EpisodeAction>,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(warp::reject::custom(crate::error::AppError::Authorization));
    }

    state
        .episode_action_service
        .upload_episode_actions(actions)
        .await
        .map_err(warp::reject::custom)?;

    Ok(json(&serde_json::json!({
        "update_urls": [],
    })))
}
