use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;
use uuid::Uuid;
use chrono;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid API key")]
    InvalidApiKey,
    
    #[error("File too large: {size}MB exceeds limit of {limit}MB")]
    FileTooLarge { size: usize, limit: usize },
    
    #[error("Invalid file format: {message}")]
    InvalidFile { message: String },
    
    #[error("Rate limit exceeded: maximum concurrent requests reached")]
    RateLimitExceeded,
    
    #[error("PDF processing failed: {message}")]
    ProcessingError { message: String },
    
    #[error("OCR processing failed: {message}")]
    OcrError { message: String },
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Internal server error: {message}")]
    Internal { message: String },
    
    #[error("Missing or invalid content type")]
    InvalidContentType,
    
    #[error("Missing file in request")]
    MissingFile,
    
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigError { message: String },
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("Database error: {message}")]
    DatabaseError { message: String },
}

impl AppError {
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::InvalidApiKey => "INVALID_API_KEY",
            AppError::FileTooLarge { .. } => "FILE_TOO_LARGE",
            AppError::InvalidFile { .. } => "INVALID_FILE",
            AppError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            AppError::ProcessingError { .. } => "PROCESSING_ERROR",
            AppError::OcrError { .. } => "OCR_ERROR",
            AppError::Timeout => "REQUEST_TIMEOUT",
            AppError::Internal { .. } => "INTERNAL_ERROR",
            AppError::InvalidContentType => "INVALID_CONTENT_TYPE",
            AppError::MissingFile => "MISSING_FILE",
            AppError::ValidationError { .. } => "VALIDATION_ERROR",
            AppError::ConfigError { .. } => "CONFIG_ERROR",
            AppError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            AppError::DatabaseError { .. } => "DATABASE_ERROR",
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::InvalidApiKey => StatusCode::UNAUTHORIZED,
            AppError::FileTooLarge { .. } => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::InvalidFile { .. } => StatusCode::BAD_REQUEST,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::ProcessingError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::OcrError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Timeout => StatusCode::REQUEST_TIMEOUT,
            AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InvalidContentType => StatusCode::BAD_REQUEST,
            AppError::MissingFile => StatusCode::BAD_REQUEST,
            AppError::ValidationError { .. } => StatusCode::BAD_REQUEST,
            AppError::ConfigError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AppError::DatabaseError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();
        let request_id = Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Structured logging with context
        tracing::error!(
            error_code = error_code,
            status_code = %status,
            request_id = %request_id,
            error_message = %message,
            "API error occurred"
        );

        let body = Json(json!({
            "success": false,
            "error": {
                "code": error_code,
                "message": message,
                "request_id": request_id,
                "timestamp": timestamp
            },
            "data": null
        }));

        (status, body).into_response()
    }
}

// Convert common errors to AppError
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal {
            message: err.to_string(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Internal {
            message: format!("IO error: {}", err),
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ValidationError {
            message: format!("JSON parsing error: {}", err),
        }
    }
}

impl From<tokio::time::error::Elapsed> for AppError {
    fn from(_: tokio::time::error::Elapsed) -> Self {
        AppError::Timeout
    }
}

// Helper methods for creating specific errors
impl AppError {
    pub fn validation(message: impl Into<String>) -> Self {
        AppError::ValidationError {
            message: message.into(),
        }
    }
    
    pub fn config(message: impl Into<String>) -> Self {
        AppError::ConfigError {
            message: message.into(),
        }
    }
    
    pub fn service_unavailable(service: impl Into<String>) -> Self {
        AppError::ServiceUnavailable {
            service: service.into(),
        }
    }
    
    pub fn processing(message: impl Into<String>) -> Self {
        AppError::ProcessingError {
            message: message.into(),
        }
    }
    
    pub fn internal(message: impl Into<String>) -> Self {
        AppError::Internal {
            message: message.into(),
        }
    }
}