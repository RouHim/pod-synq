use warp::{reject, reply::json, Rejection, Reply};

use crate::constants::{DEFAULT_DEVICE_CAPTION, DEFAULT_DEVICE_TYPE};
use crate::error::AppError;
use crate::middleware::AuthorizedContext;
use crate::models::{DeviceInfo, DeviceUpdatesResponse, UpdateDeviceRequest, UpdatesQueryParams};
use crate::state::AppState;

pub async fn list_devices(
    auth: AuthorizedContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    let devices = state
        .device_service
        .list_user_devices(auth.user_id)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let mut device_infos = Vec::new();

    for device in devices {
        let sub_count = state
            .subscription_service
            .count_subscriptions(auth.user_id, Some(device.id))
            .await?;

        device_infos.push(DeviceInfo {
            id: device.device_id,
            caption: device
                .caption
                .unwrap_or_else(|| DEFAULT_DEVICE_CAPTION.to_string()),
            device_type: device
                .r#type
                .unwrap_or_else(|| DEFAULT_DEVICE_TYPE.to_string()),
            subscriptions: sub_count,
        });
    }

    Ok(json(&device_infos))
}

pub async fn update_device(
    device_id: String,
    auth: AuthorizedContext,
    state: AppState,
    req: UpdateDeviceRequest,
) -> Result<impl Reply, Rejection> {
    let db_device_id = state
        .device_service
        .get_or_create_device(
            auth.user_id,
            &device_id,
            req.caption.as_deref(),
            req.device_type.as_deref(),
        )
        .await
        .map_err(warp::reject::custom)?;

    tracing::info!(
        "Device {} (ID: {}) updated for user {}",
        device_id,
        db_device_id,
        auth.username
    );

    Ok(json(&serde_json::json!({
        "status": "ok",
    })))
}

pub async fn get_device_updates(
    device_id: String,
    params: UpdatesQueryParams,
    auth: AuthorizedContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    let db_device_id = state
        .device_service
        .find_by_device_id(auth.user_id, &device_id)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let since = params.since.unwrap_or(0);

    // Get subscription changes (URLs)
    let (add_urls, remove) = state
        .subscription_service
        .get_changes_since(auth.user_id, db_device_id.id, since)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    // Enrich add_urls with metadata
    let add = state
        .podcast_service
        .get_metadata_for_urls(&add_urls)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let updates = if params.include_actions.unwrap_or(false) {
        let actions = state
            .episode_action_service
            .get_actions_since(auth.user_id, Some(db_device_id.id), None, since)
            .await
            .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

        actions
            .into_iter()
            .map(|action| {
                serde_json::json!({
                    "podcast": action.podcast_url,
                    "url": action.episode_url,
                    "device": action.device,
                    "action": action.action,
                    "timestamp": action.timestamp,
                    "started": action.started,
                    "position": action.position,
                    "total": action.total,
                })
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(json(&DeviceUpdatesResponse {
        add,
        remove,
        updates,
        timestamp: chrono::Utc::now().timestamp(),
    }))
}
