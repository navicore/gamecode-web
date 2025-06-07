use anyhow::Result;
use axum::{
    Router,
    extract::State,
    http::{StatusCode, Method},
    response::IntoResponse,
    Json,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{CorsLayer, Any},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod auth;
mod config;
mod error;
mod providers;
mod api;

use config::Config;
use providers::ProviderManager;

pub struct AppState {
    pub config: Config,
    pub providers: ProviderManager,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting GameCode Web server...");

    // Load configuration
    let config = Config::load()?;
    info!("Configuration loaded");

    // Initialize provider manager
    let providers = ProviderManager::new(&config).await?;
    info!("Providers initialized: {:?}", providers.list_available());

    // Create shared state
    let state = Arc::new(AppState {
        config: config.clone(),
        providers,
    });

    // Build router
    let app = Router::new()
        // API routes
        .nest("/api", api::routes())
        // Serve static files and WASM app
        .fallback_service(ServeDir::new(&config.server.static_dir))
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any)
        )
        .layer(TraceLayer::new_for_http())
        // Add state
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on http://{}", addr);
    info!("Serve static files from: {}", config.server.static_dir);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}