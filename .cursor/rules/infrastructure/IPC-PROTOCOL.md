# IPC Protocol

## 📍 Localização
`crates/clippit-ipc/src/protocol.rs`

## 🎯 Responsabilidade
Define protocolo de comunicação entre daemon e clientes.

## 📨 Mensagens (IpcMessage)

```rust
pub enum IpcMessage {
    Ping,
    QueryHistory { limit: usize },
    QueryHistoryMetadata { limit: usize, offset: usize },
    SearchHistory { query: String },
    GetEntryData { id: i64 },
    SelectItem { id: i64 },
    RequestAutocompleteSuggestions {
        partial_word: String,
        context: AppContext,
        max_results: usize,
    },
    AcceptSuggestion {
        suggestion: String,
        partial_word: String,
    },
}
```

## 📬 Respostas (IpcResponse)

```rust
pub enum IpcResponse {
    Ok,
    Pong,
    Error { message: String },
    HistoryResponse { entries: Vec<HistoryEntry> },
    HistoryMetadataResponse { entries: Vec<HistoryEntry> },
    SearchHistoryResponse { entries: Vec<HistoryEntry> },
    EntryDataResponse { entry: HistoryEntry },
    AutocompleteSuggestions {
        suggestions: Vec<Suggestion>,
        query: String,
    },
}
```

## 🔌 Transport
- **Socket**: Unix Domain Socket `/tmp/clippit.sock`
- **Format**: JSON linha por linha (`\n` delimited)
- **Encoding**: UTF-8

## 🔗 Links
- [Architecture](../01-ARCHITECTURE.md)
- [IPC Server](../daemon/IPC-SERVER.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
