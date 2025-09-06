use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

pub async fn logging_middleware(mut request: Request, next: Next) -> Response {
    let start = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();
    
    // Add request ID to headers for tracing
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        version = ?version,
        "Request started"
    );

    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %status,
        duration_ms = %duration.as_millis(),
        "Request completed"
    );

    response
}