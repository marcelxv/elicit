use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ExtractRequest {
    pub file_name: Option<String>,
    pub file_size: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub name: String,
    pub size: usize,
    pub content: Vec<u8>,
    pub mime_type: Option<String>,
}

impl ProcessedFile {
    pub fn new(name: String, content: Vec<u8>) -> Self {
        let size = content.len();
        Self {
            name,
            size,
            content,
            mime_type: None,
        }
    }

    pub fn with_mime_type(mut self, mime_type: String) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    pub fn is_pdf(&self) -> bool {
        self.mime_type
            .as_ref()
            .map(|mt| mt == "application/pdf")
            .unwrap_or_else(|| {
                self.name.to_lowercase().ends_with(".pdf")
                    || self.content.starts_with(b"%PDF")
            })
    }
}