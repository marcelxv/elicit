# Elicit – PDF Extractor Service Specification

## Overview

Elicit is a Rust-based HTTP service that extracts text from PDF documents, similar to LlamaParse. The service receives PDF files via HTTP requests along with API key authentication and returns extracted text content.

## Architecture

### Technology Stack
- **Language**: Rust
- **Web Framework**: Axum or Warp
- **PDF Processing**: `pdf-extract` or `lopdf` crate
- **Authentication**: Custom API key validation
- **Serialization**: `serde` with JSON
- **HTTP Client**: `reqwest` (if needed for external services)
- **Logging**: `tracing` and `tracing-subscriber`
- **Configuration**: `config` crate or environment variables

### Core Components
1. **HTTP Server**: Handles incoming requests
2. **Authentication Middleware**: Validates API keys
3. **PDF Processor**: Extracts text from PDF files
4. **Response Handler**: Formats and returns extracted text
5. **Error Handler**: Manages and formats errors
6. **Configuration Manager**: Handles service configuration

## API Specification

### Endpoint

#### Extract Text from PDF

**POST** `/api/v1/extract`

**Headers:**
- `Authorization: Bearer <api_key>`
- `Content-Type: multipart/form-data` or `application/pdf`

**Request Body:**
- **Multipart Form**: PDF file in `file` field
- **Binary**: Raw PDF content

**Response:**

**Success (200 OK):**
```json
{
  "success": true,
  "data": {
    "text": "Extracted text content from the PDF...",
    "pages": 5,
    "metadata": {
      "title": "Document Title",
      "author": "Author Name",
      "creation_date": "2024-01-15T10:30:00Z",
      "modification_date": "2024-01-15T10:30:00Z"
    }
  },
  "processing_time_ms": 1250
}
```

**Error Responses:**

**400 Bad Request:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_FILE",
    "message": "Invalid PDF file or corrupted content"
  }
}
```

**401 Unauthorized:**
```json
{
  "success": false,
  "error": {
    "code": "INVALID_API_KEY",
    "message": "Invalid or missing API key"
  }
}
```

**413 Payload Too Large:**
```json
{
  "success": false,
  "error": {
    "code": "FILE_TOO_LARGE",
    "message": "File size exceeds maximum limit of 10MB"
  }
}
```

**429 Too Many Requests:**
```json
{
  "success": false,
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Maximum concurrent requests limit reached. Please try again later."
  }
}
```

**500 Internal Server Error:**
```json
{
  "success": false,
  "error": {
    "code": "PROCESSING_ERROR",
    "message": "Failed to process PDF file"
  }
}
```

## Configuration

### Environment Variables

```bash
# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# File Processing
MAX_FILE_SIZE_MB=10
MAX_CONCURRENT_REQUESTS=100

# API Keys (comma-separated)
VALID_API_KEYS=key1,key2,key3

# Logging
RUST_LOG=info
LOG_FORMAT=json

# Performance
WORKER_THREADS=4
REQUEST_TIMEOUT_SECONDS=30
```

### Configuration File (config.toml)

```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4

[processing]
max_file_size_mb = 10
max_concurrent_requests = 100
request_timeout_seconds = 30

[security]
api_keys = ["key1", "key2", "key3"]
rate_limit_per_minute = 60

[logging]
level = "info"
format = "json"
```

## Project Structure

```
elicit/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .env.example
├── Dockerfile
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── config/
│   │   └── mod.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   └── extract.rs
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── rate_limit.rs
│   │   └── logging.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── pdf_processor.rs
│   │   └── ocr_service.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── request.rs
│   │   └── response.rs
│   └── error/
│       ├── mod.rs
│       └── types.rs
├── tests/
│   ├── integration/
│   │   └── api_tests.rs
│   └── unit/
│       ├── pdf_processor_tests.rs
│       └── ocr_service_tests.rs
└── examples/
    ├── sample.pdf
    ├── scanned_sample.pdf
    └── client_example.rs
```

## Dependencies (Cargo.toml)

```toml
[package]
name = "elicit"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# PDF processing
pdf-extract = "0.7"
# OCR support
tesseract = "0.14"
image = "0.24"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Configuration
config = "0.14"
dotenvy = "0.15"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Utilities
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
bytes = "1.0"
once_cell = "1.19"

# File handling
multipart = "0.18"

# Concurrency control
tokio = { version = "1.0", features = ["full", "sync"] }

[dev-dependencies]
reqwest = { version = "0.11", features = ["multipart"] }
tempfile = "3.0"
```

## Core Features

### 1. PDF Text Extraction
- Support for various PDF formats
- Plain text extraction (no formatting preservation)
- OCR support for scanned PDFs
- Fail-fast error handling for corrupted files
- Memory-efficient processing for files up to 10MB

### 2. Authentication
- API key-based authentication via environment variables
- Bearer token support
- Simple key validation middleware
- Global concurrent request limiting (100 max)

### 3. File Handling
- Multipart form data support
- Binary PDF upload
- File size validation
- MIME type verification
- Temporary file cleanup

### 4. Performance
- Async/await processing
- Concurrent request handling
- Memory-efficient streaming
- Request timeout handling
- Resource cleanup

### 5. Monitoring & Logging
- Structured logging with tracing
- Request/response logging
- Basic performance metrics
- Error tracking
- Health check endpoint
- Concurrent request counter

## Security Considerations

1. **Input Validation**
   - File size limits
   - MIME type verification
   - PDF structure validation

2. **Authentication**
   - Environment-based API key storage
   - Global concurrent request limiting
   - Request timeout

3. **Resource Management**
   - Memory limits
   - Temporary file cleanup
   - Process isolation

4. **Error Handling**
   - No sensitive information in errors
   - Proper error logging
   - Graceful degradation

## Performance Requirements

- **Throughput**: Handle 100+ concurrent requests
- **Latency**: < 5 seconds for typical PDFs (< 10MB)
- **Memory**: < 512MB per request
- **File Size**: Support up to 10MB PDFs
- **Availability**: 99.9% uptime

## Testing Strategy

### Unit Tests
- PDF processing functions
- Authentication middleware
- Configuration loading
- Error handling

### Integration Tests
- End-to-end API testing
- File upload scenarios
- Authentication flows
- Error response validation

### Load Testing
- Concurrent request handling
- Memory usage under load
- Response time benchmarks

## Deployment

### Docker Support (Railway Compatible)
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tesseract-ocr \
    tesseract-ocr-eng \
    libtesseract-dev \
    libleptonica-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/elicit /usr/local/bin/
EXPOSE $PORT
CMD ["elicit"]
```

### Railway Deployment
- Set `PORT` environment variable (Railway provides this automatically)
- Configure `VALID_API_KEYS` in Railway environment variables
- Set `MAX_FILE_SIZE_MB=10` and `MAX_CONCURRENT_REQUESTS=100`
- Railway will handle HTTPS termination and domain routing

### Health Check Endpoint
**GET** `/health`

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

## Implementation Details

### Concurrent Request Limiting
```rust
use tokio::sync::Semaphore;
use once_cell::sync::Lazy;

static REQUEST_SEMAPHORE: Lazy<Semaphore> = Lazy::new(|| {
    Semaphore::new(100) // Max 100 concurrent requests
});

// In handler:
let _permit = REQUEST_SEMAPHORE.acquire().await
    .map_err(|_| "Rate limit exceeded")?;
```

### OCR Integration
- Use `tesseract` crate for OCR processing
- Detect if PDF contains images/scanned content
- Fall back to OCR when text extraction yields minimal results
- Process images page by page for memory efficiency

### API Key Validation
```rust
// Load from environment at startup
static VALID_KEYS: Lazy<HashSet<String>> = Lazy::new(|| {
    env::var("VALID_API_KEYS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .collect()
});
```

## Future Enhancements

1. **Advanced Features**
   - Table extraction
   - Multi-language OCR support
   - Image extraction
   - Document structure analysis

2. **Scalability**
   - Horizontal scaling support
   - Queue-based processing
   - Caching layer (file hash-based)
   - Database integration for analytics

3. **API Improvements**
   - Webhook support
   - Batch processing
   - Progress tracking for large files
   - Custom output formats

This specification provides a comprehensive foundation for building a production-ready PDF extraction service in Rust that matches the functionality of LlamaParse while maintaining high performance and security standards.