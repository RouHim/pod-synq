use serde::{Deserialize, Serialize};
use warp::{reply::json, Rejection, Reply};

use crate::middleware::AuthContext;

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    pub session_id: String,
}

pub async fn login(_username: String, _auth: AuthContext) -> Result<impl Reply, Rejection> {
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
