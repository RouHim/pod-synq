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
}

impl AuthService {
    pub fn new(user_service: Arc<crate::services::UserService>) -> Self {
        Self { user_service }
    }

    pub async fn verify_credentials(&self, username: &str, password: &str) -> AppResult<i64> {
        self.user_service
            .verify_credentials(username, password)
            .await
    }
}

pub fn with_auth(
    auth_service: AuthService,
) -> impl Filter<Extract = (AuthContext,), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization").and_then(
        move |auth_header: Option<String>| {
            let auth_service = auth_service.clone();
            async move {
                tracing::info!("Auth middleware called");
                let auth_header = auth_header.ok_or(AppError::Authentication)?;
                tracing::info!("Auth header present: {}", &auth_header[..15]);

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
