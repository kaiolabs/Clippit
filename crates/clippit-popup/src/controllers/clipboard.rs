use arboard::{Clipboard, ImageData};
use clippit_core::Config;

/// Copies an entry to the clipboard and shows a system notification
/// 
/// This function:
/// 1. Gets the full entry data via IPC
/// 2. Copies the content to clipboard using arboard (Wayland-native)
/// 3. Shows a system notification to user
/// 4. Returns success status to allow caller to close the window immediately
/// 
/// # Arguments
/// * `entry_id` - The ID of the entry to copy
/// 
/// # Returns
/// * `true` if copy was successful, `false` otherwise
pub fn copy_to_clipboard(entry_id: i64) -> bool {
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("🔵 copy_to_clipboard() START");
    eprintln!("   entry_id: {}", entry_id);
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Load config to check if notifications are enabled
    let config = Config::load().unwrap_or_default();
    let show_notifications = config.ui.show_notifications;
    eprintln!("🔔 Notifications enabled: {}", show_notifications);
    
    // Get entry data from daemon via IPC
    eprintln!("📡 Getting full data for entry ID {}...", entry_id);
    let query_start = std::time::Instant::now();
    
    let success = match clippit_ipc::IpcClient::get_entry_data(entry_id) {
        Ok(entry) => {
            let query_duration = query_start.elapsed();
            eprintln!("✅ Entry data retrieved in {:?}", query_duration);
            eprintln!("   Content type: {:?}", entry.content_type);
            eprintln!("   Has text: {}", entry.content_text.is_some());
            eprintln!("   Has data: {}", entry.content_data.is_some());
            eprintln!("   Has image_path: {}", entry.image_path.is_some());
            
            if let Some(ref path) = entry.image_path {
                eprintln!("   Image path: {}", path);
            }
            
            if let Some(ref data) = entry.content_data {
                eprintln!("   Data size: {} bytes ({:.2} MB)", data.len(), data.len() as f64 / (1024.0 * 1024.0));
            }
            
            // Create clipboard instance
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    eprintln!("❌ Failed to create clipboard: {}", e);
                    show_notification("Erro", &format!("Erro ao acessar clipboard: {}", e), show_notifications);
                    return false;
                }
            };
            
            // Copy content based on type
            match entry.content_type {
                clippit_ipc::ContentType::Text => {
                    if let Some(text) = &entry.content_text {
                        eprintln!("🔵 Copying {} chars to clipboard using arboard...", text.len());
                        
                        match clipboard.set_text(text) {
                            Ok(_) => {
                                eprintln!("✅ Text copied to clipboard: {} chars", text.len());
                                // Show preview of copied text (first 80 chars)
                                let preview = if text.len() > 80 {
                                    format!("{}...", &text[..80])
                                } else {
                                    text.clone()
                                };
                                show_notification("Clippit", &format!("Copiado: {}", preview), show_notifications);
                                true
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to copy text: {}", e);
                                show_notification("Erro", &format!("Erro ao copiar texto: {}", e), show_notifications);
                                false
                            }
                        }
                    } else {
                        eprintln!("❌ Text entry has no content");
                        show_notification("Erro", "Entrada sem conteúdo", show_notifications);
                        false
                    }
                }
                clippit_ipc::ContentType::Image => {
                    if let Some(image_path) = &entry.image_path {
                        eprintln!("📸 Copying image from file: {}", image_path);
                        
                        // Load image from file
                        match image::open(image_path) {
                            Ok(img) => {
                                let rgba = img.to_rgba8();
                                let img_data = ImageData {
                                    width: rgba.width() as usize,
                                    height: rgba.height() as usize,
                                    bytes: rgba.as_raw().into(),
                                };
                                
                                match clipboard.set_image(img_data) {
                                    Ok(_) => {
                                        eprintln!("✅ Image copied to clipboard from file: {}", image_path);
                                        show_notification("Clippit", &format!("Imagem copiada ({}x{})", rgba.width(), rgba.height()), show_notifications);
                                        true
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Failed to copy image: {}", e);
                                        show_notification("Erro", &format!("Erro ao copiar imagem: {}", e), show_notifications);
                                        false
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to load image: {}", e);
                                show_notification("Erro", &format!("Erro ao carregar imagem: {}", e), show_notifications);
                                false
                            }
                        }
                    } else {
                        eprintln!("❌ Image entry has no image_path!");
                        show_notification("Erro", "Imagem sem caminho", show_notifications);
                        false
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("❌ FAILED: Could not get entry data for ID {}: {}", entry_id, e);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            show_notification("Erro", &format!("Erro: {}", e), show_notifications);
            false
        }
    };
    
    if success {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("✅ copy_to_clipboard() COMPLETED");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    } else {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("❌ copy_to_clipboard() FAILED");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }
    
    success
}

/// Show a system notification using notify-send (reliable and blocking)
fn show_notification(summary: &str, body: &str, enabled: bool) {
    if !enabled {
        eprintln!("🔕 Notificações desabilitadas - pulando notificação");
        return;
    }
    
    eprintln!("🔔 Enviando notificação via notify-send...");
    eprintln!("   Summary: {}", summary);
    eprintln!("   Body: {}", body);
    
    // Use notify-send directly - it's more reliable for short-lived processes
    match std::process::Command::new("notify-send")
        .arg(summary)
        .arg(body)
        .arg("-i")
        .arg("edit-copy")
        .arg("-t")
        .arg("3000")
        .arg("-a")
        .arg("Clippit")
        .arg("-u")
        .arg("normal")
        .status()
    {
        Ok(status) if status.success() => {
            eprintln!("✅ Notificação enviada com sucesso!");
        }
        Ok(status) => {
            eprintln!("⚠️  notify-send retornou status: {:?}", status);
        }
        Err(e) => {
            eprintln!("❌ Falha ao executar notify-send: {}", e);
            eprintln!("   Verifique se libnotify-bin está instalado");
        }
    }
}
