use std::process::Command;

/// Gets the currently focused window ID using xdotool
/// 
/// # Returns
/// The window ID as u64, or 0 if unable to determine
pub fn get_focused_window_id() -> u64 {
    // Tenta ler da variável de ambiente primeiro (setada pelo daemon)
    if let Ok(window_id_str) = std::env::var("CLIPPIT_TARGET_WINDOW") {
        eprintln!("🔵 CLIPPIT_TARGET_WINDOW env var: '{}'", window_id_str);
        if let Ok(window_id) = window_id_str.trim().parse::<u64>() {
            if window_id > 0 {
                eprintln!("✅ Using window ID from daemon: {}", window_id);
                return window_id;
            } else {
                eprintln!("⚠️  Window ID from daemon is 0 (invalid)");
            }
        } else {
            eprintln!("⚠️  Failed to parse window ID: '{}'", window_id_str);
        }
    } else {
        eprintln!("⚠️  CLIPPIT_TARGET_WINDOW env var not set");
    }
    
    // Fallback: tenta capturar diretamente
    eprintln!("⚠️  Trying to capture window ID directly...");
    let result = Command::new("xdotool")
        .arg("getwindowfocus")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                eprintln!("❌ xdotool getwindowfocus failed: {:?}", o.status);
                None
            }
        })
        .and_then(|s| {
            let trimmed = s.trim();
            eprintln!("🔵 xdotool output: '{}'", trimmed);
            trimmed.parse::<u64>().ok()
        })
        .unwrap_or(0);
    
    eprintln!("🔵 Final window ID: {}", result);
    result
}

/// Simulates a paste operation to a specific window
/// 
/// # Arguments
/// * `target_window_id` - The X11 window ID to paste into
pub fn simulate_paste_to_window(target_window_id: u64) {
    eprintln!("🔵 simulate_paste_to_window() - target: {}", target_window_id);
    
    // ⏰ CRITICAL: GTK window.close() é ASSÍNCRONO!
    // Aguarda tempo suficiente para o popup fechar COMPLETAMENTE
    eprintln!("⏳ Aguardando 500ms para popup fechar completamente...");
    std::thread::sleep(std::time::Duration::from_millis(500));
    eprintln!("✅ Popup deve estar fechado agora");
    
    // Foca explicitamente na janela alvo usando windowactivate (mais robusto que windowfocus)
    if target_window_id > 0 {
        eprintln!("🎯 Ativando janela {} com windowactivate...", target_window_id);
        let activate_result = Command::new("xdotool")
            .args(&["windowactivate", &target_window_id.to_string()])
            .output();
        
        match activate_result {
            Ok(o) if o.status.success() => {
                eprintln!("✅ Janela {} ativada com sucesso", target_window_id);
            },
            Ok(o) => {
                eprintln!("⚠️ xdotool windowactivate falhou: {}", String::from_utf8_lossy(&o.stderr));
                eprintln!("   Status: {}", o.status);
                eprintln!("   Stdout: {}", String::from_utf8_lossy(&o.stdout));
            },
            Err(e) => {
                eprintln!("❌ Falha ao executar xdotool windowactivate: {}", e);
            }
        }
        
        // Delay adicional para garantir que o foco foi aplicado
        eprintln!("⏳ Aguardando 200ms para foco ser aplicado...");
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Verificar se o foco foi aplicado
        if let Ok(output) = Command::new("xdotool").arg("getactivewindow").output() {
            if output.status.success() {
                let active_window = String::from_utf8_lossy(&output.stdout).trim().to_string();
                eprintln!("🔍 Janela ativa agora: {} (esperado: {})", active_window, target_window_id);
                if active_window == target_window_id.to_string() {
                    eprintln!("✅ Foco confirmado na janela correta!");
                } else {
                    eprintln!("⚠️ AVISO: Foco em janela diferente! Tentando colar mesmo assim...");
                }
            }
        }
    } else {
        eprintln!("⚠️ Window ID inválido (0), tentando colar na janela ativa");
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    eprintln!("⌨️  Simulando Ctrl+V com --clearmodifiers...");
    
    // Use xdotool to simulate Ctrl+V
    let output = Command::new("xdotool")
        .args(&[
            "key",
            "--clearmodifiers",
            "ctrl+v"
        ])
        .output();
    
    match output {
        Ok(o) if o.status.success() => {
            eprintln!("✅ Ctrl+V simulado com sucesso!");
            eprintln!("   Agora verifique se o conteúdo foi colado.");
        }
        Ok(o) => {
            eprintln!("❌ xdotool key falhou!");
            eprintln!("   Status: {}", o.status);
            eprintln!("   Stderr: {}", String::from_utf8_lossy(&o.stderr));
            eprintln!("   Stdout: {}", String::from_utf8_lossy(&o.stdout));
        }
        Err(e) => {
            eprintln!("❌ Falha ao executar xdotool: {}", e);
        }
    }
}
