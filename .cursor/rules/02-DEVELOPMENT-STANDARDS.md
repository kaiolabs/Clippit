# Padrões de Desenvolvimento - Clippit

## 🎯 Filosofia de Desenvolvimento

1. **Código Limpo**: Prefira clareza sobre cleverness
2. **Type Safety**: Use o sistema de tipos Rust ao máximo
3. **Error Handling**: Sempre use `Result<T, E>`, nunca `panic!` em produção
4. **Performance**: Otimize depois de medir, não antes
5. **Documentação**: Código auto-documentado + doc comments
6. **Testing**: TDD quando viável, cobertura mínima 70%

## 📝 Convenções de Código Rust

### Naming Conventions

```rust
// ✅ Correto
mod clipboard_monitor;           // snake_case para módulos
struct ClipboardEntry;            // PascalCase para tipos
trait ContentValidator;           // PascalCase para traits
enum ContentType;                 // PascalCase para enums
fn add_entry();                   // snake_case para funções
const MAX_HISTORY_ITEMS: usize;   // SCREAMING_SNAKE_CASE para constantes
let history_manager = ...;        // snake_case para variáveis

// ❌ Incorreto
mod ClipboardMonitor;             // Não use PascalCase em módulos
struct clipboard_entry;           // Não use snake_case em tipos
const maxHistoryItems: usize;     // Não use camelCase
let HistoryManager = ...;         // Não use PascalCase em variáveis
```

### Estrutura de Módulos

```rust
// crates/clippit-core/src/lib.rs
pub mod config;
pub mod history;
pub mod storage;
pub mod types;
pub mod validator;

// Re-exports principais
pub use config::Config;
pub use history::HistoryManager;
pub use types::{ClipboardEntry, ContentType};
pub use validator::ContentValidator;
```

### Error Handling

```rust
// ✅ Use Result para operações que podem falhar
pub fn load_config() -> Result<Config, Error> {
    let path = config_path()?;  // Propaga erro com ?
    let content = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

// ✅ Use anyhow para errors em aplicações
use anyhow::{Result, Context};

pub fn process() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;
    Ok(())
}

// ✅ Use thiserror para errors em bibliotecas
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Entry not found: {id}")]
    NotFound { id: i64 },
}

// ❌ Evite panic! em código de produção
// panic!("Something went wrong");  // NÃO!

// ❌ Evite unwrap() exceto em testes
// let config = load_config().unwrap();  // NÃO!

// ✅ Em testes, unwrap() é aceitável
#[test]
fn test_config() {
    let config = load_config().unwrap();
    assert_eq!(config.max_items, 100);
}
```

### Documentação

```rust
/// Adiciona uma entrada ao histórico do clipboard.
///
/// Esta função valida o conteúdo, verifica duplicatas usando SHA256,
/// e adiciona ao banco de dados se for único.
///
/// # Arguments
///
/// * `content` - O conteúdo a ser adicionado (texto ou imagem)
/// * `content_type` - O tipo do conteúdo ([`ContentType::Text`] ou [`ContentType::Image`])
///
/// # Returns
///
/// Retorna `Ok(entry_id)` com o ID da entrada inserida, ou `Err` se:
/// - O conteúdo for inválido (muito grande, formato incorreto)
/// - A entrada já existir no histórico (duplicata)
/// - Ocorrer erro de banco de dados
///
/// # Examples
///
/// ```rust
/// use clippit_core::{HistoryManager, ContentType};
///
/// let mut manager = HistoryManager::new()?;
/// let entry_id = manager.add_entry("Hello, world!", ContentType::Text)?;
/// println!("Added entry with ID: {}", entry_id);
/// ```
///
/// # Errors
///
/// Esta função retorna erro se:
/// - Texto exceder 10MB
/// - Imagem exceder 50MB
/// - Formato de imagem não suportado
/// - Falha ao escrever no banco de dados
///
/// # See Also
///
/// - [`validate_content`] para regras de validação
/// - [`get_recent`] para recuperar entradas
pub fn add_entry(&mut self, content: &str, content_type: ContentType) -> Result<i64, StorageError> {
    // Implementação
}
```

### Traits e Generics

```rust
// ✅ Use traits para abstração
pub trait ClipboardProvider {
    fn get_text(&self) -> Result<String>;
    fn get_image(&self) -> Result<Vec<u8>>;
    fn set_content(&self, content: &ClipboardEntry) -> Result<()>;
}

// ✅ Implemente traits comuns
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardEntry {
    // ...
}

// ✅ Use generics quando apropriado
pub struct Cache<T> {
    items: Vec<T>,
    capacity: usize,
}

impl<T: Clone> Cache<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            capacity,
        }
    }
}
```

### Async/Await Patterns

```rust
// ✅ Use async para I/O operations
pub async fn start_ipc_server(history: Arc<Mutex<HistoryManager>>) -> Result<()> {
    let listener = LocalSocketListener::bind("/tmp/clippit.sock")?;
    
    loop {
        let (conn, _) = listener.accept().await?;
        let history = history.clone();
        
        tokio::spawn(async move {
            handle_connection(conn, history).await;
        });
    }
}

// ✅ Use tokio::spawn para tasks concorrentes
tokio::spawn(async move {
    clipboard_monitor::start_monitor(history).await;
});

// ✅ Use Arc<Mutex<>> para shared state
let history = Arc::new(Mutex::new(HistoryManager::new()?));
```

## 🏗️ Estrutura de Arquivos

### Organização de Crate

```
crates/clippit-example/
├── Cargo.toml                  # Dependências e metadata
├── src/
│   ├── lib.rs                  # Entry point (se biblioteca)
│   ├── main.rs                 # Entry point (se binário)
│   ├── module1.rs              # Módulo simples
│   ├── module2/                # Módulo com submódulos
│   │   ├── mod.rs              # Re-exports
│   │   ├── submodule1.rs
│   │   └── submodule2.rs
│   └── error.rs                # Tipos de erro centralizados
├── tests/                      # Testes de integração
│   └── integration_test.rs
└── benches/                    # Benchmarks (opcional)
    └── benchmark.rs
```

### Tamanho de Arquivos

- **Módulos**: Máximo 500 linhas
- **Funções**: Máximo 100 linhas
- **Testes**: Qualquer tamanho razoável

Se exceder, refatore em submódulos/funções.

## 🧪 Testing Standards

### Testes Unitários

```rust
// ✅ Testes no mesmo arquivo do código
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_text_entry() {
        let mut manager = HistoryManager::new_in_memory().unwrap();
        let result = manager.add_entry("test", ContentType::Text);
        assert!(result.is_ok());
        
        let entries = manager.get_recent(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_text.as_deref(), Some("test"));
    }
    
    #[test]
    fn test_duplicate_detection() {
        let mut manager = HistoryManager::new_in_memory().unwrap();
        
        manager.add_entry("duplicate", ContentType::Text).unwrap();
        let result = manager.add_entry("duplicate", ContentType::Text);
        
        // Duplicatas devem ser ignoradas silenciosamente
        assert!(result.is_ok());
        assert_eq!(manager.get_recent(10).unwrap().len(), 1);
    }
    
    #[test]
    #[should_panic(expected = "too large")]
    fn test_oversized_content() {
        let mut manager = HistoryManager::new_in_memory().unwrap();
        let huge_text = "x".repeat(11 * 1024 * 1024); // 11MB
        manager.add_entry(&huge_text, ContentType::Text).unwrap();
    }
}
```

### Testes de Integração

```rust
// tests/ipc_test.rs
use clippit_ipc::{IpcClient, IpcMessage};
use clippit_daemon;

#[tokio::test]
async fn test_ipc_query_history() {
    // Setup: Start daemon
    let daemon = tokio::spawn(async {
        clippit_daemon::start().await
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Test: Query via IPC
    let client = IpcClient::connect().await.unwrap();
    let response = client.send_message(
        IpcMessage::QueryHistory { limit: 10 }
    ).await.unwrap();
    
    assert!(matches!(response, IpcResponse::HistoryResponse { .. }));
    
    // Cleanup
    daemon.abort();
}
```

### Mocks e Helpers

```rust
// Crie helpers para testes comuns
pub fn create_test_entry(text: &str) -> ClipboardEntry {
    ClipboardEntry {
        id: 1,
        content_type: ContentType::Text,
        content_text: Some(text.to_string()),
        content_data: None,
        image_path: None,
        thumbnail_data: None,
        timestamp: Utc::now(),
    }
}

#[cfg(test)]
mod test_helpers {
    pub fn setup_test_db() -> HistoryManager {
        HistoryManager::new_in_memory().unwrap()
    }
}
```

## 🔧 Formatação e Linting

### Rustfmt

```toml
# rustfmt.toml (na raiz do workspace)
edition = "2021"
max_width = 100
tab_spaces = 4
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

### Clippy

```bash
# Execute clippy com todos warnings como erros
cargo clippy --all-targets --all-features -- -D warnings

# Ignore warnings específicos (quando justificado)
#[allow(clippy::too_many_arguments)]  // Justificado: API legacy
```

### Comandos Pre-Commit

```bash
# Antes de cada commit, execute:
cargo fmt --all                           # Formatar código
cargo clippy -- -D warnings                # Linting
cargo test --all                           # Testes
cargo build --release                      # Build de verificação
```

## 📦 Dependências

### Versionamento

```toml
[dependencies]
# ✅ Use versão específica com caret (padrão)
serde = "1.0"              # Aceita 1.x.x (compatível)

# ✅ Use tilde para patches
tokio = "~1.36"            # Aceita 1.36.x apenas

# ✅ Use versão exata quando necessário
some-unstable = "=0.5.2"   # Exatamente esta versão

# ❌ Evite wildcards
# bad-dep = "*"            # NÃO!
```

### Features

```toml
[dependencies]
tokio = { version = "1.36", features = ["full"] }
rusqlite = { version = "0.31", features = ["bundled"] }
arboard = { version = "3.6", features = ["wayland-data-control"] }

[dev-dependencies]
# Dependências apenas para testes
mockall = "0.12"
```

## 🔀 Git Workflow

### Branch Naming

```
feature/nome-da-feature    # Nova funcionalidade
fix/descricao-do-bug       # Correção de bug
refactor/descricao         # Refatoração
docs/descricao             # Documentação
test/descricao             # Testes
chore/descricao            # Manutenção
```

### Commit Messages (Conventional Commits)

```bash
# Formato
<tipo>(<escopo>): <descrição>

[corpo opcional]

[footer opcional]

# Tipos
feat     # Nova feature
fix      # Correção de bug
docs     # Documentação
style    # Formatação (sem mudança de lógica)
refactor # Refatoração
test     # Testes
chore    # Manutenção, dependências

# Exemplos
feat(popup): adiciona suporte a preview de imagem em hover

fix(daemon): corrige memory leak no monitor de clipboard

Closes #42

docs(readme): atualiza instruções de instalação para Ubuntu 24.04

refactor(core): extrai validação para ContentValidator trait

test(ipc): adiciona testes de integração para protocol
```

### Commit Guidelines

1. **Primeira linha**: Máximo 50 caracteres
2. **Corpo**: Máximo 72 caracteres por linha
3. **Idioma**: Português ou inglês (seja consistente no projeto)
4. **Descrição**: Use imperativo ("adiciona", não "adicionado")
5. **Referências**: Cite issues com `#123` ou `Closes #123`

## 🔍 Code Review Checklist

### Para o Autor

- [ ] Código formatado com `cargo fmt`
- [ ] Sem warnings do clippy
- [ ] Testes passando
- [ ] Documentação atualizada
- [ ] CHANGELOG.md atualizado (se feature/fix)
- [ ] Commit messages seguem padrão
- [ ] PR description clara

### Para o Reviewer

- [ ] Código segue convenções do projeto
- [ ] Lógica está clara e correta
- [ ] Tratamento de erros adequado
- [ ] Testes cobrem casos importantes
- [ ] Sem problemas de performance óbvios
- [ ] Documentação suficiente
- [ ] Mudanças são necessárias (sem over-engineering)

## ⚠️ Anti-Patterns a Evitar

### ❌ Unwrap sem Justificativa

```rust
// ❌ Ruim
let config = Config::load().unwrap();

// ✅ Bom
let config = Config::load()
    .context("Failed to load config")?;
```

### ❌ Clone Desnecessário

```rust
// ❌ Ruim (clona string toda vez)
fn process(text: String) -> String {
    text.to_uppercase()
}

// ✅ Bom (usa referência)
fn process(text: &str) -> String {
    text.to_uppercase()
}
```

### ❌ Strings Alocadas Desnecessariamente

```rust
// ❌ Ruim
if entry.content_type == "Text".to_string() { }

// ✅ Bom
if entry.content_type == ContentType::Text { }
```

### ❌ Mutabilidade Excessiva

```rust
// ❌ Ruim
let mut result = vec![];
result.push(1);
result.push(2);

// ✅ Bom
let result = vec![1, 2];
```

### ❌ Nested Matches Profundos

```rust
// ❌ Ruim
match result {
    Ok(value) => {
        match value {
            Some(inner) => {
                match process(inner) {
                    // ...
                }
            }
        }
    }
}

// ✅ Bom (use ?, if let, early return)
let value = result?;
let Some(inner) = value else { return Ok(()); };
let processed = process(inner)?;
```

## 📚 Recursos de Referência

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [The Rust Book](https://doc.rust-lang.org/book/)

## 🔗 Links Relacionados

- [Visão Geral](./00-PROJECT-OVERVIEW.md)
- [Arquitetura](./01-ARCHITECTURE.md)
- [Core Patterns](./core/CORE-OVERVIEW.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
