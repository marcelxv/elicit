use axum::{
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::time::SystemTime;
use tracing::info;

use crate::error::AppResult;
use crate::services::{PdfProcessor, OcrService};
use crate::middleware::rate_limit::get_rate_limit_metrics;

/// Health check endpoint
pub async fn health_handler() -> AppResult<Json<Value>> {
    info!("Health check requested");
    
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Check service availability
    let pdf_service = PdfProcessor::default().is_available();
    let ocr_service = OcrService::is_available();
    
    // Get rate limiting metrics
    let (total_requests, rejected_requests, available_permits) = get_rate_limit_metrics();
    
    let status = if pdf_service {
        "healthy"
    } else {
        "degraded"
    };
    
    let response = json!({
        "status": status,
        "timestamp": timestamp,
        "version": env!("CARGO_PKG_VERSION"),
        "services": {
            "pdf_processor": pdf_service,
            "ocr_service": ocr_service
        },
        "rate_limiting": {
            "total_requests": total_requests,
            "rejected_requests": rejected_requests,
            "available_permits": available_permits,
            "rejection_rate": if total_requests > 0 { 
                (rejected_requests as f64 / total_requests as f64 * 100.0).round() / 100.0 
            } else { 
                0.0 
            }
        },
        "uptime": "N/A" // Could be implemented with a global start time
    });
    
    info!(
        status = status,
        pdf_available = pdf_service,
        ocr_available = ocr_service,
        "Health check completed"
    );
    
    Ok(Json(response))
}

/// Readiness check endpoint (for Kubernetes/Railway)
pub async fn ready_handler() -> Result<StatusCode, StatusCode> {
    let pdf_service = PdfProcessor::default().is_available();
    
    if pdf_service {
        info!("Readiness check passed");
        Ok(StatusCode::OK)
    } else {
        info!("Readiness check failed - PDF service unavailable");
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}