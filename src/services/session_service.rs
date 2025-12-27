use crate::{
    error::{AppError, AppResult},
    repository::SessionRepository,
};
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_DURATION_SECS: i64 = 30 * 24 * 60 * 60; // 30 days

#[derive(Clone)]
pub struct SessionService {
    session_repo: SessionRepository,
}

impl SessionService {
    pub fn new(session_repo: SessionRepository) -> Self {
        Self { session_repo }
    }

    pub async fn create_session(&self, user_id: i64) -> AppResult<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let expires_at = current_time + SESSION_DURATION_SECS;

        self.session_repo
            .create(&session_id, user_id, expires_at)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        tracing::info!(
            "Created session {} for user {} (expires at {})",
            session_id,
            user_id,
            expires_at
        );

        Ok(session_id)
    }

    pub async fn validate_session(&self, session_id: &str) -> AppResult<i64> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::Authentication)?;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if session.expires_at < current_time {
            // Session expired, delete it
            self.session_repo
                .delete(session_id)
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            return Err(AppError::Authentication);
        }

        Ok(session.user_id)
    }

    pub async fn delete_session(&self, session_id: &str) -> AppResult<()> {
        self.session_repo
            .delete(session_id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        tracing::info!("Deleted session {}", session_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn cleanup_expired_sessions(&self) -> AppResult<u64> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let count = self
            .session_repo
            .delete_expired(current_time)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if count > 0 {
            tracing::info!("Cleaned up {} expired sessions", count);
        }

        Ok(count)
    }
}
