# clippit-daemon - Sistema de Hotkeys

## 📍 Localização
`crates/clippit-daemon/src/hotkey.rs`

## 🎯 Responsabilidade

Gerencia hotkeys globais (Super+V padrão) para abrir/fechar o popup.

## 🔄 Fluxo

```rust
pub async fn start_hotkey_handler() {
    let config = Config::load().unwrap();
    let hotkey = parse_hotkey(&config.hotkey);
    
    let manager = GlobalHotkeyManager::new().unwrap();
    manager.register(hotkey).unwrap();
    
    loop {
        if let Some(event) = manager.receiver().recv() {
            if event.state == HotkeyState::Pressed {
                toggle_popup().await;
            }
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

async fn toggle_popup() {
    let lock_path = Path::new("/tmp/clippit-popup.lock");
    
    if lock_path.exists() {
        // Popup aberto, fechar
        let pid = fs::read_to_string(lock_path).unwrap();
        Command::new("kill").arg(&pid).status().ok();
        fs::remove_file(lock_path).ok();
    } else {
        // Abrir popup
        Command::new("clippit-popup")
            .spawn()
            .ok();
    }
}
```

## 📝 Parsing de Hotkey

```rust
fn parse_hotkey(config: &HotkeyConfig) -> Hotkey {
    let modifiers = parse_modifiers(&config.modifier);
    let key = parse_key(&config.key);
    Hotkey::new(Some(modifiers), key)
}

fn parse_modifiers(s: &str) -> Modifiers {
    let mut mods = Modifiers::empty();
    for part in s.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "super" | "meta" | "win" => mods |= Modifiers::SUPER,
            _ => {}
        }
    }
    mods
}
```

## 🔗 Links
- [Daemon Overview](./DAEMON-OVERVIEW.md)
- [Config Patterns](../core/CONFIG-PATTERNS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
