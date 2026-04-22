use super::models::{UpdateProfileRequest, User};
use crate::core::errors::{AppError, AppResult};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their unique ID.
    async fn find_by_id(&self, id: Uuid) -> AppResult<User>;

    /// Find a user by their email address.
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;

    /// Find a user by their username.
    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>>;

    /// Create a new user in the database.
    async fn create(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> AppResult<User>;

    /// Update a user's profile information.
    async fn update_profile(&self, id: Uuid, update: &UpdateProfileRequest) -> AppResult<User>;
}

pub struct PostgresUserRepository {
    db: PgPool,
}

impl PostgresUserRepository {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                   created_at, updated_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                   created_at, updated_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    async fn find_by_username(&self, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                   created_at, updated_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(user)
    }

    async fn create(
        &self,
        email: &str,
        username: &str,
        password_hash: &str,
        display_name: Option<&str>,
    ) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (email, username, password_hash, display_name)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, username, password_hash, display_name, bio, avatar_url,
                      created_at, updated_at
            "#,
            email,
            username,
            password_hash,
            display_name
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            // Check for unique constraint violations
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Email or username already exists".to_string());
                }
            }
            AppError::DatabaseError(e)
        })?;

        Ok(user)
    }

    async fn update_profile(&self, id: Uuid, update: &UpdateProfileRequest) -> AppResult<User> {
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET
                display_name = COALESCE($2, display_name),
                bio = COALESCE($3, bio),
                avatar_url = COALESCE($4, avatar_url),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, email, username, password_hash, display_name, bio, avatar_url,
                      created_at, updated_at
            "#,
            id,
            update.display_name,
            update.bio,
            update.avatar_url,
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("User with id {} not found", id)))?;

        Ok(user)
    }
}
