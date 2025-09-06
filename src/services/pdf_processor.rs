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
        let max_size_bytes = 10 * 1024 * 1024; // 10MB
        if file.content.len() > max_size_bytes {
            return Err(AppError::FileTooLarge {
                size: file.content.len() / (1024 * 1024),
                limit: 10,
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
            let ocr_text = ocr_service.extract_text_from_pdf(&file.content).await
                .map_err(|e| AppError::ProcessingError {
                    message: format!("No text found and OCR failed: {}", e),
                })?;
            
            let processing_time = start.elapsed().as_millis() as u64;
            
            return Ok(ExtractionResult {
                text: ocr_text,
                pages: self.estimate_pages(&file.content),
                metadata: PdfMetadata::new(file.size).with_ocr(),
                processing_time_ms: processing_time,
            });
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
                            if let Ok(title_str) = title_obj.as_str() {
                                return Some(String::from_utf8_lossy(title_str).trim().to_string());
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
                            if let Ok(author_str) = author_obj.as_str() {
                                return Some(String::from_utf8_lossy(author_str).trim().to_string());
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