use crate::{
    error::{AppError, AppResult},
    repository::UserRepository,
};
use argon2::PasswordVerifier;

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
        let user = self
            .user_repo
            .find_by_username(username)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::Authentication)?;

        let user_service = UserService::new(self.user_repo.clone());
        user_service.verify_password(&user.password_hash, password)?;

        Ok(user.id)
    }

    pub async fn is_empty(&self) -> AppResult<bool> {
        self.user_repo
            .is_empty()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn verify_password(&self, password_hash: &str, password: &str) -> AppResult<()> {
        let parsed_hash = argon2::PasswordHash::new(password_hash)
            .map_err(|_| AppError::Internal("Invalid password hash format".to_string()))?;

        argon2::Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Authentication)?;
        Ok(())
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
