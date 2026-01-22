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
    
    // Try to register, unregister and retry if already registered
    match manager.register(hotkey) {
        Ok(_) => {
            info!("Hotkey registered successfully");
        }
        Err(e) if e.to_string().contains("already registered") => {
            info!("Hotkey already registered, unregistering and retrying...");
            let _ = manager.unregister(hotkey);
            std::thread::sleep(std::time::Duration::from_millis(100));
            manager.register(hotkey).map_err(|e| {
                error!("Failed to register hotkey after retry: {}", e);
                e
            })?;
        }
        Err(e) => {
            error!("Failed to register hotkey: {}", e);
            return Err(e.into());
        }
    }

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
    
    info!("🔍 Checking lock file: exists={}", lock_file.exists());
    
    if lock_file.exists() {
        // Read PID from lock file
        if let Ok(content) = std::fs::read_to_string(lock_file) {
            info!("📄 Lock file content: '{}'", content);
            if let Ok(pid) = content.trim().parse::<i32>() {
                info!("🔍 Checking PID {} status...", pid);
                // Check if process is actually alive (not zombie)
                let check = Command::new("ps")
                    .args(&["-p", &pid.to_string(), "-o", "stat="])
                    .output();
                
                if let Ok(output) = check {
                    let stat = String::from_utf8_lossy(&output.stdout);
                    info!("📊 Process stat: '{}'", stat.trim());
                    // If process is zombie (Z) or doesn't exist, clean up
                    if stat.trim().starts_with('Z') || stat.trim().is_empty() {
                        info!("💀 Cleaning up stale/zombie popup (PID: {})", pid);
                        let _ = Command::new("kill").args(&["-9", &pid.to_string()]).output();
                        std::fs::remove_file(lock_file).ok();
                    } else {
                        // Process is alive, toggle (close it)
                        info!("🔄 Popup already open (PID: {}) - closing (toggle)", pid);
                        info!("📤 Sending SIGTERM...");
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
                                    info!("✅ Popup fechado após {}ms", (i + 1) * 50);
                                    break;
                                }
                            }
                        }
                        
                        info!("🗑️  Removing lock file...");
                        std::fs::remove_file(lock_file).ok();
                        info!("✅ Toggle complete");
                        return Ok(());
                    }
                }
            }
        }
    } else {
        info!("❌ Lock file does not exist");
    }
    
    // Launch clippit-popup (Wayland-native, no window ID needed)
    info!("🚀 Opening popup...");
    
    std::process::Command::new("clippit-popup")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .ok();
    
    // Small delay to allow popup to create lock file
    std::thread::sleep(std::time::Duration::from_millis(150));
    
    info!("✅ Popup launch complete");

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
