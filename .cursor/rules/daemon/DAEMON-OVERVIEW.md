# clippit-daemon - Overview

## 📍 Localização
`crates/clippit-daemon/`

## 🎯 Responsabilidade

**clippit-daemon** é o serviço de background que orquestra todas as operações do Clippit:
- Monitora clipboard (Wayland/X11)
- Gerencia hotkeys globais
- Provê servidor IPC para comunicação com UIs
- Monitora digitação para autocomplete global

### Princípios
- ✅ **Always Running**: Serviço systemd user-level
- ✅ **Low Resource**: ~20MB RAM, ~0% CPU idle
- ✅ **Async First**: Tokio runtime para concorrência
- ✅ **Fault Tolerant**: Recupera de erros sem crash

## 📦 Estrutura

```
crates/clippit-daemon/
├── Cargo.toml
└── src/
    ├── main.rs                    # Entry point, orchestration
    ├── monitor.rs                 # Clipboard monitor (polling)
    ├── hotkey.rs                  # Global hotkeys handler
    ├── typing_monitor.rs          # Typing monitor (autocomplete)
    └── autocomplete_manager.rs    # Autocomplete manager
```

## 🔧 Módulos

### 1. main.rs - Orchestrator

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    tracing_subscriber::fmt().init();
    
    // Load config
    let config = Config::load()?;
    
    // Create shared history
    let history = Arc::new(Mutex::new(
        HistoryManager::new(config.general.max_history_items)?
    ));
    
    // Spawn tasks
    let h1 = history.clone();
    tokio::spawn(async move {
        monitor::start_monitor(h1).await
    });
    
    let h2 = history.clone();
    tokio::spawn(async move {
        hotkey::start_hotkey_handler(h2).await
    });
    
    let h3 = history.clone();
    tokio::spawn(async move {
        typing_monitor::start(h3).await
    });
    
    // IPC server (blocks)
    ipc_server::run(history).await?;
    
    Ok(())
}
```

**Ver**: [../01-ARCHITECTURE.md](../01-ARCHITECTURE.md)

### 2. monitor.rs - Clipboard Monitor
Polling a cada 80ms do clipboard Wayland/X11.
**Ver**: [MONITOR-CLIPBOARD.md](./MONITOR-CLIPBOARD.md)

### 3. hotkey.rs - Hotkeys
Registra hotkey global (Super+V) e spawna popup.
**Ver**: [HOTKEYS-SYSTEM.md](./HOTKEYS-SYSTEM.md)

### 4. typing_monitor.rs - Autocomplete
Monitora digitação e gera sugestões.
**Ver**: [TYPING-AUTOCOMPLETE.md](./TYPING-AUTOCOMPLETE.md)

### 5. IPC Server
Processa mensagens IPC de clientes (popup, dashboard, ibus).
**Ver**: [IPC-SERVER.md](./IPC-SERVER.md)

## 📊 Dependências

```toml
[dependencies]
tokio = { version = "1.36", features = ["full"] }
arboard = { version = "3.6", features = ["wayland-data-control"] }
global-hotkey = "0.7"
rdev = "0.5"
clippit-core = { path = "../clippit-core" }
clippit-ipc = { path = "../clippit-ipc" }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
chrono = "0.4"
sha2 = "0.10"
image = "0.25"
```

## 🔄 Ciclo de Vida

### Inicialização
1. Carrega configuração (`Config::load()`)
2. Cria `HistoryManager` compartilhado
3. Spawna tasks assíncronas
4. Inicia IPC server (blocking)

### Runtime
- **Monitor task**: Loop infinito polling clipboard
- **Hotkey task**: Loop bloqueante escutando eventos
- **Typing task**: Loop processando keystrokes
- **IPC task**: Processa conexões de clientes

### Shutdown
- Graceful: SIGTERM/SIGINT
- Salva estado se necessário
- Fecha conexões IPC

## ✅ Padrões

### Shared State

```rust
let history = Arc::new(Mutex::new(HistoryManager::new(100)?));

// Clone para cada task
let h1 = history.clone();
tokio::spawn(async move {
    // Usa h1
});
```

### Error Handling

```rust
// Tasks não devem panic, apenas log erro
tokio::spawn(async move {
    loop {
        if let Err(e) = process().await {
            tracing::error!("Error: {}", e);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
});
```

### Configuration Reload

```rust
// Reload config periodically
loop {
    let config = Config::load()?;
    // Usa config
    tokio::time::sleep(Duration::from_secs(60)).await;
}
```

## 🚫 Anti-Patterns

❌ **Panic em Tasks**: Use log + continue
❌ **Busy Loop**: Sempre use sleep entre iterações
❌ **Blocking Calls**: Use async ou spawn_blocking
❌ **Memory Leaks**: Cleanup resources

## 🔗 Links

- **Monitor**: [MONITOR-CLIPBOARD.md](./MONITOR-CLIPBOARD.md)
- **Hotkeys**: [HOTKEYS-SYSTEM.md](./HOTKEYS-SYSTEM.md)
- **Typing**: [TYPING-AUTOCOMPLETE.md](./TYPING-AUTOCOMPLETE.md)
- **IPC**: [IPC-SERVER.md](./IPC-SERVER.md)
- **Core**: [../core/CORE-OVERVIEW.md](../core/CORE-OVERVIEW.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
