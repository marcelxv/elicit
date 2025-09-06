use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use tracing::{debug, warn, info};

use crate::config::Config;
use crate::error::AppError;

pub async fn auth_middleware(headers: HeaderMap, request: Request, next: Next) -> Result<Response, AppError> {
    let path = request.uri().path();
    let method = request.method();
    
    // Skip auth for health endpoint
    if path == "/health" {
        debug!("Skipping auth for health endpoint");
        return Ok(next.run(request).await);
    }

    debug!("Authenticating request: {} {}", method, path);

    // Extract Authorization header
    let auth_header = match headers.get("authorization") {
        Some(header) => match header.to_str() {
            Ok(value) => value,
            Err(_) => {
                warn!("Invalid Authorization header format for {} {}", method, path);
                return Err(AppError::InvalidApiKey);
            }
        },
        None => {
            warn!("Missing Authorization header for {} {}", method, path);
            return Err(AppError::InvalidApiKey);
        }
    };

    // Check for Bearer token format
    if !auth_header.starts_with("Bearer ") {
        warn!("Authorization header missing Bearer prefix for {} {}", method, path);
        return Err(AppError::InvalidApiKey);
    }

    // Extract the token
    let token = auth_header.strip_prefix("Bearer ").unwrap_or("");
    
    if token.is_empty() {
        warn!("Empty Bearer token for {} {}", method, path);
        return Err(AppError::InvalidApiKey);
    }

    // Validate the API key
    if !Config::validate_api_key(token) {
        warn!("Invalid API key attempted for {} {}: {}", method, path, 
              if token.len() > 8 { &token[..8] } else { token });
        return Err(AppError::InvalidApiKey);
    }

    info!("Valid API key authenticated for {} {}", method, path);
    Ok(next.run(request).await)
}