use serde::Deserialize;
use warp::{reject, reply::json, Rejection, Reply};

use crate::error::AppError;
use crate::middleware::AuthContext;
use crate::models::{FavoriteMetadata, SettingRequest};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SettingsQueryParams {
    pub podcast: Option<String>,
    pub device: Option<String>,
    pub episode: Option<String>,
}

pub async fn get_settings(
    username: String,
    scope: String,
    params: SettingsQueryParams,
    auth: AuthContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

    let device_id = if let Some(device_str) = params.device {
        let device = state
            .device_service
            .find_by_device_id(auth.user_id, &device_str)
            .await
            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
        Some(device.id)
    } else {
        None
    };

    let settings = state
        .setting_service
        .get_settings(
            auth.user_id,
            &scope,
            params.podcast.as_deref(),
            device_id,
            params.episode.as_deref(),
        )
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(json(&settings))
}

pub async fn save_settings(
    username: String,
    scope: String,
    params: SettingsQueryParams,
    auth: AuthContext,
    state: AppState,
    req: SettingRequest,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

    let device_id = if let Some(device_str) = params.device {
        let device = state
            .device_service
            .find_by_device_id(auth.user_id, &device_str)
            .await
            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
        Some(device.id)
    } else {
        None
    };

    // Handle favorites integration for episode scope
    if scope == "episode" {
        if let Some(ref episode_url) = params.episode {
            // Check if is_favorite is being set
            if let Some(ref set_map) = req.set {
                if let Some(is_favorite_value) = set_map.get("is_favorite") {
                    let is_favorite = is_favorite_value.as_bool().unwrap_or(false);

                    if is_favorite {
                        // Extract metadata from the request
                        let metadata = FavoriteMetadata {
                            podcast_url: params.podcast.as_deref().unwrap_or(""),
                            episode_url,
                            title: set_map.get("title").and_then(|v| v.as_str()),
                            podcast_title: set_map.get("podcast_title").and_then(|v| v.as_str()),
                            description: set_map.get("description").and_then(|v| v.as_str()),
                            website: set_map.get("website").and_then(|v| v.as_str()),
                            released: set_map.get("released").and_then(|v| v.as_str()),
                        };

                        // Add to favorites
                        state
                            .favorite_service
                            .add_favorite(auth.user_id, &metadata)
                            .await
                            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
                    } else {
                        // Remove from favorites
                        state
                            .favorite_service
                            .remove_favorite(auth.user_id, episode_url)
                            .await
                            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
                    }
                }
            }

            // Check if is_favorite is being removed
            if let Some(ref remove_list) = req.remove {
                if remove_list.contains(&"is_favorite".to_string()) {
                    // Remove from favorites
                    state
                        .favorite_service
                        .remove_favorite(auth.user_id, episode_url)
                        .await
                        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;
                }
            }
        }
    }

    let settings = state
        .setting_service
        .save_settings(
            auth.user_id,
            &scope,
            params.podcast.as_deref(),
            device_id,
            params.episode.as_deref(),
            req,
        )
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(json(&settings))
}
