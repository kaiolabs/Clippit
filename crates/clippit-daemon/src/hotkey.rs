use anyhow::Result;
use clippit_core::{Config, HistoryManager};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{error, info};

pub async fn start_hotkey_handler(_history_manager: Arc<Mutex<HistoryManager>>) -> Result<()> {
    info!("Starting hotkey handler...");

    // Load configuration
    let config = Config::load().unwrap_or_default();
    
    let manager = GlobalHotKeyManager::new()?;

    // Parse modifier from config
    let modifiers = parse_modifiers(&config.hotkeys.show_history_modifier);
    
    // Parse key from config
    let key_code = parse_key(&config.hotkeys.show_history_key);
    
    // Register hotkey
    let hotkey = HotKey::new(modifiers, key_code);
    manager.register(hotkey)?;

    // Log configured hotkey prominently
    eprintln!(""); 
    eprintln!("═══════════════════════════════════════════════════════");
    eprintln!("  🔑 ATALHO REGISTRADO: {} + {}", 
        config.hotkeys.show_history_modifier.to_uppercase(), 
        config.hotkeys.show_history_key.to_uppercase()
    );
    eprintln!("═══════════════════════════════════════════════════════");
    eprintln!("");
    
    info!("Hotkey handler ready with: {} + {}", 
        config.hotkeys.show_history_modifier, 
        config.hotkeys.show_history_key
    );

    let receiver = GlobalHotKeyEvent::receiver();

    loop {
        if let Ok(event) = receiver.try_recv() {
            if event.state == HotKeyState::Pressed {
                info!("Hotkey pressed! Notifying UI to show popup");

                // Send signal to UI via IPC
                if let Err(e) = notify_ui_show_popup() {
                    error!("Failed to notify UI: {}", e);
                }
            }
        }

        sleep(Duration::from_millis(10)).await;
    }
}

fn notify_ui_show_popup() -> Result<()> {
    // Check lock file instead of pgrep (more reliable)
    let lock_file = std::path::Path::new("/tmp/clippit-popup.lock");
    
    if lock_file.exists() {
        // Read PID from lock file
        if let Ok(content) = std::fs::read_to_string(lock_file) {
            if let Ok(pid) = content.trim().parse::<i32>() {
                // Check if process is actually alive (not zombie)
                let check = Command::new("ps")
                    .args(&["-p", &pid.to_string(), "-o", "stat="])
                    .output();
                
                if let Ok(output) = check {
                    let stat = String::from_utf8_lossy(&output.stdout);
                    // If process is zombie (Z) or doesn't exist, clean up
                    if stat.trim().starts_with('Z') || stat.trim().is_empty() {
                        info!("Cleaning up stale/zombie popup (PID: {})", pid);
                        let _ = Command::new("kill").args(&["-9", &pid.to_string()]).output();
                        std::fs::remove_file(lock_file).ok();
                    } else {
                        // Process is alive, toggle (close it)
                        info!("Popup already open - closing (toggle)");
                        let _ = Command::new("kill").args(&["-TERM", &pid.to_string()]).output();
                        
                        // Aguarda o popup fechar completamente (até 500ms)
                        for i in 0..10 {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            // Verifica se o processo ainda existe
                            let check = Command::new("ps")
                                .args(&["-p", &pid.to_string()])
                                .output();
                            
                            if let Ok(output) = check {
                                if !output.status.success() {
                                    info!("Popup fechado após {}ms", (i + 1) * 50);
                                    break;
                                }
                            }
                        }
                        
                        std::fs::remove_file(lock_file).ok();
                        return Ok(());
                    }
                }
            }
        }
    }
    
    // Launch clippit-popup IMEDIATAMENTE (captura window ID de forma rápida)
    info!("Opening popup...");
    
    // Tenta capturar window ID usando xdotool (sem timeout primeiro)
    let result = Command::new("xdotool")
        .arg("getwindowfocus")
        .output();
    
    let active_window_id = match result {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let trimmed = stdout.trim();
                info!("✅ xdotool getwindowfocus output: '{}'", trimmed);
                
                if !trimmed.is_empty() && trimmed != "0" {
                    trimmed.to_string()
                } else {
                    info!("⚠️  getwindowfocus returned 0 or empty, trying getactivewindow...");
                    // Fallback: getactivewindow
                    Command::new("xdotool")
                        .arg("getactivewindow")
                        .output()
                        .ok()
                        .and_then(|o| {
                            if o.status.success() {
                                let out = String::from_utf8_lossy(&o.stdout);
                                let trimmed = out.trim();
                                info!("✅ xdotool getactivewindow output: '{}'", trimmed);
                                if !trimmed.is_empty() && trimmed != "0" {
                                    Some(trimmed.to_string())
                                } else {
                                    None
                                }
                            } else {
                                info!("❌ getactivewindow failed: {:?}", o.status);
                                None
                            }
                        })
                        .unwrap_or_else(|| "0".to_string())
                }
            } else {
                info!("❌ xdotool getwindowfocus failed: {:?}", output.status);
                info!("   stderr: {}", String::from_utf8_lossy(&output.stderr));
                "0".to_string()
            }
        }
        Err(e) => {
            info!("❌ Failed to execute xdotool: {}", e);
            "0".to_string()
        }
    };
    
    info!("🎯 Final captured window ID: {}", active_window_id);
    
    std::process::Command::new("clippit-popup")
        .env("CLIPPIT_TARGET_WINDOW", &active_window_id)  // Passa window ID via env
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .ok();

    Ok(())
}

fn parse_modifiers(modifier_str: &str) -> Option<Modifiers> {
    let mut modifiers = Modifiers::empty();
    
    for part in modifier_str.split('+') {
        let part = part.trim().to_lowercase();
        match part.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "super" | "meta" | "win" => modifiers |= Modifiers::SUPER,
            "none" | "" => return None,
            _ => {}
        }
    }
    
    if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    }
}

fn parse_key(key_str: &str) -> Code {
    let key = key_str.trim().to_lowercase();
    
    match key.as_str() {
        // Letters
        "a" => Code::KeyA,
        "b" => Code::KeyB,
        "c" => Code::KeyC,
        "d" => Code::KeyD,
        "e" => Code::KeyE,
        "f" => Code::KeyF,
        "g" => Code::KeyG,
        "h" => Code::KeyH,
        "i" => Code::KeyI,
        "j" => Code::KeyJ,
        "k" => Code::KeyK,
        "l" => Code::KeyL,
        "m" => Code::KeyM,
        "n" => Code::KeyN,
        "o" => Code::KeyO,
        "p" => Code::KeyP,
        "q" => Code::KeyQ,
        "r" => Code::KeyR,
        "s" => Code::KeyS,
        "t" => Code::KeyT,
        "u" => Code::KeyU,
        "v" => Code::KeyV,
        "w" => Code::KeyW,
        "x" => Code::KeyX,
        "y" => Code::KeyY,
        "z" => Code::KeyZ,
        
        // Numbers
        "0" | "digit0" => Code::Digit0,
        "1" | "digit1" => Code::Digit1,
        "2" | "digit2" => Code::Digit2,
        "3" | "digit3" => Code::Digit3,
        "4" | "digit4" => Code::Digit4,
        "5" | "digit5" => Code::Digit5,
        "6" | "digit6" => Code::Digit6,
        "7" | "digit7" => Code::Digit7,
        "8" | "digit8" => Code::Digit8,
        "9" | "digit9" => Code::Digit9,
        
        // Numpad
        "kp_0" | "numpad0" => Code::Numpad0,
        "kp_1" | "numpad1" => Code::Numpad1,
        "kp_2" | "numpad2" => Code::Numpad2,
        "kp_3" | "numpad3" => Code::Numpad3,
        "kp_4" | "numpad4" => Code::Numpad4,
        "kp_5" | "numpad5" => Code::Numpad5,
        "kp_6" | "numpad6" => Code::Numpad6,
        "kp_7" | "numpad7" => Code::Numpad7,
        "kp_8" | "numpad8" => Code::Numpad8,
        "kp_9" | "numpad9" => Code::Numpad9,
        
        // Function keys
        "f1" => Code::F1,
        "f2" => Code::F2,
        "f3" => Code::F3,
        "f4" => Code::F4,
        "f5" => Code::F5,
        "f6" => Code::F6,
        "f7" => Code::F7,
        "f8" => Code::F8,
        "f9" => Code::F9,
        "f10" => Code::F10,
        "f11" => Code::F11,
        "f12" => Code::F12,
        
        // Special keys
        "space" => Code::Space,
        "enter" | "return" => Code::Enter,
        "escape" | "esc" => Code::Escape,
        "backspace" => Code::Backspace,
        "tab" => Code::Tab,
        "insert" => Code::Insert,
        "delete" => Code::Delete,
        "home" => Code::Home,
        "end" => Code::End,
        "pageup" => Code::PageUp,
        "pagedown" => Code::PageDown,
        
        // Arrow keys
        "left" => Code::ArrowLeft,
        "right" => Code::ArrowRight,
        "up" => Code::ArrowUp,
        "down" => Code::ArrowDown,
        
        // Default
        _ => Code::KeyV, // Fallback to V
    }
}
