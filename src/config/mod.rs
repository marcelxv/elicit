use std::collections::HashSet;
use std::env;
use anyhow::{Result, Context};
use once_cell::sync::Lazy;
use tracing::{info, warn, error};

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub max_file_size_mb: usize,
    pub max_concurrent_requests: usize,
    pub request_timeout_seconds: u64,
    pub worker_threads: usize,
}

// Global API keys loaded from environment
pub static VALID_API_KEYS: Lazy<HashSet<String>> = Lazy::new(|| {
    env::var("VALID_API_KEYS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
});

impl Config {
    pub fn from_env() -> Result<Self> {
        info!("Loading configuration from environment variables");
        
        let config = Config {
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| {
                info!("SERVER_HOST not set, using default: 0.0.0.0");
                "0.0.0.0".to_string()
            }),
            server_port: Self::parse_env_var("SERVER_PORT", 8080)
                .context("Failed to parse SERVER_PORT")?,
            max_file_size_mb: Self::parse_env_var("MAX_FILE_SIZE_MB", 10)
                .context("Failed to parse MAX_FILE_SIZE_MB")?,
            max_concurrent_requests: Self::parse_env_var("MAX_CONCURRENT_REQUESTS", 100)
                .context("Failed to parse MAX_CONCURRENT_REQUESTS")?,
            request_timeout_seconds: Self::parse_env_var("REQUEST_TIMEOUT_SECONDS", 30)
                .context("Failed to parse REQUEST_TIMEOUT_SECONDS")?,
            worker_threads: Self::parse_env_var("WORKER_THREADS", 4)
                .context("Failed to parse WORKER_THREADS")?,
        };
        
        // Validate configuration values
        config.validate()?;

        // Validate that we have at least one API key
        if VALID_API_KEYS.is_empty() {
            warn!("No valid API keys configured. Set VALID_API_KEYS environment variable.");
        } else {
            info!("Loaded {} valid API keys", VALID_API_KEYS.len());
        }

        info!("Configuration loaded successfully: {:?}", config);
        Ok(config)
    }
    
    fn parse_env_var<T>(var_name: &str, default: T) -> Result<T>
    where
        T: std::str::FromStr + Copy + std::fmt::Debug,
        T::Err: std::fmt::Display,
    {
        match env::var(var_name) {
            Ok(val) => match val.parse() {
                Ok(parsed) => Ok(parsed),
                Err(e) => {
                    warn!("Failed to parse {}: {} (using default: {:?})", var_name, e, default);
                    Ok(default)
                }
            },
            Err(_) => {
                info!("{} not set, using default: {:?}", var_name, default);
                Ok(default)
            }
        }
    }
    
    fn validate(&self) -> Result<()> {
        if self.server_port == 0 {
            return Err(anyhow::anyhow!("SERVER_PORT must be greater than 0"));
        }
        if self.max_file_size_mb == 0 {
            return Err(anyhow::anyhow!("MAX_FILE_SIZE_MB must be greater than 0"));
        }
        if self.max_concurrent_requests == 0 {
            return Err(anyhow::anyhow!("MAX_CONCURRENT_REQUESTS must be greater than 0"));
        }
        if self.request_timeout_seconds == 0 {
            return Err(anyhow::anyhow!("REQUEST_TIMEOUT_SECONDS must be greater than 0"));
        }
        if self.worker_threads == 0 {
            return Err(anyhow::anyhow!("WORKER_THREADS must be greater than 0"));
        }
        Ok(())
    }

    pub fn validate_api_key(key: &str) -> bool {
        VALID_API_KEYS.contains(key)
    }
}