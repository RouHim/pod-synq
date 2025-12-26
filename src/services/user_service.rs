use crate::{error::AppError, models::User, repository::UserRepository};

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(user_repo: UserRepository) -> Self {
        Self { user_repo }
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> AppResult<()> {
        self.user_repo
            .create(username, password_hash, is_admin)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn verify_credentials(&self, username: &str, password: &str) -> AppResult<i64> {
        self.user_repo
            .find_by_username(username, password)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn get_user(&self, id: i64) -> AppResult<crate::models::User> {
        self.user_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn reset_password(&self, id: i64, new_password: &str) -> AppResult<()> {
        self.user_repo
            .update_password(id, new_password)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        Ok(())
    }

    pub async fn delete_user(&self, id: i64) -> AppResult<()> {
        self.user_repo
            .delete(id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        tracing::info!("Deleted user ID: {}", id);
        Ok(())
    }

    pub async fn list_users(&self) -> AppResult<Vec<crate::models::User>> {
        self.user_repo
            .list_all()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn count(&self) -> AppResult<i64> {
        self.user_repo
            .count()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn is_empty(&self) -> AppResult<bool> {
        self.user_repo
            .is_empty()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
    }

    pub async fn initialize_admin_if_needed(
        &self,
        username: &str,
        password: &str,
    ) -> AppResult<bool> {
        let is_empty = self.is_empty().await?;

        if !is_empty {
            self.create_user(username, password, true).await?;
            tracing::info!("Initialized admin user: {}", username);
            return Ok(true);
        }

        Ok(false)
    }
}
