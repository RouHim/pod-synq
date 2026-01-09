use crate::constants::SESSION_DURATION_SECS;
use crate::error::AppResult;
use crate::repository::traits::SessionRepositoryTrait;
use crate::repository::SessionRepository;
use crate::utils::unix_timestamp;

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
        let current_time = unix_timestamp();
        let expires_at = current_time + SESSION_DURATION_SECS;

        self.session_repo
            .create(&session_id, user_id, expires_at)
            .await?;

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
            .await?
            .ok_or(crate::error::AppError::Authentication)?;

        let current_time = unix_timestamp();

        if session.expires_at < current_time {
            // Session expired, delete it
            self.session_repo.delete(session_id).await?;
            return Err(crate::error::AppError::Authentication);
        }

        Ok(session.user_id)
    }

    pub async fn delete_session(&self, session_id: &str) -> AppResult<()> {
        self.session_repo.delete(session_id).await?;
        tracing::info!("Deleted session {}", session_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn cleanup_expired_sessions(&self) -> AppResult<u64> {
        let current_time = unix_timestamp();

        let count = self.session_repo.delete_expired(current_time).await?;

        if count > 0 {
            tracing::info!("Cleaned up {} expired sessions", count);
        }

        Ok(count)
    }
}
