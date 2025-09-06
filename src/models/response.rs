use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractResponse {
    pub success: bool,
    pub data: ExtractData,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractData {
    pub text: String,
    pub pages: usize,
    pub metadata: PdfMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PdfMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub modification_date: Option<DateTime<Utc>>,
    pub file_size_bytes: usize,
    pub ocr_used: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub service: String,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
}

impl ExtractResponse {
    pub fn new(text: String, pages: usize, metadata: PdfMetadata, processing_time_ms: u64) -> Self {
        Self {
            success: true,
            data: ExtractData {
                text,
                pages,
                metadata,
            },
            processing_time_ms,
        }
    }
}

impl PdfMetadata {
    pub fn new(file_size_bytes: usize) -> Self {
        Self {
            title: None,
            author: None,
            creation_date: None,
            modification_date: None,
            file_size_bytes,
            ocr_used: false,
        }
    }

    pub fn with_ocr(mut self) -> Self {
        self.ocr_used = true;
        self
    }

    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    pub fn with_author(mut self, author: Option<String>) -> Self {
        self.author = author;
        self
    }

    pub fn with_dates(
        mut self,
        creation_date: Option<DateTime<Utc>>,
        modification_date: Option<DateTime<Utc>>,
    ) -> Self {
        self.creation_date = creation_date;
        self.modification_date = modification_date;
        self
    }
}