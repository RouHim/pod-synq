use crate::middleware::AuthContext;
use crate::models::DeviceSyncRequest;
use crate::state::AppState;
use warp::{reject, reply::json, Rejection, Reply};

/// GET /api/2/sync-devices/{username}.json
/// Get current device synchronization status
pub async fn get_sync_status(
    username: String,
    auth: AuthContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    // Verify the authenticated user matches the requested username
    if auth.username != username {
        return Err(reject::custom(crate::error::AppError::Authentication));
    }

    let status = state
        .device_sync_service
        .get_sync_status(auth.user_id)
        .await
        .map_err(reject::custom)?;

    Ok(json(&status))
}

/// POST /api/2/sync-devices/{username}.json
/// Update device synchronization groups
pub async fn update_sync_groups(
    username: String,
    auth: AuthContext,
    state: AppState,
    request: DeviceSyncRequest,
) -> Result<impl Reply, Rejection> {
    // Verify the authenticated user matches the requested username
    if auth.username != username {
        return Err(reject::custom(crate::error::AppError::Authentication));
    }

    let status = state
        .device_sync_service
        .update_sync_groups(auth.user_id, request.synchronize, request.stop_synchronize)
        .await
        .map_err(reject::custom)?;

    Ok(json(&status))
}
