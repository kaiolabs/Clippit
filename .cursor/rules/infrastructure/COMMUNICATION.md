# Communication Patterns

## 🔄 Padrões de Comunicação

### 1. Daemon ↔ UI (IPC)
- **Protocol**: JSON sobre Unix Socket
- **Client**: `IpcClient` (sync/async)
- **Server**: IPC loop no daemon
- **Pattern**: Request-Response

### 2. Daemon ↔ IBus Engine (IPC)
- **Protocol**: Mesmo que UI
- **Mensagens**: Autocomplete specific
- **Flow**: Bidirecional

### 3. Daemon ↔ System
- **Clipboard**: arboard (polling)
- **Hotkeys**: global-hotkey (events)
- **Keyboard**: rdev (blocking events)

### 4. UI ↔ System
- **Clipboard copy**: arboard direct
- **Spawn**: Command::new()

## 🔌 Connection Management

### IPC Client
```rust
pub struct IpcClient {
    conn: LocalSocketStream,
}

impl IpcClient {
    pub fn connect() -> Result<Self>;
    pub fn send_message(&mut self, msg: IpcMessage) -> Result<IpcResponse>;
}
```

### Retry Logic
```rust
fn connect_with_retry() -> Result<IpcClient> {
    for i in 0..5 {
        match IpcClient::connect() {
            Ok(client) => return Ok(client),
            Err(_) if i < 4 => {
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(e),
        }
    }
}
```

## 🔗 Links
- [IPC Protocol](./IPC-PROTOCOL.md)
- [Architecture](../01-ARCHITECTURE.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
