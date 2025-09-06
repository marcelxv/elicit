//! Elicit PDF Extractor Service
//! 
//! A high-performance Rust service for extracting text from PDF documents
//! with OCR support for scanned documents.

pub mod config;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;

pub use config::Config;
pub use error::{AppError, AppResult};