use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::infrastructure::config::Settings;

pub fn init(settings: &Settings) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        if settings.app.env == "production" {
            EnvFilter::new("info")
        } else {
            EnvFilter::new("debug,axum_server=debug,sqlx=warn,tower_http=debug")
        }
    });

    if settings.app.env == "production" {
        // In production: JSON format (great for log aggregation tools like Datadog)
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        // In development: Pretty human-readable format
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}
