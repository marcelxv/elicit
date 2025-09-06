use crate::error::{AppError, AppResult};
use tracing::{info, warn, error, debug};
use std::path::Path;
use std::time::Instant;

pub struct OcrService;

impl OcrService {
    pub fn new() -> AppResult<Self> {
        // Check if Tesseract is available
        if !Self::is_tesseract_available() {
            return Err(AppError::OcrError {
                message: "Tesseract OCR not available on this system".to_string()
            });
        }
        
        Ok(Self)
    }

    pub async fn extract_text_from_pdf(&self, pdf_data: &[u8]) -> AppResult<String> {
        let start = Instant::now();
        info!("Starting OCR extraction from PDF ({} bytes)", pdf_data.len());
        
        // Check if PDF is likely to contain scanned content
        let is_scanned = Self::is_likely_scanned_pdf(pdf_data);
        debug!("PDF scanned content detection: {}", is_scanned);
        
        if !is_scanned {
            warn!("PDF does not appear to contain scanned images, OCR may not be necessary");
            return Err(AppError::OcrError {
                message: "PDF does not appear to contain scanned content that requires OCR".to_string()
            });
        }
        
        info!("PDF appears to contain scanned content, OCR would be beneficial");
        
        // For production implementation, you would:
        // 1. Use pdf2image or similar to convert PDF pages to images
        // 2. Run Tesseract OCR on each image
        // 3. Combine results from all pages
        // 4. Apply post-processing (spell check, formatting)
        
        let processing_time = start.elapsed().as_millis();
        warn!("OCR extraction not fully implemented yet (took {}ms to analyze)", processing_time);
        
        Err(AppError::OcrError {
            message: "OCR extraction from PDF requires additional image conversion libraries (pdf2image, pillow). This is a placeholder implementation.".to_string()
        })
    }

    pub async fn extract_text_from_image(&self, _image_data: &[u8]) -> AppResult<String> {
        // For now, return a placeholder since we need proper image conversion libraries
        // In a production environment, you would:
        // 1. Convert PDF pages to images using pdf2image or similar
        // 2. Use Tesseract OCR to extract text from images
        // 3. Combine results from all pages
        
        warn!("OCR from image data not yet implemented - requires image conversion libraries");
        Err(AppError::OcrError {
            message: "OCR from image data requires additional image conversion libraries".to_string()
        })
    }

    pub fn is_tesseract_available() -> bool {
        // Check if tesseract command is available in PATH
        std::process::Command::new("tesseract")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    pub fn is_available() -> bool {
        Self::is_tesseract_available()
    }
}

impl Default for OcrService {
    fn default() -> Self {
        Self::new().unwrap_or(Self)
    }
}

impl OcrService {
    fn is_likely_scanned_pdf(pdf_data: &[u8]) -> bool {
        // Enhanced heuristic to detect scanned PDFs
        let pdf_str = String::from_utf8_lossy(pdf_data);
        
        // Count image-related markers
        let image_markers = [
            "/Image",
            "/DCTDecode",  // JPEG compression
            "/CCITTFaxDecode", // Fax/scan compression
            "/JBIG2Decode", // JBIG2 compression (common in scans)
            "/JPXDecode",  // JPEG2000
        ];
        
        let image_count = image_markers.iter()
            .map(|marker| pdf_str.matches(marker).count())
            .sum::<usize>();
        
        // Count text-related markers
        let text_markers = [
            "/Font",
            "/Text",
            "BT", // Begin text
            "ET", // End text
        ];
        
        let text_count = text_markers.iter()
            .map(|marker| pdf_str.matches(marker).count())
            .sum::<usize>();
        
        debug!("PDF analysis: {} image markers, {} text markers", image_count, text_count);
        
        // If we have significantly more image markers than text markers,
        // it's likely a scanned PDF
        image_count > 0 && (text_count == 0 || image_count > text_count * 2)
    }
}