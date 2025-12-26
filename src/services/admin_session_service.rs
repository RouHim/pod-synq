use crate::{
    error::{AppError, AppResult},
    repository::AdminSessionRepository,
};

#[derive(Clone)]
pub struct AdminSessionService {
    session_repo: AdminSessionRepository,
}

impl AdminSessionService {
    pub fn new(session_repo: AdminSessionRepository) -> Self {
        Self { session_repo }
    }

    pub async fn create_session(&self, user_id: i64, duration_hours: i64) -> AppResult<String> {
        let expires_at = chrono::Utc::now().timestamp() + (duration_hours * 3600);
        let session = self.session_repo.create(user_id, expires_at).await?;
        tracing::info!("Created session for user ID: {}", user_id);
        Ok(session.id)
    }

    pub async fn validate_session(&self, session_id: &str) -> AppResult<i64> {
        let session = self
            .session_repo
            .find_by_id(session_id)
            .await?
            .ok_or(AppError::SessionExpired)?;

        let now = chrono::Utc::now().timestamp();
        if session.expires_at < now {
            self.session_repo.delete(session_id).await?;
            return Err(AppError::SessionExpired);
        }

        Ok(session.user_id)
    }

    pub async fn delete_session(&self, session_id: &str) -> AppResult<()> {
        self.session_repo.delete(session_id).await?;
        tracing::info!("Deleted session: {}", session_id);
        Ok(())
    }

    pub async fn delete_user_sessions(&self, user_id: i64) -> AppResult<()> {
        self.session_repo.delete_by_user(user_id).await?;
        tracing::info!("Deleted all sessions for user ID: {}", user_id);
        Ok(())
    }

    pub async fn cleanup_expired_sessions(&self) -> AppResult<()> {
        self.session_repo.cleanup_expired().await?;
        tracing::info!("Cleaned up expired sessions");
        Ok(())
    }
}
