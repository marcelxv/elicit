use crate::error::{AppError, AppResult};
use tracing::{info, warn, debug};
use std::time::Instant;
use std::process::Command;
use tempfile::{NamedTempFile, TempDir};
use std::io::Write;

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

        // Check if we can actually run OCR (Tesseract installed)
        if !Self::is_tesseract_available() {
            warn!("Tesseract OCR is not available on this system");
            return Err(AppError::OcrError {
                message: "This PDF appears to be scanned and requires OCR, but Tesseract is not installed. Please install Tesseract OCR to process scanned PDFs.".to_string()
            });
        }

        // Try to perform basic OCR using pdfimages and tesseract
        let ocr_result = self.perform_ocr_on_pdf(pdf_data).await;

        let processing_time = start.elapsed().as_millis();

        match ocr_result {
            Ok(text) => {
                info!("OCR extraction completed successfully ({}ms), extracted {} characters", processing_time, text.len());
                Ok(text)
            }
            Err(e) => {
                warn!("OCR extraction failed after {}ms: {}", processing_time, e);
                // Return empty string to allow processing to continue
                Ok(String::new())
            }
        }
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
    async fn perform_ocr_on_pdf(&self, pdf_data: &[u8]) -> AppResult<String> {
        // Create a temporary file for the PDF
        let mut pdf_file = NamedTempFile::new()
            .map_err(|e| AppError::OcrError {
                message: format!("Failed to create temp file: {}", e)
            })?;

        pdf_file.write_all(pdf_data)
            .map_err(|e| AppError::OcrError {
                message: format!("Failed to write PDF to temp file: {}", e)
            })?;

        let pdf_path = pdf_file.path();

        // Create temp directory for extracted images
        let temp_dir = TempDir::new()
            .map_err(|e| AppError::OcrError {
                message: format!("Failed to create temp directory: {}", e)
            })?;

        // First, try to extract images from PDF using pdfimages
        let image_prefix = temp_dir.path().join("page");
        let extract_result = Command::new("pdfimages")
            .arg("-j") // Extract as JPEG
            .arg("-png") // Also extract PNG images
            .arg(pdf_path)
            .arg(&image_prefix)
            .output();

        if extract_result.is_err() {
            // If pdfimages is not available, try converting with ImageMagick
            debug!("pdfimages not available, trying ImageMagick convert");

            let convert_result = Command::new("convert")
                .arg("-density").arg("150")
                .arg(pdf_path)
                .arg("-quality").arg("100")
                .arg(temp_dir.path().join("page-%03d.png").to_str().unwrap())
                .output();

            if convert_result.is_err() {
                return Err(AppError::OcrError {
                    message: "Neither pdfimages nor ImageMagick are available to extract images from PDF".to_string()
                });
            }
        }

        // Find all extracted images
        let mut extracted_text = String::new();
        let entries = std::fs::read_dir(temp_dir.path())
            .map_err(|e| AppError::OcrError {
                message: format!("Failed to read temp directory: {}", e)
            })?;

        let mut page_count = 0;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()).map(|s| s == "png" || s == "jpg" || s == "jpeg").unwrap_or(false) {
                    page_count += 1;
                    debug!("Processing page {}: {:?}", page_count, path);

                    // Run Tesseract on each image
                    let output = Command::new("tesseract")
                        .arg(&path)
                        .arg("-") // Output to stdout
                        .arg("-l").arg("spa+eng") // Spanish and English
                        .arg("--psm").arg("1") // Auto page segmentation with OSD
                        .output();

                    if let Ok(output) = output {
                        if output.status.success() {
                            let text = String::from_utf8_lossy(&output.stdout);
                            extracted_text.push_str(&text);
                            extracted_text.push_str("\n\n");
                        }
                    }
                }
            }
        }

        if page_count == 0 {
            return Err(AppError::OcrError {
                message: "No images could be extracted from the PDF".to_string()
            });
        }

        info!("OCR processed {} pages", page_count);
        Ok(extracted_text.trim().to_string())
    }

    fn is_likely_scanned_pdf(pdf_data: &[u8]) -> bool {
        // Enhanced heuristic to detect scanned PDFs
        let pdf_str = String::from_utf8_lossy(pdf_data);

        // Check for CamScanner or other scanning app signatures
        let scan_indicators = [
            "CamScanner",
            "Adobe Scan",
            "TinyScanner",
            "Scanner Pro",
            "Genius Scan",
        ];

        let has_scan_app = scan_indicators.iter()
            .any(|indicator| pdf_str.contains(indicator));

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

        debug!("PDF analysis: {} image markers, {} text markers, scan app: {}",
               image_count, text_count, has_scan_app);

        // Multiple conditions for scanned PDF detection:
        // 1. Created by scanning apps (CamScanner, etc.)
        // 2. High image to text ratio
        // 3. Many images with few fonts
        has_scan_app ||
        (image_count > 0 && text_count == 0) ||
        (image_count > 10 && image_count >= text_count) ||
        (image_count > text_count * 2)
    }
}