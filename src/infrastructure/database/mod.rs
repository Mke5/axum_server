use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::infrastructure::config::DatabaseSettings;

pub async fn create_pool(settings: &DatabaseSettings) -> anyhow::Result<PgPool> {
    tracing::info!("Connecting to PostgreSQL database...");

    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .connect(&settings.url)
        .await?;

    tracing::info!(
        "Database connected! Pool size: {}",
        settings.max_connections
    );

    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("Migrations complete!");
    Ok(())
}
