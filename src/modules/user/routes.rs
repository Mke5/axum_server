use axum::{
    Router,
    routing::{get, post, put},
};

use super::handlers;
use crate::core::state::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        // Public routes — no authentication needed
        .route("/api/users/register", post(handlers::register))
        .route("/api/users/login", post(handlers::login))
        // Protected routes — require valid JWT token
        .route("/api/users/me", get(handlers::get_profile))
        .route("/api/users/me", put(handlers::update_profile))
        .with_state(state)
}
