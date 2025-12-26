use thiserror::Error;
use warp::http::StatusCode;
use warp::{
    reply::{json, with_status},
    Rejection, Reply,
};

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Authentication failed")]
    Authentication,

    #[error("Authorization failed")]
    Authorization,

    #[error("User not found")]
    UserNotFound,

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Subscription not found")]
    SubscriptionNotFound,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Session expired")]
    SessionExpired,

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl warp::reject::Reject for AppError {}

impl Reply for AppError {
    fn into_response(self) -> warp::reply::Response {
        let (status, error_message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::Authentication => {
                tracing::warn!("Authentication failed");
                (StatusCode::UNAUTHORIZED, "Authentication failed")
            }
            AppError::Authorization => {
                tracing::warn!("Authorization failed");
                (StatusCode::FORBIDDEN, "Authorization failed")
            }
            AppError::UserNotFound => {
                tracing::info!("User not found");
                (StatusCode::NOT_FOUND, "User not found")
            }
            AppError::DeviceNotFound => {
                tracing::info!("Device not found");
                (StatusCode::NOT_FOUND, "Device not found")
            }
            AppError::SubscriptionNotFound => {
                tracing::info!("Subscription not found");
                (StatusCode::NOT_FOUND, "Subscription not found")
            }
            AppError::InvalidInput => {
                tracing::warn!("Invalid input");
                (StatusCode::BAD_REQUEST, "Invalid input")
            }
            AppError::Configuration(_) => {
                tracing::error!("Configuration error");
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            AppError::SessionExpired => {
                tracing::info!("Session expired");
                (StatusCode::UNAUTHORIZED, "Session expired")
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };

        let body = serde_json::json!({ "error": error_message });
        with_status(json(&body), status).into_response()
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (status, error_message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found")
    } else if let Some(app_err) = err.find::<AppError>() {
        match app_err {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::Authentication => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            AppError::Authorization => (StatusCode::FORBIDDEN, "Authorization failed"),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            AppError::DeviceNotFound => (StatusCode::NOT_FOUND, "Device not found"),
            AppError::SubscriptionNotFound => (StatusCode::NOT_FOUND, "Subscription not found"),
            AppError::InvalidInput => (StatusCode::BAD_REQUEST, "Invalid input"),
            AppError::Configuration(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error")
            }
            AppError::SessionExpired => (StatusCode::UNAUTHORIZED, "Session expired"),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        }
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
    };

    Ok(with_status(
        json(&serde_json::json!({ "error": error_message })),
        status,
    ))
}

pub type AppResult<T> = Result<T, AppError>;
