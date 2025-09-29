use std::time::Instant;
use pdf_extract::extract_text;
use std::io::Write;
use tempfile::NamedTempFile;
use lopdf::Document;

use crate::error::{AppError, AppResult};
use crate::models::{ProcessedFile, PdfMetadata};
use crate::services::ocr_service::OcrService;

pub struct PdfProcessor;

#[derive(Debug)]
pub struct ExtractionResult {
    pub text: String,
    pub pages: usize,
    pub metadata: PdfMetadata,
    pub processing_time_ms: u64,
}

impl PdfProcessor {
    pub fn new() -> Self {
        Self
    }

    pub async fn extract_text(&self, file: ProcessedFile) -> AppResult<ExtractionResult> {
        let start = Instant::now();
        
        tracing::info!(
            "Starting PDF text extraction for file: {} ({} bytes)",
            file.name,
            file.size
        );

        // Validate file is PDF
        if !file.is_pdf() {
            return Err(AppError::InvalidFile {
                message: "File is not a valid PDF".to_string(),
            });
        }

        // Validate file size (already checked by middleware, but double-check)
        let config = crate::config::Config::from_env()
            .map_err(|e| AppError::config(format!("Failed to load config: {}", e)))?;
        let max_size_bytes = config.max_file_size_mb * 1024 * 1024;
        if file.content.len() > max_size_bytes {
            return Err(AppError::FileTooLarge {
                size: file.content.len() / (1024 * 1024),
                limit: config.max_file_size_mb,
            });
        }

        // Validate PDF structure early
        if let Err(e) = Document::load_mem(&file.content) {
            tracing::warn!("PDF structure validation failed: {}, will try text extraction anyway", e);
        }

        // Write PDF content to temporary file for pdf-extract
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| AppError::ProcessingError {
                message: format!("Failed to create temporary file: {}", e)
            })?;
        
        temp_file.write_all(&file.content)
            .map_err(|e| AppError::ProcessingError {
                message: format!("Failed to write PDF to temporary file: {}", e)
            })?;
        
        // Try to extract text using pdf-extract
        let extracted_text = match extract_text(temp_file.path()) {
            Ok(text) => {
                tracing::debug!("PDF text extraction successful, {} characters", text.len());
                text
            }
            Err(e) => {
                tracing::warn!("PDF text extraction failed: {}, trying OCR fallback", e);
                
                // Fallback to OCR if direct text extraction fails
                let ocr_service = OcrService::new()?;
                match ocr_service.extract_text_from_pdf(&file.content).await {
                    Ok(ocr_text) => {
                        tracing::info!("OCR extraction successful, {} characters", ocr_text.len());
                        ocr_text
                    }
                    Err(ocr_err) => {
                        tracing::error!("Both PDF extraction and OCR failed: {}", ocr_err);
                        return Err(AppError::ProcessingError {
                            message: format!("PDF extraction failed: {}, OCR failed: {}", e, ocr_err),
                        });
                    }
                }
            }
        };

        // Check if we got meaningful text
        let cleaned_text = extracted_text.trim();
        if cleaned_text.is_empty() {
            tracing::warn!("No text extracted from PDF, trying OCR");

            let ocr_service = OcrService::new()?;
            match ocr_service.extract_text_from_pdf(&file.content).await {
                Ok(ocr_text) => {
                    let processing_time = start.elapsed().as_millis() as u64;

                    return Ok(ExtractionResult {
                        text: ocr_text,
                        pages: self.estimate_pages(&file.content),
                        metadata: PdfMetadata::new(file.size).with_ocr(),
                        processing_time_ms: processing_time,
                    });
                }
                Err(ocr_err) => {
                    // OCR failed - check the reason
                    let err_msg = ocr_err.to_string();

                    if err_msg.contains("does not appear to contain scanned content") {
                        tracing::info!("PDF has no extractable text and is not a scanned document");

                        // Return empty result with metadata instead of error
                        let processing_time = start.elapsed().as_millis() as u64;

                        return Ok(ExtractionResult {
                            text: String::new(),
                            pages: self.estimate_pages(&file.content),
                            metadata: PdfMetadata::new(file.size)
                                .with_title(self.extract_title(&file.content))
                                .with_author(self.extract_author(&file.content)),
                            processing_time_ms: processing_time,
                        });
                    } else if err_msg.contains("Tesseract is not installed") {
                        // PDF needs OCR but Tesseract is not available
                        tracing::warn!("PDF requires OCR but Tesseract is not installed");

                        return Err(AppError::ProcessingError {
                            message: format!("This PDF appears to be scanned and requires OCR. {}", ocr_err),
                        });
                    } else {
                        return Err(AppError::ProcessingError {
                            message: format!("No text found and OCR failed: {}", ocr_err),
                        });
                    }
                }
            }
        }

        // If text is very short, it might be a scanned PDF - try OCR as well
        let use_ocr = cleaned_text.len() < 100 || 
                      cleaned_text.chars().filter(|c| c.is_alphabetic()).count() < 50;
        
        let final_text = if use_ocr {
            tracing::info!("Text extraction yielded minimal results, trying OCR enhancement");
            
            let ocr_service = OcrService::new()?;
            match ocr_service.extract_text_from_pdf(&file.content).await {
                Ok(ocr_text) => {
                    if ocr_text.len() > cleaned_text.len() {
                        tracing::info!("OCR provided better results, using OCR text");
                        ocr_text
                    } else {
                        cleaned_text.to_string()
                    }
                }
                Err(_) => {
                    tracing::debug!("OCR enhancement failed, using original text");
                    cleaned_text.to_string()
                }
            }
        } else {
            cleaned_text.to_string()
        };

        let processing_time = start.elapsed().as_millis() as u64;
        
        tracing::info!(
            "PDF processing completed in {}ms, extracted {} characters",
            processing_time,
            final_text.len()
        );

        Ok(ExtractionResult {
            text: final_text,
            pages: self.estimate_pages(&file.content),
            metadata: PdfMetadata::new(file.size)
                .with_title(self.extract_title(&file.content))
                .with_author(self.extract_author(&file.content)),
            processing_time_ms: processing_time,
        })
    }

    fn estimate_pages(&self, pdf_content: &[u8]) -> usize {
        match Document::load_mem(pdf_content) {
            Ok(doc) => doc.get_pages().len(),
            Err(_) => {
                // Fallback to size-based estimation
                let size_kb = pdf_content.len() / 1024;
                std::cmp::max(1, size_kb / 50)
            }
        }
    }

    fn extract_title(&self, pdf_content: &[u8]) -> Option<String> {
        match Document::load_mem(pdf_content) {
            Ok(doc) => {
                if let Ok(info_dict) = doc.trailer.get(b"Info") {
                    if let Ok(info) = doc.get_object(info_dict.as_reference().ok()?) {
                        if let Ok(title_obj) = info.as_dict().ok()?.get(b"Title") {
                            if let Ok(title_bytes) = title_obj.as_str() {
                                // Handle UTF-16 encoded strings (common in PDFs)
                                let title_string = if title_bytes.starts_with(&[0xFE, 0xFF]) {
                                    // UTF-16 BE with BOM
                                    decode_utf16_be(&title_bytes[2..])
                                } else if title_bytes.starts_with(&[0xFF, 0xFE]) {
                                    // UTF-16 LE with BOM
                                    decode_utf16_le(&title_bytes[2..])
                                } else if looks_like_utf16(title_bytes) {
                                    // UTF-16 without BOM (try BE first)
                                    decode_utf16_be(title_bytes)
                                } else {
                                    // Regular UTF-8 or ASCII
                                    String::from_utf8_lossy(title_bytes).to_string()
                                };

                                let trimmed = title_string.trim();
                                if !trimmed.is_empty() {
                                    return Some(trimmed.to_string());
                                }
                            }
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    fn extract_author(&self, pdf_content: &[u8]) -> Option<String> {
        match Document::load_mem(pdf_content) {
            Ok(doc) => {
                if let Ok(info_dict) = doc.trailer.get(b"Info") {
                    if let Ok(info) = doc.get_object(info_dict.as_reference().ok()?) {
                        if let Ok(author_obj) = info.as_dict().ok()?.get(b"Author") {
                            if let Ok(author_bytes) = author_obj.as_str() {
                                // Handle UTF-16 encoded strings (common in PDFs)
                                let author_string = if author_bytes.starts_with(&[0xFE, 0xFF]) {
                                    // UTF-16 BE with BOM
                                    decode_utf16_be(&author_bytes[2..])
                                } else if author_bytes.starts_with(&[0xFF, 0xFE]) {
                                    // UTF-16 LE with BOM
                                    decode_utf16_le(&author_bytes[2..])
                                } else if looks_like_utf16(author_bytes) {
                                    // UTF-16 without BOM (try BE first)
                                    decode_utf16_be(author_bytes)
                                } else {
                                    // Regular UTF-8 or ASCII
                                    String::from_utf8_lossy(author_bytes).to_string()
                                };

                                let trimmed = author_string.trim();
                                if !trimmed.is_empty() {
                                    return Some(trimmed.to_string());
                                }
                            }
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }
}

// Helper functions for UTF-16 decoding
fn looks_like_utf16(bytes: &[u8]) -> bool {
    // Check if bytes look like UTF-16 (many null bytes in alternating positions)
    if bytes.len() < 2 {
        return false;
    }

    let null_count = bytes.iter().filter(|&&b| b == 0).count();
    // If more than 30% of bytes are null, likely UTF-16
    null_count > bytes.len() / 3
}

fn decode_utf16_be(bytes: &[u8]) -> String {
    let mut chars = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        let high = bytes[i] as u16;
        let low = bytes[i + 1] as u16;
        let code_point = (high << 8) | low;

        if let Some(ch) = char::from_u32(code_point as u32) {
            if ch != '\0' {  // Skip null characters
                chars.push(ch);
            }
        }
        i += 2;
    }

    chars.into_iter().collect()
}

fn decode_utf16_le(bytes: &[u8]) -> String {
    let mut chars = Vec::new();
    let mut i = 0;

    while i + 1 < bytes.len() {
        let low = bytes[i] as u16;
        let high = bytes[i + 1] as u16;
        let code_point = (high << 8) | low;

        if let Some(ch) = char::from_u32(code_point as u32) {
            if ch != '\0' {  // Skip null characters
                chars.push(ch);
            }
        }
        i += 2;
    }

    chars.into_iter().collect()
}

impl PdfProcessor {
    /// Check if the PDF processor is available
    pub fn is_available(&self) -> bool {
        // PDF processor is always available as it uses built-in libraries
        true
    }
}

impl Default for PdfProcessor {
    fn default() -> Self {
        Self::new()
    }
}