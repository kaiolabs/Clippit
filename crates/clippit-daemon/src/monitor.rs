use anyhow::Result;
use clippit_core::{ClipboardEntry, Config, HistoryManager};
use dirs;
use image;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};
use x11_clipboard::Clipboard;

pub async fn start_monitor(history_manager: Arc<Mutex<HistoryManager>>) -> Result<()> {
    info!("Starting clipboard monitor...");

    let clipboard = Clipboard::new()?;
    let mut last_content: Option<String> = None;
    let mut last_image_hash: Option<String> = None;

    loop {
        // Load config for each iteration (to respect runtime changes)
        let config = Config::load().unwrap_or_default();
        
        // Try to get text first
        match get_clipboard_text(&clipboard) {
            Ok(content) => {
                if let Some(text) = content {
                    // Check if content changed
                    if last_content.as_ref() != Some(&text) {
                        info!("Clipboard changed, saving to history");

                        let entry = ClipboardEntry::new_text(text.clone());
                        let mut manager = history_manager.lock().unwrap();

                        match manager.add_entry(entry) {
                            Ok(Some(id)) => {
                                info!("Saved entry with id {}", id);
                                last_content = Some(text);
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
            }
            Err(e) => {
                warn!("Failed to get clipboard content: {}", e);
            }
        }
        
        // Try to get image if enabled
        if config.privacy.enable_image_capture {
            match get_clipboard_image(&clipboard) {
                Ok(Some(image_data)) => {
                    // Validate size (convert MB to bytes)
                    let max_size_bytes = (config.privacy.max_image_size_mb as usize) * 1024 * 1024;
                    
                    if image_data.len() <= max_size_bytes {
                        // Compute hash to avoid duplicates
                        use sha2::{Digest, Sha256};
                        let mut hasher = Sha256::new();
                        hasher.update(&image_data);
                        let current_hash = format!("{:x}", hasher.finalize());
                        
                        info!("🔍 Image hash comparison:");
                        info!("   Current hash: {}...", &current_hash[..12]);
                        info!("   Last hash: {:?}", last_image_hash.as_ref().map(|h| &h[..12]));
                        info!("   Are different? {}", last_image_hash.as_ref() != Some(&current_hash));
                        
                        // Only save if different from last image
                        if last_image_hash.as_ref() != Some(&current_hash) {
                            info!("📸 New image detected, optimizing and saving...");
                            
                            // Optimize if needed (max 2048px)
                            match optimize_image(image_data.clone(), 2048) {
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
                                                    // IMPORTANTE: Atualizar hash mesmo para duplicatas para evitar loop
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
                            image_data.len() / 1024 / 1024, 
                            config.privacy.max_image_size_mb);
                    }
                }
                Ok(None) => {
                    // No image in clipboard
                }
                Err(e) => {
                    warn!("Failed to get clipboard image: {}", e);
                }
            }
        }

        // Polling mais rápido (80ms) para melhor responsividade ao tirar prints
        sleep(Duration::from_millis(80)).await;
    }
}

fn get_clipboard_text(clipboard: &Clipboard) -> Result<Option<String>> {
    let atoms = clipboard.getter.atoms.clone();

    // Try UTF8_STRING first
    if let Ok(data) = clipboard.load(
        atoms.clipboard,
        atoms.utf8_string,
        atoms.property,
        Duration::from_millis(100),
    ) {
        if !data.is_empty() {
            let text = String::from_utf8(data)?;
            return Ok(Some(text));
        }
    }

    // Fallback to STRING
    if let Ok(data) = clipboard.load(
        atoms.clipboard,
        atoms.string,
        atoms.property,
        Duration::from_millis(100),
    ) {
        if !data.is_empty() {
            let text = String::from_utf8_lossy(&data).to_string();
            return Ok(Some(text));
        }
    }

    Ok(None)
}

fn get_clipboard_image(_clipboard: &Clipboard) -> Result<Option<Vec<u8>>> {
    // Use xclip to get image data from clipboard (more reliable than x11-clipboard for images)
    use std::process::{Command, Stdio};
    
    // Try to get PNG from clipboard
    match Command::new("xclip")
        .args(&["-selection", "clipboard", "-t", "image/png", "-o"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            // Try to validate by loading with image library (more flexible than magic bytes)
            if image::load_from_memory(&output.stdout).is_ok() {
                info!("📸 Captured image from clipboard ({} bytes)", output.stdout.len());
                return Ok(Some(output.stdout));
            }
        }
        _ => {}
    }
    
    // Fallback to JPEG
    match Command::new("xclip")
        .args(&["-selection", "clipboard", "-t", "image/jpeg", "-o"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            // Try to validate by loading with image library
            if image::load_from_memory(&output.stdout).is_ok() {
                info!("📸 Captured JPEG image from clipboard ({} bytes)", output.stdout.len());
                return Ok(Some(output.stdout));
            }
        }
        _ => {}
    }
    
    Ok(None)
}

fn optimize_image(data: Vec<u8>, max_dimension: u32) -> Result<Vec<u8>> {
    // Validate minimum size (avoid processing garbage data)
    if data.len() < 100 {
        return Err(anyhow::anyhow!("Image data too small ({}  bytes), likely invalid", data.len()));
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
        resized.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
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
    thumbnail.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;
    
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

#[allow(dead_code)]
pub fn set_clipboard_content(entry: &ClipboardEntry) -> Result<()> {
    match entry.content_type {
        clippit_core::ContentType::Text => {
            let clipboard = Clipboard::new()?;
            if let Some(text) = &entry.content_text {
                clipboard.store(
                    clipboard.setter.atoms.clipboard,
                    clipboard.setter.atoms.utf8_string,
                    text.as_bytes(),
                )?;
                info!("✅ Set clipboard to text entry");
            }
        }
        clippit_core::ContentType::Image => {
            if let Some(data) = &entry.content_data {
                // Use xclip for image clipboard (more reliable than x11-clipboard for images)
                use std::io::Write;
                use std::process::{Command, Stdio};
                
                info!("📋 Setting clipboard to image ({} bytes)", data.len());
                
                let mut child = Command::new("xclip")
                    .args(&["-selection", "clipboard", "-t", "image/png"])
                    .stdin(Stdio::piped())
                    .spawn()?;
                
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(data)?;
                }
                
                let status = child.wait()?;
                
                if status.success() {
                    info!("✅ Image copied to clipboard successfully");
                } else {
                    error!("❌ xclip failed with status: {}", status);
                }
            }
        }
    }

    Ok(())
}
