use serde::Deserialize;
use warp::{
    http::header::{HeaderValue, SET_COOKIE},
    reply::{json, with_header, Reply},
    Rejection,
};

use crate::{middleware::AuthContext, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    #[serde(default)]
    _session_id: Option<String>,
}

pub async fn login(
    _username: String,
    auth: AuthContext,
    state: AppState,
) -> Result<impl Reply, Rejection> {
    tracing::info!("Login handler called for user: {}", auth.username);

    // Create session
    let session_id = state
        .session_service
        .create_session(auth.user_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create session: {:?}", e);
            warp::reject::custom(e)
        })?;

    let response = json(&serde_json::json!({
        "status": "ok",
    }));

    // Set session cookie (30 days, HttpOnly, SameSite=Lax)
    let cookie = format!(
        "sessionid={}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax",
        session_id,
        30 * 24 * 60 * 60 // 30 days in seconds
    );

    Ok(with_header(
        response,
        SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap(),
    ))
}

pub async fn logout(
    _username: String,
    _auth: AuthContext,
    state: AppState,
    cookie_header: Option<String>,
    _req: LogoutRequest,
) -> Result<impl Reply, Rejection> {
    tracing::info!("User logged out");

    // Try to extract and delete session
    if let Some(cookie) = cookie_header {
        if let Some(session_id) = extract_session_from_cookie(&cookie) {
            if let Err(e) = state.session_service.delete_session(&session_id).await {
                tracing::warn!("Failed to delete session: {:?}", e);
            }
        }
    }

    let response = json(&serde_json::json!({
        "status": "ok",
    }));

    // Clear session cookie
    let cookie = "sessionid=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax";

    Ok(with_header(
        response,
        SET_COOKIE,
        HeaderValue::from_str(cookie).unwrap(),
    ))
}

fn extract_session_from_cookie(cookie_header: &str) -> Option<String> {
    for cookie in cookie_header.split(';') {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == "sessionid" {
            return Some(parts[1].to_string());
        }
    }
    None
}
