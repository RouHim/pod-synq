use crate::config::Config;
use crate::middleware::AuthContext;
use crate::state::AppState;
use warp::{reject, reply::json, Rejection, Reply};

/// GET /api/2/favorites/{username}.json
/// Get user's favorite episodes
pub async fn get_favorites(
    username: String,
    auth: AuthContext,
    state: AppState,
    config: Config,
) -> Result<impl Reply, Rejection> {
    // Verify the authenticated user matches the requested username
    if auth.username != username {
        return Err(reject::custom(crate::error::AppError::Authentication));
    }

    let favorites = state
        .favorite_service
        .get_user_favorites(auth.user_id, &config.base_url)
        .await
        .map_err(reject::custom)?;

    Ok(json(&favorites))
}
