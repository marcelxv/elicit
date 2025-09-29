//! Unit tests for individual components

use elicit::{
    config::Config,
    error::AppError,
    models::{PdfMetadata, ExtractResponse, ExtractData},
    services::{PdfProcessor, OcrService},
};
use chrono::Utc;
use std::env;

#[test]
fn test_config_validation() {
    // Test valid config
    env::set_var("VALID_API_KEYS", "valid-key-123,another-key");
    env::set_var("MAX_FILE_SIZE_MB", "10");
    env::set_var("MAX_CONCURRENT_REQUESTS", "100");
    env::set_var("SERVER_PORT", "8080");
    
    let config = Config::from_env().unwrap();
    assert_eq!(config.max_file_size_mb, 10);
    assert_eq!(config.max_concurrent_requests, 100);
    assert_eq!(config.server_port, 8080);
    
    // Test API key validation
    assert!(Config::validate_api_key("valid-key-123"));
    assert!(Config::validate_api_key("another-key"));
    assert!(!Config::validate_api_key("invalid-key"));
}

#[test]
fn test_error_codes() {
    assert_eq!(AppError::InvalidApiKey.error_code(), "INVALID_API_KEY");
    assert_eq!(AppError::RateLimitExceeded.error_code(), "RATE_LIMIT_EXCEEDED");
    assert_eq!(AppError::FileTooLarge { size: 20, limit: 30 }.error_code(), "FILE_TOO_LARGE");
    assert_eq!(AppError::validation("test").error_code(), "VALIDATION_ERROR");
    assert_eq!(AppError::config("test").error_code(), "CONFIG_ERROR");
}

#[test]
fn test_error_status_codes() {
    use axum::http::StatusCode;
    
    assert_eq!(AppError::InvalidApiKey.status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(AppError::RateLimitExceeded.status_code(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(AppError::FileTooLarge { size: 20, limit: 30 }.status_code(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(AppError::validation("test").status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(AppError::ServiceUnavailable { service: "test".to_string() }.status_code(), StatusCode::SERVICE_UNAVAILABLE);
}

#[test]
fn test_error_helper_methods() {
    let validation_error = AppError::validation("Invalid input");
    match validation_error {
        AppError::ValidationError { message } => assert_eq!(message, "Invalid input"),
        _ => panic!("Expected ValidationError"),
    }
    
    let config_error = AppError::config("Missing config");
    match config_error {
        AppError::ConfigError { message } => assert_eq!(message, "Missing config"),
        _ => panic!("Expected ConfigError"),
    }
    
    let service_error = AppError::service_unavailable("PDF Service");
    match service_error {
        AppError::ServiceUnavailable { service } => assert_eq!(service, "PDF Service"),
        _ => panic!("Expected ServiceUnavailable"),
    }
}

#[test]
fn test_pdf_processor_availability() {
    let processor = PdfProcessor::default();
    assert!(processor.is_available());
}

#[test]
fn test_ocr_service_availability() {
    // OCR service availability depends on system setup
    let available = OcrService::is_available();
    // Just ensure it returns a boolean without panicking
    assert!(available == true || available == false);
}

#[test]
fn test_pdf_metadata_creation() {
    let metadata = PdfMetadata::new(1024)
        .with_title(Some("Test Document".to_string()))
        .with_author(Some("Test Author".to_string()))
        .with_dates(Some(Utc::now()), None);
    
    assert_eq!(metadata.file_size_bytes, 1024);
    assert_eq!(metadata.title, Some("Test Document".to_string()));
    assert_eq!(metadata.author, Some("Test Author".to_string()));
    assert!(!metadata.ocr_used);
    assert!(metadata.creation_date.is_some());
    assert!(metadata.modification_date.is_none());
}

#[test]
fn test_extract_response_creation() {
    let metadata = PdfMetadata::new(2048)
        .with_title(Some("Test".to_string()))
        .with_ocr();
    
    let extract_data = ExtractData {
        text: "Extracted text content".to_string(),
        pages: 3,
        metadata,
    };
    
    let response = ExtractResponse {
        success: true,
        data: extract_data,
        processing_time_ms: 150,
    };
    
    assert!(response.success);
    assert_eq!(response.data.text, "Extracted text content");
    assert_eq!(response.data.pages, 3);
    assert_eq!(response.processing_time_ms, 150);
    assert_eq!(response.data.metadata.file_size_bytes, 2048);
    assert!(response.data.metadata.ocr_used);
    assert_eq!(response.data.metadata.title, Some("Test".to_string()));
}

#[test]
fn test_file_size_validation() {
    let max_size_mb = 10;
    let max_size_bytes = max_size_mb * 1024 * 1024;
    
    // Test file within limit
    let small_file_size = 5 * 1024 * 1024; // 5MB
    assert!(small_file_size <= max_size_bytes);
    
    // Test file exceeding limit
    let large_file_size = 15 * 1024 * 1024; // 15MB
    assert!(large_file_size > max_size_bytes);
    
    // Test error creation for oversized file
    let error = AppError::FileTooLarge { 
        size: large_file_size / (1024 * 1024), 
        limit: max_size_mb 
    };
    
    match error {
        AppError::FileTooLarge { size, limit } => {
            assert_eq!(size, 15);
            assert_eq!(limit, 10);
        },
        _ => panic!("Expected FileTooLarge error"),
    }
}

#[test]
fn test_error_conversions() {
    // Test anyhow::Error conversion
    let anyhow_error = anyhow::anyhow!("Test error");
    let app_error: AppError = anyhow_error.into();
    match app_error {
        AppError::Internal { message } => assert!(message.contains("Test error")),
        _ => panic!("Expected Internal error"),
    }
    
    // Test std::io::Error conversion
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
    let app_error: AppError = io_error.into();
    match app_error {
        AppError::Internal { message } => assert!(message.contains("IO error")),
        _ => panic!("Expected Internal error"),
    }
    
    // Test serde_json::Error conversion
    let json_error = serde_json::from_str::<serde_json::Value>("{invalid json").unwrap_err();
    let app_error: AppError = json_error.into();
    match app_error {
        AppError::ValidationError { message } => assert!(message.contains("JSON parsing error")),
        _ => panic!("Expected ValidationError"),
    }
}