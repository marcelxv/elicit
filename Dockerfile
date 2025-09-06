FROM rust:1.75 as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY src ./src

# Build the application (only rebuilds if source changed)
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including Tesseract OCR and curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    tesseract-ocr \
    tesseract-ocr-eng \
    libtesseract-dev \
    libleptonica-dev \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy the binary from builder stage
COPY --from=builder /app/target/release/elicit /usr/local/bin/elicit

# Make binary executable
RUN chmod +x /usr/local/bin/elicit

# Create a non-root user with proper home directory
RUN useradd -r -m -s /bin/false elicit

# Create necessary directories and set permissions
RUN mkdir -p /app/tmp && chown -R elicit:elicit /app

# Switch to non-root user
USER elicit

# Set working directory
WORKDIR /app

# Railway sets PORT environment variable, default to 3000 for local development
ENV PORT=3000
EXPOSE $PORT

# Health check with proper Railway port handling
HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:${PORT}/ready || exit 1

# Run the binary
CMD ["/usr/local/bin/elicit"]