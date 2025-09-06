pub mod extract;
pub mod health;
pub mod waitlist;

pub use extract::*;
pub use health::*;
pub use waitlist::*;

#[cfg(test)]
use axum::{
    routing::{get, post},
    Router,
};
#[cfg(test)]
use crate::middleware::auth::auth_middleware;
#[cfg(test)]
use tower::ServiceBuilder;
#[cfg(test)]
use tower_http::cors::CorsLayer;

/// Create router for testing purposes
#[cfg(test)]
pub async fn create_router() -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/api/v1/extract/binary", post(extract_binary_handler))
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(auth_middleware))
        )
}