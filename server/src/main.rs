use anyhow::Result;
use axum::{http::Method, Router};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

mod api;
mod auth;
mod config;
mod error;
mod providers;

use auth::OidcClient;
use config::Config;
use providers::ProviderManager;

pub struct AppState {
    pub config: Config,
    pub providers: ProviderManager,
    pub oidc: OidcClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    FmtSubscriber::builder().with_max_level(Level::INFO).init();

    info!("Starting GameCode Web server...");

    let config = Config::load()?;
    info!("Configuration loaded");

    let oidc = OidcClient::discover(config.auth.oidc.clone()).await?;
    info!("OIDC metadata discovered: issuer={}", oidc.config.issuer);

    let providers = ProviderManager::new(&config).await?;
    info!("Providers initialized: {:?}", providers.list_available());

    let state = Arc::new(AppState {
        config: config.clone(),
        providers,
        oidc,
    });

    let app = Router::new()
        .nest("/api", api::routes(state.clone()))
        .fallback_service(ServeDir::new(&config.server.static_dir))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST])
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server listening on http://{}", addr);
    info!("Serve static files from: {}", config.server.static_dir);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
