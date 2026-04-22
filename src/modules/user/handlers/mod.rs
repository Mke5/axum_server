use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;
use validator::Validate;

use super::models::{LoginRequest, RegisterRequest, UpdateProfileRequest};
use super::repository::PostgresUserRepository;
use super::services::UserService;
use crate::core::{errors::AppError, middleware::AuthUser, state::AppState};

fn make_service(state: &AppState) -> UserService {
    let repo = Arc::new(PostgresUserRepository::new(state.db().clone()));
    UserService::new(repo, state.settings().clone())
}

// ============================================================
// HANDLER: POST /api/users/register
// ============================================================
/// Register a new user account.
///
/// Request body (JSON):
/// ```json
/// {
///   "email": "alice@example.com",
///   "username": "alice",
///   "password": "supersecret123",
///   "display_name": "Alice"
/// }
/// ```
///
/// Response (201 Created):
/// ```json
/// {
///   "id": "uuid",
///   "email": "alice@example.com",
///   "username": "alice",
///   ...
/// }
/// ```
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate the request data (checks email format, password length, etc.)
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let service = make_service(&state);
    let user = service.register(payload).await?;

    // Return 201 Created with the new user data
    Ok((StatusCode::CREATED, Json(user)))
}

// ============================================================
// HANDLER: POST /api/users/login
// ============================================================
/// Authenticate a user and get a JWT token.
///
/// Request body (JSON):
/// ```json
/// { "email": "alice@example.com", "password": "supersecret123" }
/// ```
///
/// Response (200 OK):
/// ```json
/// {
///   "token": "eyJhbG...",
///   "user": { ... }
/// }
/// ```
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let service = make_service(&state);
    let auth_response = service.login(payload).await?;

    Ok((StatusCode::OK, Json(auth_response)))
}

// ============================================================
// HANDLER: GET /api/users/me
// ============================================================
/// Get the currently authenticated user's profile.
///
/// Requires: Authorization: Bearer <token>
///
/// Response (200 OK):
/// ```json
/// {
///   "id": "uuid",
///   "email": "alice@example.com",
///   ...
/// }
/// ```
///
/// Note the `auth: AuthUser` parameter!
/// Axum automatically runs our auth middleware to get this.
/// If the token is invalid, Axum rejects the request with 401
/// BEFORE this function even runs.
pub async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser, // This validates the JWT token automatically.
) -> Result<impl IntoResponse, AppError> {
    let service = make_service(&state);
    let user = service.get_profile(auth.user_id).await?;

    Ok((StatusCode::OK, Json(user)))
}

// ============================================================
// HANDLER: PUT /api/users/me
// ============================================================
/// Update the currently authenticated user's profile.
///
/// Requires: Authorization: Bearer <token>
///
/// Request body (JSON):
/// ```json
/// {
///   "display_name": "Alice Wonderland",
///   "bio": "I love code!",
///   "avatar_url": "https://example.com/alice.jpg"
/// }
/// ```
pub async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let service = make_service(&state);
    let user = service.update_profile(auth.user_id, payload).await?;

    Ok((StatusCode::OK, Json(user)))
}
