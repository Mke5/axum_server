use axum::{
    Router,
    http::{HeaderValue, Method},
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

use crate::{
    core::state::AppState,
    infrastructure::{config::Settings, database},
    modules,
};

pub struct Application {
    router: Router,
    address: SocketAddr,
}

impl Application {
    pub async fn new(settings: Settings) -> anyhow::Result<Self> {
        // --- Database Setup ---
        let db_pool = database::create_pool(&settings.database).await?;
        database::run_migrations(&db_pool).await?;

        // --- Shared State ---
        // This is passed to every route handler.
        let state = AppState::new(db_pool, settings.clone());

        // --- Build the Router ---
        let router = build_router(state);

        // --- Parse the address ---
        let address: SocketAddr = settings
            .server_address()
            .parse()
            .expect("Invalid server address");

        Ok(Application { router, address })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        tracing::info!("🌐 Server listening on http://{}", self.address);

        let listener = tokio::net::TcpListener::bind(self.address).await?;
        axum::serve(listener, self.router).await?;

        Ok(())
    }
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_origin(Any); // In production, restrict this!

    Router::new()
        .route("/health", axum::routing::get(health_check))
        .merge(modules::user::routes(state.clone()))
        .merge(modules::post::routes(state.clone()))
        .layer(TraceLayer::new_for_http()) // Logs every request
        .layer(cors)
}

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "service": "axum-server",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
