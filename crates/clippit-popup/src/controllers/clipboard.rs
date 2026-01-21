use std::process::Command;
use crate::utils::simulate_paste_to_window;

/// Copies an entry to the clipboard and pastes it to a target window
/// 
/// This function:
/// 1. Gets the full entry data via IPC
/// 2. Copies the content to X11 clipboard using xclip
/// 3. Simulates Ctrl+V in the target window
/// 
/// # Arguments
/// * `entry_id` - The ID of the entry to paste
/// * `target_window_id` - The X11 window ID to paste into
pub fn copy_to_clipboard_and_paste_with_target(entry_id: i64, target_window_id: u64) {
    // Set panic handler for this thread
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("💥 PANIC in paste thread: {:?}", panic_info);
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }));
    
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("🔵 copy_to_clipboard_and_paste_with_target() START");
    eprintln!("   entry_id: {}, target_window: {}", entry_id, target_window_id);
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // OPTIMIZED: Get only the selected entry data (lazy loading)
    eprintln!("📡 Step 1: Getting full data for entry ID {}...", entry_id);
    let query_start = std::time::Instant::now();
    
    match clippit_ipc::IpcClient::get_entry_data(entry_id) {
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
            
            // 2. Copy content to X11 clipboard using xclip
            eprintln!("📋 Step 2: Copying to clipboard...");
            let copied_successfully = match entry.content_type {
                clippit_ipc::ContentType::Text => {
                    if let Some(text) = &entry.content_text {
                            eprintln!("🔵 Copying {} chars to X11 clipboard using xclip...", text.len());
                            
                            // Usa xclip para copiar texto diretamente para o clipboard do X11
                            use std::io::Write;
                            match Command::new("xclip")
                                .args(&["-selection", "clipboard"])
                                .stdin(std::process::Stdio::piped())
                                .spawn()
                            {
                                Ok(mut child) => {
                                    if let Some(mut stdin) = child.stdin.take() {
                                        match stdin.write_all(text.as_bytes()) {
                                            Ok(_) => {
                                                drop(stdin); // Fecha stdin
                                                match child.wait() {
                                                    Ok(status) if status.success() => {
                                                        eprintln!("✅ Text copied to clipboard: {} chars", text.len());
                                                        true
                                                    },
                                                    Ok(status) => {
                                                        eprintln!("❌ xclip exited with status: {}", status);
                                                        false
                                                    },
                                                    Err(e) => {
                                                        eprintln!("❌ Failed to wait for xclip: {}", e);
                                                        false
                                                    }
                                                }
                                            },
                                            Err(e) => {
                                                eprintln!("❌ Failed to write to xclip stdin: {}", e);
                                                false
                                            }
                                        }
                                    } else {
                                        eprintln!("❌ Failed to get xclip stdin");
                                        false
                                    }
                                },
                                Err(e) => {
                                    eprintln!("❌ Failed to spawn xclip: {}", e);
                                    false
                                }
                            }
                        } else {
                            eprintln!("❌ Text entry has no content");
                            false
                        }
                    },
                  clippit_ipc::ContentType::Image => {
                      // NOVA ABORDAGEM SIMPLES: Usa o arquivo de imagem diretamente
                      // Muito mais confiável que stdin para imagens grandes
                      
                      // Busca o image_path do entry original (metadata)
                      if let Some(image_path) = &entry.image_path {
                          eprintln!("📸 Copying image from file: {}", image_path);
                          
                          // xclip lê o arquivo diretamente (muito mais rápido e confiável)
                          match Command::new("xclip")
                              .args(&["-selection", "clipboard", "-t", "image/png", "-i", image_path])
                              .output()
                          {
                              Ok(output) => {
                                  if output.status.success() {
                                      eprintln!("✅ Image copied to clipboard from file: {}", image_path);
                                      true
                                  } else {
                                      eprintln!("❌ xclip failed: {:?}", String::from_utf8_lossy(&output.stderr));
                                      false
                                  }
                              },
                              Err(e) => {
                                  eprintln!("❌ Failed to execute xclip: {}", e);
                                  false
                              }
                          }
                      } else {
                          eprintln!("❌ Image entry has no image_path!");
                          false
                      }
                  }
            };
            
            // 3. Now simulate paste with target window
            eprintln!("📋 Step 3: Checking if copy was successful...");
            if copied_successfully {
                eprintln!("✅ Content copied successfully!");
                eprintln!("⌨️  Step 4: Simulating paste to window {}...", target_window_id);
                simulate_paste_to_window(target_window_id);
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                eprintln!("✅ copy_to_clipboard_and_paste_with_target() COMPLETED");
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            } else {
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                eprintln!("❌ FAILED: Skipping paste - copy failed");
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
        }
        Err(e) => {
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("❌ FAILED: Could not get entry data for ID {}: {}", entry_id, e);
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
    }
}
