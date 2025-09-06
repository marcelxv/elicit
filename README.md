# Elicit - PDF Text Extractor Service

A high-performance Rust-based HTTP service for extracting text from PDF documents with OCR support for scanned documents. Built for fast, reliable PDF processing with Railway deployment support.

## Features

- **Fast PDF Text Extraction**: Uses `pdf-extract` crate for efficient text extraction
- **OCR Support**: Tesseract OCR fallback for scanned PDFs
- **High Performance**: Handles 100+ concurrent requests
- **API Key Authentication**: Secure Bearer token authentication
- **Rate Limiting**: Global concurrent request limiting
- **Railway Ready**: Optimized for Railway deployment
- **Comprehensive Logging**: Structured logging with tracing
- **Health Monitoring**: Built-in health check endpoint

## Quick Start

### Prerequisites

- Rust 1.75+
- Tesseract OCR (for scanned PDF support)

#### Install Tesseract (macOS)
```bash
brew install tesseract
```

#### Install Tesseract (Ubuntu/Debian)
```bash
sudo apt-get update
sudo apt-get install tesseract-ocr tesseract-ocr-eng libtesseract-dev libleptonica-dev
```

### Local Development

1. **Clone and setup**:
```bash
git clone <repository-url>
cd elicit
cp .env.example .env
```

2. **Configure environment variables**:
```bash
# Edit .env file
VALID_API_KEYS=your-secret-key-1,your-secret-key-2
MAX_FILE_SIZE_MB=10
MAX_CONCURRENT_REQUESTS=100
```

3. **Run the service**:
```bash
cargo run
```

The service will start on `http://localhost:8080`

### Docker Development

```bash
# Build the image
docker build -t elicit .

# Run with environment variables
docker run -p 8080:8080 \
  -e VALID_API_KEYS="your-key-1,your-key-2" \
  -e MAX_FILE_SIZE_MB=10 \
  -e MAX_CONCURRENT_REQUESTS=100 \
  elicit
```

## API Usage

### Extract Text from PDF

**Endpoint**: `POST /api/v1/extract`

**Headers**:
```
Authorization: Bearer your-api-key
Content-Type: multipart/form-data
```

**Request Body**: Multipart form with `file` field containing PDF

**Example with curl**:
```bash
curl -X POST http://localhost:8080/api/v1/extract \
  -H "Authorization: Bearer your-api-key" \
  -F "file=@document.pdf"
```

**Success Response (200)**:
```json
{
  "success": true,
  "data": {
    "text": "Extracted text content from the PDF...",
    "pages": 5,
    "metadata": {
      "title": null,
      "author": null,
      "creation_date": null,
      "modification_date": null,
      "file_size_bytes": 1048576,
      "ocr_used": false
    }
  },
  "processing_time_ms": 1250
}
```

**Error Responses**:

- `400 Bad Request`: Invalid file or missing file
- `401 Unauthorized`: Invalid or missing API key
- `413 Payload Too Large`: File exceeds 10MB limit
- `429 Too Many Requests`: Concurrent request limit exceeded
- `500 Internal Server Error`: Processing failed

### Health Check

**Endpoint**: `GET /health`

```bash
curl http://localhost:8080/health
```

**Response**:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "service": "elicit-pdf-extractor"
}
```

## Railway Deployment

### 1. Prepare for Railway

1. **Connect your repository** to Railway
2. **Set environment variables** in Railway dashboard:
   ```
   VALID_API_KEYS=your-production-key-1,your-production-key-2
   MAX_FILE_SIZE_MB=10
   MAX_CONCURRENT_REQUESTS=100
   RUST_LOG=info
   ```

### 2. Deploy

Railway will automatically:
- Build using the Dockerfile
- Set the `PORT` environment variable
- Handle HTTPS termination
- Provide a public URL

### 3. Verify Deployment

```bash
# Check health
curl https://your-app.railway.app/health

# Test extraction
curl -X POST https://your-app.railway.app/api/v1/extract \
  -H "Authorization: Bearer your-production-key" \
  -F "file=@test.pdf"
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | Server bind address |
| `SERVER_PORT` | `8080` | Server port (Railway sets `PORT`) |
| `MAX_FILE_SIZE_MB` | `10` | Maximum file size in MB |
| `MAX_CONCURRENT_REQUESTS` | `100` | Global concurrent request limit |
| `VALID_API_KEYS` | - | Comma-separated API keys |
| `REQUEST_TIMEOUT_SECONDS` | `30` | Request timeout |
| `WORKER_THREADS` | `4` | Tokio worker threads |
| `RUST_LOG` | `info` | Log level |

## Performance

- **Throughput**: 100+ concurrent requests
- **Latency**: < 5 seconds for typical PDFs (< 10MB)
- **Memory**: < 512MB per request
- **File Size**: Up to 10MB PDFs
- **OCR Fallback**: Automatic for scanned documents

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   HTTP Client   │───▶│   Axum Server    │───▶│  PDF Processor  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │  Auth Middleware │    │   OCR Service   │
                       └──────────────────┘    └─────────────────┘
                                │                        │
                                ▼                        ▼
                       ┌──────────────────┐    ┌─────────────────┐
                       │ Rate Limiting    │    │   Tesseract     │
                       └──────────────────┘    └─────────────────┘
```

## Development

### Project Structure

```
elicit/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library root
│   ├── config/              # Configuration management
│   ├── handlers/            # HTTP request handlers
│   ├── middleware/          # Auth, rate limiting, logging
│   ├── services/            # PDF processing, OCR
│   ├── models/              # Request/response models
│   └── error/               # Error handling
├── Cargo.toml               # Dependencies
├── Dockerfile               # Railway deployment
├── .env.example             # Environment template
└── README.md                # This file
```

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration

# With logging
RUST_LOG=debug cargo test
```

### Adding Features

1. **New endpoints**: Add to `src/handlers/`
2. **Middleware**: Add to `src/middleware/`
3. **Services**: Add to `src/services/`
4. **Models**: Add to `src/models/`

## Troubleshooting

### Common Issues

1. **Tesseract not found**:
   ```bash
   # Install Tesseract OCR
   brew install tesseract  # macOS
   sudo apt install tesseract-ocr  # Ubuntu
   ```

2. **Rate limit errors**:
   - Check `MAX_CONCURRENT_REQUESTS` setting
   - Monitor concurrent request usage

3. **File size errors**:
   - Verify `MAX_FILE_SIZE_MB` configuration
   - Check actual file sizes

4. **OCR failures**:
   - Ensure Tesseract is properly installed
   - Check PDF contains scannable images

### Logging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

Structured JSON logging:
```bash
LOG_FORMAT=json RUST_LOG=info cargo run
```

## Security

- **API Key Authentication**: All endpoints except `/health` require valid API keys
- **Rate Limiting**: Global concurrent request limiting prevents abuse
- **File Validation**: Strict PDF validation and size limits
- **Memory Safety**: Rust's memory safety prevents common vulnerabilities
- **No Sensitive Data**: Error messages don't expose sensitive information

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## Support

For issues and questions:
- Create an issue in the repository
- Check the troubleshooting section
- Review the logs for error details