use serde::{Deserialize, Serialize};
use warp::{reject, reply::json, Rejection, Reply};

use crate::error::AppError;
use crate::middleware::AuthContext;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub caption: String,
    #[serde(rename = "type")]
    pub device_type: String,
    pub subscriptions: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub caption: Option<String>,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
}

pub async fn list_devices(
    username: String,
    auth: AuthContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

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
                .unwrap_or_else(|| "Unknown Device".to_string()),
            device_type: device.r#type.unwrap_or_else(|| "unknown".to_string()),
            subscriptions: sub_count,
        });
    }

    Ok(json(&device_infos))
}

pub async fn update_device(
    username: String,
    device_id: String,
    auth: AuthContext,
    state: AppState,
    req: UpdateDeviceRequest,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(warp::reject::custom(crate::error::AppError::Authorization));
    }

    let db_device_id = state
        .device_service
        .get_or_create_device(
            auth.user_id,
            &device_id,
            req.caption.as_deref(),
            req.device_type.as_deref(),
        )
        .await
        .map_err(|e| warp::reject::custom(e.into()))?;

    tracing::info!(
        "Device {} (ID: {}) updated for user {}",
        device_id,
        db_device_id,
        username
    );

    Ok(json(&serde_json::json!({
        "status": "ok",
    })))
}
