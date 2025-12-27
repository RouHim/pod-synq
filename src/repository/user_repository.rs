use crate::models::User;
use sqlx::{Error, Row, SqlitePool};

#[derive(Clone)]
pub struct UserRepository {
    pool: SqlitePool,
}

impl UserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        username: &str,
        password_hash: &str,
        is_admin: bool,
    ) -> Result<User, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO users (username, password_hash, is_admin)
            VALUES (?, ?, ?)
            RETURNING id, username, password_hash, is_admin, created_at
            "#,
        )
        .bind(username)
        .bind(password_hash)
        .bind(is_admin)
        .fetch_one(&self.pool)
        .await?;

        Ok(User {
            id: result.get_unchecked(0),
            username: result.get_unchecked::<&str, _>(1).to_string(),
            password_hash: result.get_unchecked::<&str, _>(2).to_string(),
            is_admin: result.get_unchecked::<i32, _>(3) != 0,
            created_at: result.get_unchecked(4),
        })
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        let result = sqlx::query(
            r#"
            SELECT 
                id, username, password_hash, 
                CAST(is_admin AS INTEGER) as is_admin, 
                created_at
            FROM users 
            WHERE username = ?
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|row| User {
            id: row.get_unchecked(0),
            username: row.get_unchecked::<&str, _>(1).to_string(),
            password_hash: row.get_unchecked::<&str, _>(2).to_string(),
            is_admin: row.get_unchecked::<i32, _>(3) != 0,
            created_at: row.get_unchecked(4),
        }))
    }

    pub async fn is_empty(&self) -> Result<bool, Error> {
        let result = sqlx::query("SELECT COUNT(*) as count FROM users")
            .fetch_one(&self.pool)
            .await?;
        Ok(result.get_unchecked::<i64, _>(0) == 0)
    }
}
