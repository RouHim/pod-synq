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

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl warp::reject::Reject for AppError {}

impl Reply for AppError {
    fn into_response(self) -> warp::reply::Response {
        let (status, error_message) = match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database error".to_string(),
                )
            }
            AppError::Authentication => {
                tracing::warn!("Authentication failed");
                (
                    StatusCode::UNAUTHORIZED,
                    "Authentication failed".to_string(),
                )
            }
            AppError::Authorization => {
                tracing::warn!("Authorization failed");
                (StatusCode::FORBIDDEN, "Authorization failed".to_string())
            }
            AppError::BadRequest(msg) => {
                tracing::warn!("Bad request: {}", msg);
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = serde_json::json!({ "error": error_message });
        with_status(json(&body), status).into_response()
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, std::convert::Infallible> {
    let (status, error_message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not found".to_string())
    } else if let Some(app_err) = err.find::<AppError>() {
        match app_err {
            AppError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Database error".to_string(),
            ),
            AppError::Authentication => (
                StatusCode::UNAUTHORIZED,
                "Authentication failed".to_string(),
            ),
            AppError::Authorization => (StatusCode::FORBIDDEN, "Authorization failed".to_string()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        }
    } else {
        tracing::error!("Unhandled rejection: {:?}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    };

    Ok(with_status(
        json(&serde_json::json!({ "error": error_message })),
        status,
    ))
}

pub type AppResult<T> = Result<T, AppError>;
