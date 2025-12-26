use serde::{Deserialize, Serialize};
use warp::{reject, reply::json, Rejection, Reply};

use crate::error::AppError;
use crate::middleware::AuthContext;
use crate::models::SubscriptionChanges;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SubscriptionListResponse {
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize)]
pub struct SubscriptionUploadRequest {
    pub add: Option<Vec<String>>,
    pub remove: Option<Vec<String>>,
    pub timestamp: Option<i64>,
}

pub async fn get_subscriptions(
    username: String,
    device_id: String,
    auth: AuthContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let subscriptions = state
        .subscription_service
        .get_subscriptions(auth.user_id, db_device_id)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(json(&subscriptions))
}

pub async fn upload_subscriptions(
    username: String,
    device_id: String,
    auth: AuthContext,
    state: AppState,
    req: SubscriptionUploadRequest,
) -> Result<impl Reply, Rejection> {
    if username != auth.username {
        return Err(reject::custom(AppError::Authorization));
    }

    let db_device_id = state
        .device_service
        .get_or_create_device(auth.user_id, &device_id, None, None)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    let changes = SubscriptionChanges {
        add: req.add.as_deref().unwrap_or_default().to_vec(),
        remove: req.remove.as_deref().unwrap_or_default().to_vec(),
        timestamp: req
            .timestamp
            .unwrap_or_else(|| chrono::Utc::now().timestamp()),
    };

    state
        .subscription_service
        .upload_changes(auth.user_id, db_device_id, changes)
        .await
        .map_err(|e| reject::custom(AppError::Internal(e.to_string())))?;

    Ok(json(&SubscriptionListResponse {
        add: vec![],
        remove: vec![],
        timestamp: chrono::Utc::now().timestamp(),
    }))
}
