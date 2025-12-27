use serde::Deserialize;
use warp::{reply::json, Rejection, Reply};

use crate::middleware::AuthContext;

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    _session_id: Option<String>,
}

pub async fn login(_username: String, _auth: AuthContext) -> Result<impl Reply, Rejection> {
    tracing::info!("Login handler called for user: {}", _username);
    tracing::info!(
        "AuthContext - user_id: {}, username: {}",
        _auth.user_id,
        _auth.username
    );
    tracing::info!("User logged in");

    Ok(json(&serde_json::json!({
        "status": "ok",
    })))
}

pub async fn logout(
    _username: String,
    _auth: AuthContext,
    _req: LogoutRequest,
) -> Result<impl Reply, Rejection> {
    tracing::info!("User logged out");

    Ok(json(&serde_json::json!({
        "status": "ok",
    })))
}
