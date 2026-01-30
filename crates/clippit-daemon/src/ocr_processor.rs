use anyhow::Result;
use clippit_core::storage::Storage;
use std::path::Path;
use tesseract::Tesseract;
use tracing::{error, info, warn};

pub struct OCRProcessor {
    languages: String, // "por+eng"
}

impl OCRProcessor {
    pub fn new(languages: &str) -> Self {
        Self {
            languages: languages.to_string(),
        }
    }

    /// Processa OCR em imagem e retorna texto extraído
    pub fn process_image(&self, image_path: &Path) -> Result<Option<String>> {
        info!("🔍 Starting OCR for: {:?}", image_path);

        // Verificar se arquivo existe
        if !image_path.exists() {
            warn!("⚠️ Image file not found: {:?}", image_path);
            return Ok(None);
        }

        // Inicializar Tesseract
        let mut tesseract = match Tesseract::new(None, Some(&self.languages)) {
            Ok(t) => t,
            Err(e) => {
                error!("❌ Failed to initialize Tesseract: {}", e);
                return Err(e.into());
            }
        };

        // Configurar imagem
        tesseract = match tesseract.set_image(image_path.to_str().unwrap()) {
            Ok(t) => t,
            Err(e) => {
                error!("❌ Failed to set image: {}", e);
                return Err(e.into());
            }
        };

        // Extrair texto
        match tesseract.get_text() {
            Ok(text) => {
                let trimmed = text.trim();
                if trimmed.is_empty() {
                    info!("ℹ️ No text found in image");
                    Ok(None)
                } else {
                    info!("✅ OCR extracted {} characters", trimmed.len());
                    Ok(Some(trimmed.to_string()))
                }
            }
            Err(e) => {
                error!("❌ OCR failed: {}", e);
                Err(e.into())
            }
        }
    }
}

/// Processa OCR para uma entrada do histórico (função assíncrona)
pub async fn process_ocr_for_entry(
    entry_id: i64,
    image_path: String,
    languages: String,
    db_path: String,
) {
    // Spawn blocking para não bloquear runtime async
    let result = tokio::task::spawn_blocking(move || {
        let processor = OCRProcessor::new(&languages);
        processor.process_image(Path::new(&image_path))
    })
    .await;

    match result {
        Ok(Ok(Some(ocr_text))) => {
            // Criar conexão própria para write (WAL permite múltiplas conexões)
            match Storage::new(&db_path) {
                Ok(storage) => {
                    if let Err(e) = storage.update_ocr_text(entry_id, &ocr_text) {
                        error!("❌ Failed to update OCR text in database: {}", e);
                    } else {
                        info!("✅ OCR text saved for entry {}", entry_id);
                    }
                }
                Err(e) => {
                    error!("❌ Failed to open database for OCR update: {}", e);
                }
            }
        }
        Ok(Ok(None)) => {
            info!("ℹ️ No text found in image {}", entry_id);
        }
        Ok(Err(e)) => {
            error!("❌ OCR processing error for entry {}: {}", entry_id, e);
        }
        Err(e) => {
            error!("❌ Tokio task error: {}", e);
        }
    }
}
