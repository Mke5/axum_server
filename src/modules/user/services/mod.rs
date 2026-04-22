use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::errors::{AppError, AppResult};
use crate::core::middleware::create_token;
use crate::infrastructure::config::Settings;

use super::models::{
    AuthResponse, LoginRequest, RegisterRequest, UpdateProfileRequest, UserResponse,
};
use super::repository::UserRepository;

pub struct UserService {
    repo: Arc<dyn UserRepository>,
    settings: Settings,
}

impl UserService {
    pub fn new(repo: Arc<dyn UserRepository>, settings: Settings) -> Self {
        Self { repo, settings }
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<UserResponse> {
        tracing::info!("Registering new user: {}", req.email);

        // Step 1: Check for duplicate email
        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        // Step 2: Check for duplicate username
        if self.repo.find_by_username(&req.username).await?.is_some() {
            return Err(AppError::Conflict("Username already taken".to_string()));
        }

        // Step 3: Hash the password using Argon2
        let password_hash = hash_password(&req.password)?;

        // Step 4: Save to database
        let user = self
            .repo
            .create(
                &req.email,
                &req.username,
                &password_hash,
                req.display_name.as_deref(),
            )
            .await?;

        tracing::info!("User registered successfully: {}", user.id);

        Ok(user.into())
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        tracing::info!("Login attempt for: {}", req.email);

        // Step 1: Find the user
        let user = self
            .repo
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        // Step 2: Verify the password
        verify_password(&req.password, &user.password_hash)?;

        // Step 3: Create a JWT token for this user
        let token = create_token(&user.id, &self.settings)?;

        tracing::info!("User logged in successfully: {}", user.id);

        Ok(AuthResponse {
            token,
            user: user.into(),
        })
    }

    pub async fn get_profile(&self, user_id: Uuid) -> AppResult<UserResponse> {
        let user = self.repo.find_by_id(user_id).await?;
        Ok(user.into())
    }

    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> AppResult<UserResponse> {
        tracing::info!("Updating profile for user: {}", user_id);

        // Verify user exists first
        let _ = self.repo.find_by_id(user_id).await?;

        // Apply the update
        let user = self.repo.update_profile(user_id, &req).await?;

        Ok(user.into())
    }
}

fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| {
            tracing::error!("Password hashing failed: {}", e);
            AppError::InternalServerError
        })
}

fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| {
        tracing::error!("Failed to parse password hash: {}", e);
        AppError::InternalServerError
    })?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)
}
