# Integration Tests

## 🎯 Escopo

Testes de integração verificam **interação entre módulos** e componentes.

## 📝 Localização

```
crates/clippit-*/tests/
├── integration_test.rs
└── ipc_test.rs
```

## 📋 Exemplos

### IPC Client-Server
```rust
// tests/ipc_test.rs

#[tokio::test]
async fn test_ipc_ping_pong() {
    // Start daemon
    let daemon = tokio::spawn(async {
        clippit_daemon::start().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test IPC
    let mut client = IpcClient::connect().unwrap();
    let response = client.send_message(IpcMessage::Ping).unwrap();
    
    assert!(matches!(response, IpcResponse::Pong));
    
    daemon.abort();
}

#[tokio::test]
async fn test_ipc_query_history() {
    // Setup daemon with data
    let daemon = start_test_daemon().await;
    
    // Query via IPC
    let mut client = IpcClient::connect().unwrap();
    let response = client.send_message(
        IpcMessage::QueryHistory { limit: 10 }
    ).unwrap();
    
    if let IpcResponse::HistoryResponse { entries } = response {
        assert!(entries.len() <= 10);
    } else {
        panic!("Wrong response type");
    }
}
```

### History + Storage
```rust
#[test]
fn test_history_storage_integration() {
    let mut manager = HistoryManager::new_in_memory().unwrap();
    
    // Add multiple entries
    for i in 0..5 {
        manager.add_entry(&format!("entry {}", i), ContentType::Text).unwrap();
    }
    
    // Query
    let entries = manager.get_recent(10).unwrap();
    assert_eq!(entries.len(), 5);
    
    // Search
    let results = manager.search("entry 2").unwrap();
    assert_eq!(results.len(), 1);
}
```

### Config + Validation
```rust
#[test]
fn test_config_validation_integration() {
    let mut config = Config::default();
    
    // Invalid value
    config.general.max_history_items = 0;
    assert!(config.validate().is_err());
    
    // Valid value
    config.general.max_history_items = 50;
    assert!(config.validate().is_ok());
}
```

## 🔧 Helpers

```rust
// tests/common/mod.rs

pub fn start_test_daemon() -> JoinHandle<()> {
    tokio::spawn(async {
        let history = Arc::new(Mutex::new(
            HistoryManager::new_in_memory().unwrap()
        ));
        
        ipc_server::run(history).await.unwrap();
    })
}

pub fn create_test_entries(n: usize) -> Vec<ClipboardEntry> {
    (0..n)
        .map(|i| ClipboardEntry::new_text(format!("test {}", i)))
        .collect()
}
```

## 🔗 Links
- [Testing Strategy](./TESTING-STRATEGY.md)
- [Unit Tests](./UNIT-TESTS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
