use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, State},
    http::{HeaderMap, request::Parts},
};
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::{errors::AppError, state::AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject — the user's ID
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at time (Unix timestamp)
    pub iat: usize,
}

impl Claims {
    /// Get the user's UUID from the claims.
    pub fn user_id(&self) -> Result<Uuid, AppError> {
        Uuid::parse_str(&self.sub).map_err(|_| AppError::InvalidToken)
    }
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub claims: Claims,
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Step 1: Get the Authorization header
        let token = extract_token_from_headers(&parts.headers)?;

        // Step 2: Validate the token using our secret key
        let secret = state.settings().jwt.secret.as_bytes();
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        )
        .map_err(|e| {
            tracing::debug!("JWT validation failed: {}", e);
            AppError::InvalidToken
        })?;

        // Step 3: Extract the user ID from the claims
        let user_id = token_data.claims.user_id()?;

        Ok(AuthUser {
            user_id,
            claims: token_data.claims,
        })
    }
}

fn extract_token_from_headers(headers: &HeaderMap) -> Result<String, AppError> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(AppError::MissingToken)?
        .to_str()
        .map_err(|_| AppError::InvalidToken)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::InvalidToken);
    }

    Ok(auth_header["Bearer ".len()..].to_string())
}

pub fn create_token(
    user_id: &Uuid,
    settings: &crate::infrastructure::config::Settings,
) -> Result<String, AppError> {
    use jsonwebtoken::{EncodingKey, Header, encode};

    let now = chrono::Utc::now();
    let expiration = now + chrono::Duration::hours(settings.jwt.expiration_hours);

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(settings.jwt.secret.as_bytes()),
    )
    .map_err(|e| {
        tracing::error!("Failed to create JWT: {}", e);
        AppError::InternalServerError
    })
}
