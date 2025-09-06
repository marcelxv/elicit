use axum::{
    extract::Multipart,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use std::time::Instant;
use tracing::{info, warn, debug, error};

use crate::error::{AppError, AppResult};
use crate::models::{ProcessedFile, ExtractResponse};
use crate::services::PdfProcessor;
use crate::middleware::rate_limit::REQUEST_SEMAPHORE;

pub async fn extract_handler(headers: HeaderMap, mut multipart: Multipart) -> AppResult<Json<ExtractResponse>> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    
    info!(request_id = %request_id, "Starting PDF extraction request");
    
    // Acquire rate limiting permit
    let _permit = REQUEST_SEMAPHORE
        .try_acquire()
        .map_err(|_| {
            warn!(request_id = %request_id, "Rate limit exceeded");
            AppError::RateLimitExceeded
        })?;
    
    debug!(request_id = %request_id, "Rate limit permit acquired");
    
    // Extract file from multipart form
    let file = match extract_file_from_multipart(&mut multipart).await {
        Ok(file) => {
            info!(
                request_id = %request_id,
                file_name = %file.name,
                file_size = file.size,
                "File extracted from multipart form"
            );
            file
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Failed to extract file from multipart");
            return Err(e);
        }
    };
    
    // Validate file size
    let max_size_bytes = 10 * 1024 * 1024; // 10MB
    if file.size > max_size_bytes {
        warn!(
            request_id = %request_id,
            file_size = file.size,
            max_size = max_size_bytes,
            "File size exceeds limit"
        );
        return Err(AppError::FileTooLarge {
            size: file.size / (1024 * 1024),
            limit: 10,
        });
    }
    
    // Process the PDF
    let processor = PdfProcessor::new();
    let result = match processor.extract_text(file).await {
        Ok(result) => {
            info!(
                request_id = %request_id,
                text_length = result.text.len(),
                pages = result.pages,
                processing_time_ms = result.processing_time_ms,
                "PDF processing completed successfully"
            );
            result
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "PDF processing failed");
            return Err(e);
        }
    };
    
    let total_time = start.elapsed().as_millis() as u64;
    
    let response = ExtractResponse::new(
        result.text,
        result.pages,
        result.metadata,
        total_time,
    );
    
    info!(
        request_id = %request_id,
        total_time_ms = total_time,
        "Request completed successfully"
    );
    
    Ok(Json(response))
}

async fn extract_file_from_multipart(multipart: &mut Multipart) -> AppResult<ProcessedFile> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InvalidFile {
        message: format!("Failed to read multipart field: {}", e),
    })? {
        let field_name = field.name().unwrap_or("");
        
        if field_name == "file" {
            let file_name = field.file_name()
                .unwrap_or("unknown.pdf")
                .to_string();
            
            let content_type = field.content_type()
                .map(|ct| ct.to_string());
            
            let data = field.bytes().await.map_err(|e| AppError::InvalidFile {
                message: format!("Failed to read file data: {}", e),
            })?;
            
            if data.is_empty() {
                return Err(AppError::InvalidFile {
                    message: "File is empty".to_string(),
                });
            }
            
            let mut file = ProcessedFile::new(file_name, data.to_vec());
            
            if let Some(mime_type) = content_type {
                file = file.with_mime_type(mime_type);
            }
            
            // Validate it's a PDF
            if !file.is_pdf() {
                return Err(AppError::InvalidFile {
                    message: "File is not a valid PDF document".to_string(),
                });
            }
            
            tracing::debug!(
                "Extracted file: {} ({} bytes, type: {:?})",
                file.name,
                file.size,
                file.mime_type
            );
            
            return Ok(file);
        }
    }
    
    Err(AppError::MissingFile)
}

// Alternative handler for direct binary upload
pub async fn extract_binary_handler(
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> AppResult<Json<ExtractResponse>> {
    let start = Instant::now();
    let request_id = uuid::Uuid::new_v4().to_string()[..8].to_string();
    
    info!(request_id = %request_id, "Starting binary PDF extraction request");
    
    // Acquire rate limiting permit
    let _permit = REQUEST_SEMAPHORE
        .try_acquire()
        .map_err(|_| {
            warn!(request_id = %request_id, "Rate limit exceeded");
            AppError::RateLimitExceeded
        })?;
    
    debug!(request_id = %request_id, "Rate limit permit acquired");
    
    // Check content type
    let content_type = headers
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("");
    
    if !content_type.contains("application/pdf") {
        warn!(
            request_id = %request_id,
            content_type = content_type,
            "Invalid content type for binary upload"
        );
        return Err(AppError::InvalidContentType);
    }
    
    if body.is_empty() {
        warn!(request_id = %request_id, "Empty body received");
        return Err(AppError::MissingFile);
    }
    
    // Validate file size
    let max_size_bytes = 10 * 1024 * 1024; // 10MB
    if body.len() > max_size_bytes {
        warn!(
            request_id = %request_id,
            file_size = body.len(),
            max_size = max_size_bytes,
            "Binary file size exceeds limit"
        );
        return Err(AppError::FileTooLarge {
            size: body.len() / (1024 * 1024),
            limit: 10,
        });
    }
    
    let file = ProcessedFile::new(
        "uploaded.pdf".to_string(),
        body.to_vec(),
    ).with_mime_type("application/pdf".to_string());
    
    info!(
        request_id = %request_id,
        file_size = file.size,
        "Processing binary PDF"
    );
    
    // Process the PDF
    let processor = PdfProcessor::new();
    let result = match processor.extract_text(file).await {
        Ok(result) => {
            info!(
                request_id = %request_id,
                text_length = result.text.len(),
                pages = result.pages,
                processing_time_ms = result.processing_time_ms,
                "Binary PDF processing completed successfully"
            );
            result
        }
        Err(e) => {
            error!(request_id = %request_id, error = %e, "Binary PDF processing failed");
            return Err(e);
        }
    };
    
    let total_time = start.elapsed().as_millis() as u64;
    
    let response = ExtractResponse::new(
        result.text,
        result.pages,
        result.metadata,
        total_time,
    );
    
    info!(
        request_id = %request_id,
        total_time_ms = total_time,
        "Binary request completed successfully"
    );
    
    Ok(Json(response))
}