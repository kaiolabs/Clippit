use anyhow::Result;
use arboard::{Clipboard, ImageData};
use clippit_core::{ClipboardEntry, Config, HistoryManager};
use dirs;
use image::{self, DynamicImage, ImageFormat};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

pub async fn start_monitor(history_manager: Arc<Mutex<HistoryManager>>) -> Result<()> {
    info!("Starting clipboard monitor (Wayland-native with arboard)...");

    let mut clipboard = Clipboard::new()?;
    let mut last_text: Option<String> = None;
    let mut last_image_hash: Option<String> = None;

    loop {
        // Load config for each iteration (to respect runtime changes)
        let config = Config::load().unwrap_or_default();
        
        // Try to get text first
        match clipboard.get_text() {
            Ok(text) => {
                // Check if content changed
                if last_text.as_ref() != Some(&text) {
                    info!("Clipboard text changed, saving to history");

                    let entry = ClipboardEntry::new_text(text.clone());
                    let mut manager = history_manager.lock().unwrap();

                    match manager.add_entry(entry) {
                        Ok(Some(id)) => {
                            info!("Saved text entry with id {}", id);
                            last_text = Some(text);
                        }
                        Ok(None) => {
                            // Duplicate, skip
                        }
                        Err(e) => {
                            error!("Failed to save entry: {}", e);
                        }
                    }
                }
            }
            Err(_) => {
                // No text in clipboard or error reading - this is normal
            }
        }
        
        // Try to get image if enabled
        if config.privacy.enable_image_capture {
            match clipboard.get_image() {
                Ok(img_data) => {
                    // Convert arboard::ImageData to Vec<u8> (PNG format)
                    match convert_image_data_to_png(&img_data) {
                        Ok(png_bytes) => {
                            // Validate size (convert MB to bytes)
                            let max_size_bytes = (config.privacy.max_image_size_mb as usize) * 1024 * 1024;
                            
                            if png_bytes.len() <= max_size_bytes {
                                // Compute hash to avoid duplicates
                                use sha2::{Digest, Sha256};
                                let mut hasher = Sha256::new();
                                hasher.update(&png_bytes);
                                let current_hash = format!("{:x}", hasher.finalize());
                                
                                info!("🔍 Image hash comparison:");
                                info!("   Current hash: {}...", &current_hash[..12]);
                                info!("   Last hash: {:?}", last_image_hash.as_ref().map(|h| &h[..12]));
                                info!("   Are different? {}", last_image_hash.as_ref() != Some(&current_hash));
                                
                                // Only save if different from last image
                                if last_image_hash.as_ref() != Some(&current_hash) {
                                    info!("📸 New image detected, optimizing and saving...");
                                    
                                    // Optimize if needed (max 2048px)
                                    match optimize_image(png_bytes.clone(), 2048) {
                                        Ok(optimized) => {
                                            // Generate thumbnail (128x128)
                                            let thumbnail = create_thumbnail(&optimized, 128).ok();
                                            
                                            // Save image to file
                                            match save_image_to_file(&optimized, &current_hash) {
                                                Ok(image_path) => {
                                                    info!("💾 Saved image to: {}", image_path);
                                                    
                                                    let entry = ClipboardEntry::new_image(image_path, thumbnail);
                                                    let mut manager = history_manager.lock().unwrap();
                                                    
                                                    match manager.add_entry(entry) {
                                                        Ok(Some(id)) => {
                                                            info!("✅ Saved image entry with id {} (with thumbnail)", id);
                                                            last_image_hash = Some(current_hash.clone());
                                                        }
                                                        Ok(None) => {
                                                            info!("⏭️  Image duplicate, skipped");
                                                            // Update hash even for duplicates to avoid loop
                                                            last_image_hash = Some(current_hash.clone());
                                                        }
                                                        Err(e) => {
                                                            error!("❌ Failed to save image entry: {}", e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("❌ Failed to save image file: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ Failed to optimize image: {}", e);
                                        }
                                    }
                                }
                            } else {
                                warn!("⚠️  Image too large ({} MB > {} MB), skipping", 
                                    png_bytes.len() / 1024 / 1024, 
                                    config.privacy.max_image_size_mb);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to convert image data to PNG: {}", e);
                        }
                    }
                }
                Err(_) => {
                    // No image in clipboard - this is normal
                }
            }
        }

        // Polling interval (80ms for good responsiveness)
        sleep(Duration::from_millis(80)).await;
    }
}

/// Convert arboard ImageData to PNG bytes
fn convert_image_data_to_png(img_data: &ImageData) -> Result<Vec<u8>> {
    // Create image from raw RGBA data
    let img = image::RgbaImage::from_raw(
        img_data.width as u32,
        img_data.height as u32,
        img_data.bytes.to_vec()
    )
    .ok_or_else(|| anyhow::anyhow!("Failed to create image from raw data"))?;
    
    let dynamic_img = DynamicImage::ImageRgba8(img);
    
    // Encode to PNG
    let mut buf = Vec::new();
    dynamic_img.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    
    info!("📸 Converted clipboard image to PNG ({} bytes)", buf.len());
    Ok(buf)
}

fn optimize_image(data: Vec<u8>, max_dimension: u32) -> Result<Vec<u8>> {
    // Validate minimum size (avoid processing garbage data)
    if data.len() < 100 {
        return Err(anyhow::anyhow!("Image data too small ({} bytes), likely invalid", data.len()));
    }
    
    // Try to load image with better error messages
    let img = match image::load_from_memory(&data) {
        Ok(img) => img,
        Err(e) => {
            // Log the error type for debugging
            warn!("⚠️ Failed to load image from {} bytes: {}", data.len(), e);
            return Err(anyhow::anyhow!("Invalid image format: {}", e));
        }
    };
    
    let needs_resize = img.width() > max_dimension || img.height() > max_dimension;
    
    if needs_resize {
        info!("🔧 Optimizing image from {}x{} to max {}px", img.width(), img.height(), max_dimension);
        let resized = img.resize(max_dimension, max_dimension, image::imageops::FilterType::Lanczos3);
        let mut buf = Vec::new();
        resized.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
        info!("✅ Image optimized: {} bytes -> {} bytes", data.len(), buf.len());
        Ok(buf)
    } else {
        info!("✅ Image already optimal ({}x{})", img.width(), img.height());
        Ok(data)
    }
}

/// Create a thumbnail (128x128) for preview purposes
fn create_thumbnail(data: &[u8], size: u32) -> Result<Vec<u8>> {
    if data.len() < 100 {
        return Err(anyhow::anyhow!("Image data too small"));
    }
    
    let img = image::load_from_memory(data)?;
    let thumbnail = img.resize(size, size, image::imageops::FilterType::Lanczos3);
    
    let mut buf = Vec::new();
    thumbnail.write_to(&mut Cursor::new(&mut buf), ImageFormat::Png)?;
    
    Ok(buf)
}

/// Save image to disk and return the file path
fn save_image_to_file(image_data: &[u8], hash: &str) -> Result<String> {
    // Create images directory if it doesn't exist
    let mut images_dir = dirs::data_local_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get data directory"))?;
    images_dir.push("clippit");
    images_dir.push("images");
    
    std::fs::create_dir_all(&images_dir)?;
    
    // Create filename from hash
    let filename = format!("{}.png", hash);
    let mut file_path = images_dir.clone();
    file_path.push(&filename);
    
    // Save image to file
    std::fs::write(&file_path, image_data)?;
    
    // Return absolute path as string
    Ok(file_path.to_string_lossy().to_string())
}

/// Set clipboard content (used for copying entries back to clipboard)
#[allow(dead_code)]
pub fn set_clipboard_content(entry: &ClipboardEntry) -> Result<()> {
    let mut clipboard = Clipboard::new()?;
    
    match entry.content_type {
        clippit_core::ContentType::Text => {
            if let Some(text) = &entry.content_text {
                clipboard.set_text(text)?;
                info!("✅ Set clipboard to text entry");
            }
        }
        clippit_core::ContentType::Image => {
            if let Some(data) = &entry.content_data {
                info!("📋 Setting clipboard to image ({} bytes)", data.len());
                
                // Load image and convert to ImageData
                let img = image::load_from_memory(data)?;
                let rgba = img.to_rgba8();
                
                let img_data = ImageData {
                    width: rgba.width() as usize,
                    height: rgba.height() as usize,
                    bytes: rgba.as_raw().into(),
                };
                
                clipboard.set_image(img_data)?;
                info!("✅ Image copied to clipboard successfully");
            }
        }
    }

    Ok(())
}
