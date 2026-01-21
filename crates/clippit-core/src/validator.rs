use anyhow::{anyhow, Result};
use image::ImageFormat;
use std::io::Cursor;

const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024; // 10MB
const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024; // 50MB

pub struct ContentValidator;

impl ContentValidator {
    pub fn validate_text(text: &str) -> Result<()> {
        if text.is_empty() {
            return Err(anyhow!("Text content is empty"));
        }

        if text.len() > MAX_TEXT_SIZE {
            return Err(anyhow!(
                "Text content exceeds maximum size of {} bytes",
                MAX_TEXT_SIZE
            ));
        }

        // Validate UTF-8 (already done by str type, but double check)
        if !text.is_ascii() && text.chars().any(|c| c == '\u{FFFD}') {
            return Err(anyhow!("Text contains invalid UTF-8 sequences"));
        }

        Ok(())
    }

    pub fn validate_image(data: &[u8]) -> Result<()> {
        if data.is_empty() {
            return Err(anyhow!("Image data is empty"));
        }

        if data.len() > MAX_IMAGE_SIZE {
            return Err(anyhow!(
                "Image data exceeds maximum size of {} bytes",
                MAX_IMAGE_SIZE
            ));
        }

        // Try to decode to validate format
        let cursor = Cursor::new(data);
        let format = image::ImageReader::new(cursor)
            .with_guessed_format()?
            .format();

        match format {
            Some(ImageFormat::Png) | Some(ImageFormat::Jpeg) => Ok(()),
            _ => Err(anyhow!(
                "Unsupported image format. Only PNG and JPEG are supported"
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_text_valid() {
        assert!(ContentValidator::validate_text("Hello, world!").is_ok());
    }

    #[test]
    fn test_validate_text_empty() {
        assert!(ContentValidator::validate_text("").is_err());
    }

    #[test]
    fn test_validate_text_too_large() {
        let large_text = "a".repeat(MAX_TEXT_SIZE + 1);
        assert!(ContentValidator::validate_text(&large_text).is_err());
    }
}
