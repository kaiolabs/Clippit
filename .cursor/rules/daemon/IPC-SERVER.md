# clippit-daemon - IPC Server

## 📍 Localização
`crates/clippit-daemon/src/main.rs` (função `handle_ipc_message`)

## 🎯 Responsabilidade

Servidor IPC que processa requisições de clientes (popup, dashboard, ibus) via Unix socket.

## 🔄 Fluxo

```rust
async fn run_ipc_server(history: Arc<Mutex<HistoryManager>>) -> Result<()> {
    let socket_path = "/tmp/clippit.sock";
    
    // Remove socket antigo
    let _ = fs::remove_file(socket_path);
    
    let listener = LocalSocketListener::bind(socket_path)?;
    
    loop {
        match listener.accept().await {
            Ok((conn, _)) => {
                let history = history.clone();
                tokio::spawn(async move {
                    handle_connection(conn, history).await;
                });
            }
            Err(e) => {
                tracing::error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(
    mut conn: LocalSocketStream,
    history: Arc<Mutex<HistoryManager>>
) {
    let mut reader = BufReader::new(&mut conn);
    let mut line = String::new();
    
    while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
        if let Ok(msg) = serde_json::from_str::<IpcMessage>(&line) {
            let response = handle_ipc_message(msg, &history).await;
            let response_json = serde_json::to_string(&response).unwrap();
            
            conn.write_all(response_json.as_bytes()).await.ok();
            conn.write_all(b"\n").await.ok();
        }
        
        line.clear();
    }
}
```

## 📨 Message Handler

```rust
async fn handle_ipc_message(
    msg: IpcMessage,
    history: &Arc<Mutex<HistoryManager>>
) -> IpcResponse {
    match msg {
        IpcMessage::Ping => {
            IpcResponse::Pong
        }
        
        IpcMessage::QueryHistory { limit } => {
            let h = history.lock().unwrap();
            match h.get_recent(limit) {
                Ok(entries) => IpcResponse::HistoryResponse { entries },
                Err(e) => IpcResponse::Error { 
                    message: e.to_string() 
                },
            }
        }
        
        IpcMessage::QueryHistoryMetadata { limit, offset } => {
            let h = history.lock().unwrap();
            match h.get_recent_metadata_with_offset(limit, offset) {
                Ok(entries) => IpcResponse::HistoryMetadataResponse { entries },
                Err(e) => IpcResponse::Error { 
                    message: e.to_string() 
                },
            }
        }
        
        IpcMessage::SearchHistory { query } => {
            let h = history.lock().unwrap();
            match h.search(&query) {
                Ok(entries) => IpcResponse::SearchHistoryResponse { entries },
                Err(e) => IpcResponse::Error { 
                    message: e.to_string() 
                },
            }
        }
        
        IpcMessage::GetEntryData { id } => {
            let h = history.lock().unwrap();
            match h.get_by_id(id) {
                Ok(Some(entry)) => IpcResponse::EntryDataResponse { entry },
                Ok(None) => IpcResponse::Error { 
                    message: format!("Entry {} not found", id) 
                },
                Err(e) => IpcResponse::Error { 
                    message: e.to_string() 
                },
            }
        }
        
        IpcMessage::SelectItem { id } => {
            // Apenas acknowledge
            IpcResponse::Ok
        }
        
        IpcMessage::RequestAutocompleteSuggestions { 
            partial_word, 
            max_results,
            .. 
        } => {
            let h = history.lock().unwrap();
            let suggestions = generate_suggestions(&h, &partial_word, max_results);
            
            IpcResponse::AutocompleteSuggestions {
                suggestions,
                query: partial_word,
            }
        }
        
        _ => {
            IpcResponse::Error { 
                message: "Unsupported message".to_string() 
            }
        }
    }
}
```

## 📝 Protocol

**Formato**: JSON linha por linha
**Socket**: `/tmp/clippit.sock` (Unix Domain Socket)
**Encoding**: UTF-8

**Request**:
```json
{"QueryHistory":{"limit":30}}\n
```

**Response**:
```json
{"HistoryResponse":{"entries":[...]}}\n
```

## 🔗 Links
- [Daemon Overview](./DAEMON-OVERVIEW.md)
- [IPC Protocol](../infrastructure/IPC-PROTOCOL.md)
- [Architecture](../01-ARCHITECTURE.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
