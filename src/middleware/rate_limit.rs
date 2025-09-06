use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::Semaphore;
use tracing::{info, warn, debug};

use crate::error::AppError;

// Metrics for rate limiting
static TOTAL_REQUESTS: AtomicU64 = AtomicU64::new(0);
static REJECTED_REQUESTS: AtomicU64 = AtomicU64::new(0);

// Global semaphore for concurrent request limiting
pub static REQUEST_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| {
    let max_requests = std::env::var("MAX_CONCURRENT_REQUESTS")
        .unwrap_or_else(|_| "100".to_string())
        .parse::<usize>()
        .unwrap_or(100);
    
    info!(
        max_concurrent_requests = max_requests,
        "Initializing request semaphore"
    );
    Semaphore::new(max_requests)
});

pub async fn rate_limit_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    
    // Skip rate limiting for health endpoints
    if path == "/health" || path == "/ready" {
        return Ok(next.run(request).await);
    }

    // Increment total requests counter
    let total_requests = TOTAL_REQUESTS.fetch_add(1, Ordering::Relaxed) + 1;
    
    // Try to acquire a permit
    let _permit = REQUEST_SEMAPHORE
        .try_acquire()
        .map_err(|_| {
            let rejected = REJECTED_REQUESTS.fetch_add(1, Ordering::Relaxed) + 1;
            warn!(
                path = path,
                total_requests = total_requests,
                rejected_requests = rejected,
                available_permits = REQUEST_SEMAPHORE.available_permits(),
                "Rate limit exceeded - too many concurrent requests"
            );
            AppError::RateLimitExceeded
        })?;

    debug!(
        path = path,
        total_requests = total_requests,
        available_permits = REQUEST_SEMAPHORE.available_permits(),
        "Request permit acquired"
    );
    
    // Process the request
    let response = next.run(request).await;
    
    debug!(
        path = path,
        available_permits = REQUEST_SEMAPHORE.available_permits() + 1, // +1 because permit will be released
        "Request completed, permit released"
    );
    
    Ok(response)
}

/// Get rate limiting metrics
pub fn get_rate_limit_metrics() -> (u64, u64, usize) {
    let total = TOTAL_REQUESTS.load(Ordering::Relaxed);
    let rejected = REJECTED_REQUESTS.load(Ordering::Relaxed);
    let available = REQUEST_SEMAPHORE.available_permits();
    (total, rejected, available)
}