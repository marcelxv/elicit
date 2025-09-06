use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::env;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;

use config::Config;
use handlers::{extract_handler, extract_binary_handler, health_handler, ready_handler};
use middleware::auth::auth_middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "elicit=debug,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    tracing::info!("Starting Elicit PDF Extractor Service");
    tracing::info!("Max file size: {}MB", config.max_file_size_mb);
    tracing::info!("Max concurrent requests: {}", config.max_concurrent_requests);

    // Build our application with routes
    let app = Router::new()
        // Health endpoints (no auth required)
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        // API endpoints (auth required)
        .route("/api/v1/extract", post(extract_handler))
        .route("/api/v1/extract/binary", post(extract_binary_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(config.max_file_size_mb * 1024 * 1024))
                .layer(axum::middleware::from_fn(auth_middleware))
        );

    // Determine port from environment (Railway compatibility)
    let port = env::var("PORT")
        .unwrap_or_else(|_| config.server_port.to_string())
        .parse::<u16>()
        .unwrap_or(config.server_port);

    let host = config.server_host;
    let addr = format!("{}:{}", host, port);
    
    tracing::info!("Server listening on {}", addr);
    
    let listener = TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}