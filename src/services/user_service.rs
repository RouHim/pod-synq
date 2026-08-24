use crate::{
    error::{AppError, AppResult},
    repository::{UserRepository, UserRepositoryTrait},
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

    pub async fn find_by_id(&self, id: i64) -> AppResult<Option<crate::models::User>> {
        self.user_repo
            .find_by_id(id)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub async fn verify_credentials(&self, username: &str, password: &str) -> AppResult<i64> {
        let user = self
            .user_repo
            .find_by_username(username)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?
            .ok_or(AppError::Authentication)?;

        self.verify_password(&user.password_hash, password)?;

        Ok(user.id)
    }

    pub async fn is_empty(&self) -> AppResult<bool> {
        self.user_repo
            .is_empty()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))
    }

    pub fn hash_password(password: &str) -> AppResult<String> {
        use argon2::{
            password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
            Argon2,
        };

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?
            .to_string();

        Ok(password_hash)
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

        if is_empty {
            let password_hash = Self::hash_password(password)?;
            self.create_user(username, &password_hash, true).await?;
            tracing::info!("Initialized admin user: {}", username);
            return Ok(true);
        }

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_returns_phc_format_string_starting_with_argon2id() {
        let hash = UserService::hash_password("correct horse battery staple").unwrap();

        assert!(
            hash.starts_with("$argon2id$"),
            "expected PHC-format argon2id hash, got: {hash}"
        );
    }

    #[test]
    fn test_hash_password_produces_different_hashes_for_same_input_due_to_random_salt() {
        let first = UserService::hash_password("same password").unwrap();
        let second = UserService::hash_password("same password").unwrap();

        assert_ne!(first, second);
    }

    #[tokio::test]
    async fn test_verify_password_accepts_correct_password() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should connect");
        let service = UserService::new(UserRepository::new(pool));

        let hash = UserService::hash_password("s3cret-passw0rd").unwrap();

        assert!(service.verify_password(&hash, "s3cret-passw0rd").is_ok());
    }

    #[tokio::test]
    async fn test_verify_password_rejects_wrong_password() {
        let pool = sqlx::sqlite::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("in-memory sqlite pool should connect");
        let service = UserService::new(UserRepository::new(pool));

        let hash = UserService::hash_password("s3cret-passw0rd").unwrap();

        assert!(service.verify_password(&hash, "wrong-password").is_err());
    }
}
