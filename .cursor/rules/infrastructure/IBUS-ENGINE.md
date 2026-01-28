# IBus Engine

## 📍 Localização
`crates/clippit-ibus/src/`

## 🎯 Responsabilidade
Engine IBus para captura de digitação e autocomplete global via Input Method.

## 🏗️ Estrutura

```rust
pub struct ClippitEngine {
    typing_buffer: Arc<Mutex<TypingBuffer>>,
    connection: Connection,
    ipc_client: IpcClient,
    enabled: Arc<Mutex<bool>>,
}

impl ClippitEngine {
    pub fn new() -> Result<Self>;
    pub fn run() -> Result<()>;
    pub fn process_key_press(keyval, keycode, state) -> Result<bool>;
    pub fn enable();
    pub fn disable();
}
```

## 🔄 Fluxo

1. IBus Framework recebe keystroke
2. ClippitEngine::process_key_press()
3. Atualiza TypingBuffer
4. Se palavra >= 2 chars, IPC RequestAutocompleteSuggestions
5. Daemon responde com sugestões
6. Exibe popup flutuante

## 📝 DBus Interface

**Component**: `org.clippit.IBus.Clippit`
**Engine**: `clippit`
**XML**: `data/clippit.xml`

## 🔗 Links
- [Typing Monitor](../daemon/TYPING-AUTOCOMPLETE.md)
- [Autocomplete Feature](../features/AUTOCOMPLETE-GLOBAL.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
