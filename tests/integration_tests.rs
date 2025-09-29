//! Integration tests for the Elicit PDF extraction service

use std::env;
use elicit::{
    config::Config,
    error::AppError,
};

#[tokio::test]
async fn test_config_loading() {
    // Clean up environment variables from other tests
    env::remove_var("SERVER_HOST");
    env::remove_var("SERVER_PORT");
    env::remove_var("MAX_FILE_SIZE_MB");
    env::remove_var("MAX_CONCURRENT_REQUESTS");
    
    // Set environment variables before any config operations
    env::set_var("SERVER_HOST", "127.0.0.1");
    env::set_var("SERVER_PORT", "8080");
    env::set_var("MAX_FILE_SIZE_MB", "5");
    env::set_var("MAX_CONCURRENT_REQUESTS", "50");
    
    let config = Config::from_env().unwrap();
    assert_eq!(config.server_host, "127.0.0.1");
    assert_eq!(config.server_port, 8080);
    assert_eq!(config.max_file_size_mb, 5);
    assert_eq!(config.max_concurrent_requests, 50);

    // Clean up after test
    env::remove_var("SERVER_HOST");
    env::remove_var("SERVER_PORT");
    env::remove_var("MAX_FILE_SIZE_MB");
    env::remove_var("MAX_CONCURRENT_REQUESTS");

    // Note: API key validation testing is complex due to static initialization
    // This would require a separate test process or refactoring the API key handling
}

#[tokio::test]
async fn test_error_response_format() {
    let error = AppError::InvalidApiKey;
    
    // Test error code
    assert_eq!(error.error_code(), "INVALID_API_KEY");
    
    // Test status code
    use axum::http::StatusCode;
    assert_eq!(error.status_code(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_file_size_validation() {
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

#[tokio::test]
async fn test_error_helper_methods() {
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

#[tokio::test]
async fn test_concurrent_request_limits() {
    // Clean up environment variables from other tests
    env::remove_var("SERVER_HOST");
    env::remove_var("SERVER_PORT");
    env::remove_var("MAX_FILE_SIZE_MB");
    env::remove_var("MAX_CONCURRENT_REQUESTS");
    env::remove_var("REQUEST_TIMEOUT_SECONDS");
    env::remove_var("WORKER_THREADS");

    env::set_var("MAX_CONCURRENT_REQUESTS", "5");

    let config = Config::from_env().unwrap();
    assert_eq!(config.max_concurrent_requests, 5);
    
    // Test that we can create the semaphore with the configured limit
    let semaphore = tokio::sync::Semaphore::new(config.max_concurrent_requests);
    assert_eq!(semaphore.available_permits(), 5);
}