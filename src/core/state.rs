use sqlx::PgPool;
use std::sync::Arc;

use crate::infrastructure::config::Settings;

/// The raw state data — what we actually store.
#[derive(Debug)]
pub struct AppStateInner {
    pub db: PgPool,
    pub settings: Settings,
}

#[derive(Clone, Debug)]
pub struct AppState(pub Arc<AppStateInner>);

impl AppState {
    pub fn new(db: PgPool, settings: Settings) -> Self {
        AppState(Arc::new(AppStateInner { db, settings }))
    }

    /// Get a reference to the database pool.
    pub fn db(&self) -> &PgPool {
        &self.0.db
    }

    /// Get a reference to the application settings.
    pub fn settings(&self) -> &Settings {
        &self.0.settings
    }
}
