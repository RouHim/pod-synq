use crate::middleware::AuthorizedContext;
use crate::models::DeviceSyncRequest;
use crate::state::AppState;
use warp::{reply::json, Rejection, Reply};

/// GET /api/2/sync-devices/{username}.json
/// Get current device synchronization status
pub async fn get_sync_status(
    auth: AuthorizedContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    let status = state
        .device_sync_service
        .get_sync_status(auth.user_id)
        .await
        .map_err(warp::reject::custom)?;

    Ok(json(&status))
}

/// POST /api/2/sync-devices/{username}.json
/// Update device synchronization groups
pub async fn update_sync_groups(
    auth: AuthorizedContext,
    state: AppState,
    request: DeviceSyncRequest,
) -> Result<impl Reply, Rejection> {
    let status = state
        .device_sync_service
        .update_sync_groups(auth.user_id, request.synchronize, request.stop_synchronize)
        .await
        .map_err(warp::reject::custom)?;

    Ok(json(&status))
}
