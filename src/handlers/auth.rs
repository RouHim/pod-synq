use warp::{
    http::header::{HeaderValue, SET_COOKIE},
    reply::{json, with_header, Reply},
    Rejection,
};

use crate::{
    constants::{SESSION_COOKIE_NAME, SESSION_DURATION_SECS},
    middleware::AuthContext,
    models::LogoutRequest,
    state::AppState,
};

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

    // Set session cookie (HttpOnly, SameSite=Lax)
    let cookie = format!(
        "{}={}; Max-Age={}; Path=/; HttpOnly; SameSite=Lax",
        SESSION_COOKIE_NAME, session_id, SESSION_DURATION_SECS
    );

    Ok(with_header(
        response,
        SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("cookie contains only valid header characters"),
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
    let cookie = format!(
        "{}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax",
        SESSION_COOKIE_NAME
    );

    Ok(with_header(
        response,
        SET_COOKIE,
        HeaderValue::from_str(&cookie).expect("valid cookie header"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_session_cookie_yields_value() {
        assert_eq!(
            extract_session_from_cookie("sessionid=abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn session_cookie_among_multiple_cookies_is_found() {
        assert_eq!(
            extract_session_from_cookie("other=x; sessionid=abc123"),
            Some("abc123".to_string())
        );
    }

    #[test]
    fn cookies_without_sessionid_yield_none() {
        assert_eq!(extract_session_from_cookie("foo=bar"), None);
    }

    #[test]
    fn empty_cookie_header_yields_none() {
        assert_eq!(extract_session_from_cookie(""), None);
    }

    #[test]
    fn value_containing_equals_sign_is_preserved_in_full() {
        assert_eq!(
            extract_session_from_cookie("sessionid=a=b"),
            Some("a=b".to_string())
        );
    }

    #[test]
    fn surrounding_whitespace_on_pair_is_trimmed() {
        assert_eq!(
            extract_session_from_cookie(" sessionid=abc "),
            Some("abc".to_string())
        );
    }
}
