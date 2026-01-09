use crate::config::Config;
use crate::middleware::AuthorizedContext;
use crate::state::AppState;
use warp::{reply::json, Rejection, Reply};

/// GET /api/2/favorites/{username}.json
/// Get user's favorite episodes
pub async fn get_favorites(
    auth: AuthorizedContext,
    state: AppState,
    config: Config,
) -> Result<impl Reply, Rejection> {
    let favorites = state
        .favorite_service
        .get_user_favorites(auth.user_id, &config.base_url)
        .await
        .map_err(warp::reject::custom)?;

    Ok(json(&favorites))
}
