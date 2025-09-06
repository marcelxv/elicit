use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use std::{env, fs};
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
use handlers::{extract_handler, extract_binary_handler, health_handler, ready_handler, waitlist_handler};
use middleware::auth::auth_middleware;

/// Serve the landing page HTML
async fn serve_landing_page() -> Html<String> {
    // Try local path first (for development), then container path (for deployment)
    let html_content = fs::read_to_string("elicit-landing.html")
        .or_else(|_| fs::read_to_string("/app/elicit-landing.html"))
        .unwrap_or_else(|_| "<h1>Landing page not found</h1>".to_string());
    Html(html_content)
}

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
    // Routes that don't require authentication
    let public_routes = Router::new()
        .route("/", get(serve_landing_page))
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/api/waitlist", post(waitlist_handler));

    // Routes that require authentication
    let protected_routes = Router::new()
        .route("/api/v1/extract", post(extract_handler))
        .route("/api/v1/extract/binary", post(extract_binary_handler))
        .layer(axum::middleware::from_fn(auth_middleware));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(config.max_file_size_mb * 1024 * 1024))
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