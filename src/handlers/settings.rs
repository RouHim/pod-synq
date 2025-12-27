use serde::Deserialize;
use warp::{reject, reply::json, Rejection, Reply};

use crate::error::AppError;
use crate::middleware::AuthContext;
use crate::models::SettingRequest;
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
