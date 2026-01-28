# Testing Strategy

## 🎯 Estratégia de Testes

### Pirâmide de Testes
```
       /\
      /  \     E2E Tests (poucos)
     /----\
    /      \   Integration Tests (alguns)
   /--------\
  /          \ Unit Tests (muitos)
 /____________\
```

### Objetivos
- ✅ **70%+ cobertura** de código crítico
- ✅ **Testes rápidos** (< 5s para suite completa)
- ✅ **Determinísticos** (sem flakiness)
- ✅ **Isolados** (sem dependências externas)

## 📋 Tipos de Testes

### 1. Unit Tests
- **O quê**: Funções individuais, métodos
- **Onde**: `#[cfg(test)] mod tests` no próprio arquivo
- **Ferramentas**: Built-in Rust test framework
- **Exemplo**: Validação, parsing, transformações

### 2. Integration Tests
- **O quê**: Interação entre módulos
- **Onde**: `tests/` directory
- **Ferramentas**: cargo test
- **Exemplo**: IPC client-server, HistoryManager + Storage

### 3. E2E Tests
- **O quê**: Fluxos completos
- **Onde**: Manual ou scripts
- **Ferramentas**: Shell scripts
- **Exemplo**: Daemon → capture → popup → copy

## 🔧 Ferramentas

### Cargo Test
```bash
# Todos os testes
cargo test

# Crate específico
cargo test -p clippit-core

# Teste específico
cargo test test_add_entry

# Com output
cargo test -- --nocapture

# Apenas testes que passam
cargo test -- --quiet
```

### Coverage
```bash
# Com tarpaulin
cargo tarpaulin --out Html
```

### Benchmarks
```bash
cargo bench
```

## ✅ Regras

### 1. In-Memory para Testes
```rust
let manager = HistoryManager::new_in_memory()?;
```

### 2. Mock Externo
```rust
#[cfg(test)]
mod tests {
    fn mock_ipc_client() -> IpcClient {
        // Mock implementation
    }
}
```

### 3. Fixtures
```rust
fn create_test_entry() -> ClipboardEntry {
    ClipboardEntry::new_text("test".to_string())
}
```

## 🔗 Links
- [Unit Tests](./UNIT-TESTS.md)
- [Integration Tests](./INTEGRATION-TESTS.md)
- [Development Standards](../02-DEVELOPMENT-STANDARDS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
