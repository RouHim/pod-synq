use base64::Engine;
use std::sync::Arc;
use warp::{Filter, Rejection};

use crate::error::{AppError, AppResult};

#[derive(Clone, Debug)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
}

#[derive(Clone)]
pub struct AuthService {
    user_service: Arc<crate::services::UserService>,
    session_service: Arc<crate::services::SessionService>,
}

impl AuthService {
    pub fn new(
        user_service: Arc<crate::services::UserService>,
        session_service: Arc<crate::services::SessionService>,
    ) -> Self {
        Self {
            user_service,
            session_service,
        }
    }

    pub async fn verify_credentials(&self, username: &str, password: &str) -> AppResult<i64> {
        self.user_service
            .verify_credentials(username, password)
            .await
    }

    pub async fn verify_session(&self, session_id: &str) -> AppResult<i64> {
        self.session_service.validate_session(session_id).await
    }

    pub async fn get_username_by_id(&self, user_id: i64) -> AppResult<String> {
        let user = self
            .user_service
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::Authentication)?;
        Ok(user.username)
    }
}

fn extract_session_from_cookie(cookie_header: &str) -> Option<String> {
    // Parse cookie header to extract sessionid
    for cookie in cookie_header.split(';') {
        let parts: Vec<&str> = cookie.trim().splitn(2, '=').collect();
        if parts.len() == 2 && parts[0] == "sessionid" {
            return Some(parts[1].to_string());
        }
    }
    None
}

pub fn with_auth(
    auth_service: AuthService,
) -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
    warp::header::optional::<String>("cookie")
        .and(warp::header::optional::<String>("authorization"))
        .and_then(
            move |cookie_header: Option<String>, auth_header: Option<String>| {
                let auth_service = auth_service.clone();
                async move {
                    tracing::debug!("Auth middleware called");

                    // Try cookie-based authentication first
                    if let Some(cookie) = cookie_header {
                        if let Some(session_id) = extract_session_from_cookie(&cookie) {
                            tracing::debug!("Attempting cookie authentication");
                            match auth_service.verify_session(&session_id).await {
                                Ok(user_id) => {
                                    match auth_service.get_username_by_id(user_id).await {
                                        Ok(username) => {
                                            tracing::info!(
                                                "Cookie auth successful for user: {} (id: {})",
                                                username,
                                                user_id
                                            );
                                            return Ok(AuthContext { user_id, username });
                                        }
                                        Err(e) => {
                                            tracing::warn!("Failed to get username: {:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::debug!("Session validation failed: {:?}", e);
                                }
                            }
                        }
                    }

                    // Fall back to Basic Auth
                    let auth_header = auth_header.ok_or(AppError::Authentication)?;
                    tracing::debug!("Attempting Basic Auth");

                    if !auth_header.starts_with("Basic ") {
                        tracing::warn!("Invalid auth header format");
                        return Err(warp::reject::custom(AppError::Authentication));
                    }

                    let encoded = &auth_header[6..];
                    let decoded = base64::engine::general_purpose::STANDARD
                        .decode(encoded)
                        .map_err(|e| {
                            tracing::error!("Base64 decode error: {}", e);
                            AppError::Authentication
                        })?;

                    let credentials = String::from_utf8(decoded).map_err(|e| {
                        tracing::error!("UTF8 decode error: {}", e);
                        AppError::Authentication
                    })?;

                    let mut parts = credentials.splitn(2, ':');
                    let username = parts.next().ok_or(AppError::Authentication)?.to_string();
                    let password = parts.next().ok_or(AppError::Authentication)?.to_string();

                    tracing::info!("Verifying credentials for user: {}", username);

                    let user_id = auth_service
                        .verify_credentials(&username, &password)
                        .await
                        .map_err(|e| {
                            tracing::error!("Credential verification failed: {:?}", e);
                            warp::reject::custom(e)
                        })?;

                    tracing::info!("Auth successful for user: {} (id: {})", username, user_id);
                    Ok(AuthContext { user_id, username })
                }
            },
        )
}
