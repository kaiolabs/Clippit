# Unit Tests

## 🎯 Escopo

Testes unitários focam em **funções e métodos individuais** isoladamente.

## 📝 Padrões

### Localização
```rust
// No mesmo arquivo do código
// crates/clippit-core/src/history.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_text_entry() {
        // ...
    }
}
```

### Nomenclatura
```rust
#[test]
fn test_<função>_<cenário>_<resultado_esperado>()
```

**Exemplos**:
- `test_add_entry_valid_text_success()`
- `test_add_entry_duplicate_ignored()`
- `test_validate_text_too_large_error()`

### Estrutura AAA
```rust
#[test]
fn test_example() {
    // Arrange (setup)
    let mut manager = HistoryManager::new_in_memory().unwrap();
    let text = "test content";
    
    // Act (execute)
    let result = manager.add_entry(text, ContentType::Text);
    
    // Assert (verify)
    assert!(result.is_ok());
    let entries = manager.get_recent(10).unwrap();
    assert_eq!(entries.len(), 1);
}
```

## 📋 Exemplos

### Core - Validation
```rust
#[test]
fn test_validate_text_empty_error() {
    let result = ContentValidator::validate_text("");
    assert!(matches!(result, Err(ValidationError::TextEmpty)));
}

#[test]
fn test_validate_text_too_large() {
    let huge = "x".repeat(11 * 1024 * 1024);
    let result = ContentValidator::validate_text(&huge);
    assert!(matches!(result, Err(ValidationError::TextTooLarge { .. })));
}
```

### Core - Config
```rust
#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert_eq!(config.general.max_history_items, 100);
    assert_eq!(config.general.poll_interval_ms, 200);
}

#[test]
fn test_config_roundtrip() {
    let config = Config::default();
    let toml = toml::to_string(&config).unwrap();
    let parsed: Config = toml::from_str(&toml).unwrap();
    assert!(parsed.validate().is_ok());
}
```

### Core - History
```rust
#[test]
fn test_duplicate_detection() {
    let mut manager = HistoryManager::new_in_memory().unwrap();
    
    manager.add_entry("dup", ContentType::Text).unwrap();
    manager.add_entry("dup", ContentType::Text).unwrap();
    
    let entries = manager.get_recent(10).unwrap();
    assert_eq!(entries.len(), 1);
}
```

## 🔗 Links
- [Testing Strategy](./TESTING-STRATEGY.md)
- [Development Standards](../02-DEVELOPMENT-STANDARDS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
