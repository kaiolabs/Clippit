# clippit-core - Overview

## 📍 Localização
`crates/clippit-core/`

## 🎯 Responsabilidade

**clippit-core** é a biblioteca compartilhada que contém toda a **lógica de negócio** do Clippit. É utilizada por todos os outros crates do projeto (daemon, popup, dashboard, ibus, etc).

### Princípios
- ✅ **Zero dependências de UI**: Não depende de GTK, Qt, ou frameworks de apresentação
- ✅ **Pure Business Logic**: Apenas regras de negócio, tipos e utilitários
- ✅ **Reusabilidade**: Código compartilhado entre todos os componentes
- ✅ **Testabilidade**: Fácil de testar isoladamente (suporta banco in-memory)

## 📦 Estrutura de Módulos

```
crates/clippit-core/
├── Cargo.toml
├── locales/
│   ├── en.yml          # Traduções inglês
│   └── pt.yml          # Traduções português
└── src/
    ├── lib.rs          # Entry point, re-exports
    ├── config.rs       # Sistema de configuração TOML
    ├── history.rs      # HistoryManager (lógica de histórico)
    ├── storage.rs      # Storage (camada SQLite)
    ├── types.rs        # Tipos principais (ClipboardEntry, etc)
    └── validator.rs    # ContentValidator (validação)
```

## 🔧 Módulos Principais

### 1. **lib.rs** - Entry Point

```rust
pub mod config;
pub mod history;
pub mod storage;
pub mod types;
pub mod validator;

// Re-exports para API pública
pub use config::Config;
pub use history::HistoryManager;
pub use storage::Storage;
pub use types::{ClipboardEntry, ContentType};
pub use validator::ContentValidator;

// Inicialização i18n
rust_i18n::i18n!("locales", fallback = "en");

/// Define o idioma da aplicação
pub fn set_language(lang: &str) {
    rust_i18n::set_locale(lang);
}
```

**Responsabilidade**: Expor API pública consistente.

### 2. **config.rs** - Configuração

```rust
// Hierarquia de configuração
Config
├── GeneralConfig
├── HotkeyConfig
├── UiConfig
├── SearchConfig
├── FeaturesConfig
├── PrivacyConfig
├── AdvancedConfig
└── AutocompleteConfig
    └── AutocompleteAIConfig
```

**Ver**: [CONFIG-PATTERNS.md](./CONFIG-PATTERNS.md)

### 3. **history.rs** - Gerenciador de Histórico

```rust
pub struct HistoryManager {
    storage: Storage,
    last_hash: Option<String>,
    max_entries: usize,
}
```

**Responsabilidades**:
- Validação de entradas
- Detecção de duplicatas (SHA256)
- Pruning automático
- Busca e listagem

**Ver**: [HISTORY-STORAGE.md](./HISTORY-STORAGE.md)

### 4. **storage.rs** - Camada de Persistência

```rust
pub struct Storage {
    conn: Connection,  // rusqlite
}
```

**Responsabilidades**:
- Abstração SQLite
- Schema management
- CRUD operations
- Queries otimizadas

**Ver**: [HISTORY-STORAGE.md](./HISTORY-STORAGE.md)

### 5. **types.rs** - Tipos de Dados

```rust
pub enum ContentType { Text, Image }

pub struct ClipboardEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: Option<String>,
    pub image_path: Option<String>,
    pub thumbnail_data: Option<Vec<u8>>,
    pub timestamp: DateTime<Utc>,
}
```

**Ver**: [TYPES-DEFINITIONS.md](./TYPES-DEFINITIONS.md)

### 6. **validator.rs** - Validação de Conteúdo

```rust
pub struct ContentValidator;

impl ContentValidator {
    pub fn validate_text(text: &str) -> Result<(), ValidationError>;
    pub fn validate_image(data: &[u8]) -> Result<(), ValidationError>;
}
```

**Ver**: [VALIDATION.md](./VALIDATION.md)

## 📊 Dependências

```toml
[dependencies]
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
chrono = { version = "0.4", features = ["serde"] }
sha2 = "0.10"
image = "0.25"
anyhow = "1.0"
thiserror = "1.0"
rust-i18n = "3.0"
dirs = "5.0"
tracing = "0.1"
```

**Justificativas**:
- `rusqlite`: Banco de dados embarcado, zero configuração
- `serde` + `toml`: Serialização/deserialização de config
- `chrono`: Timestamps UTC
- `sha2`: Hash para deduplicação
- `image`: Validação de formatos, resize, thumbnails
- `rust-i18n`: Sistema de tradução

## 🔄 Fluxo de Uso Típico

### Inicialização

```rust
use clippit_core::{HistoryManager, Config};

// Carregar configuração
let config = Config::load()
    .unwrap_or_else(|_| Config::default());

// Criar gerenciador de histórico
let manager = HistoryManager::new(
    config.general.max_history_items
)?;
```

### Adicionar Entrada

```rust
// Texto
manager.add_entry("Conteúdo copiado", ContentType::Text)?;

// Imagem
manager.add_entry_with_image(
    image_path,
    thumbnail_bytes,
    ContentType::Image
)?;
```

### Buscar Histórico

```rust
// Últimos 100 itens (metadata apenas)
let entries = manager.get_recent_metadata(100)?;

// Buscar por query
let results = manager.search("rust")?;

// Obter entrada específica (com dados completos)
let entry = manager.get_by_id(42)?;
```

### Limpeza

```rust
// Deletar item específico
manager.delete_by_id(42)?;

// Limpar tudo
manager.clear()?;

// Pruning automático (chamado internamente)
// Remove itens mais antigos quando > max_entries
```

## 🎨 Padrões de Design

### 1. Repository Pattern
`HistoryManager` abstrai acesso ao `Storage`, que abstrai SQLite.

```
[Usuário] → [HistoryManager] → [Storage] → [SQLite]
```

### 2. Strategy Pattern
`ContentValidator` implementa diferentes estratégias de validação por tipo.

### 3. Factory Pattern
Construtores `new()` e `new_in_memory()` para diferentes ambientes.

### 4. Builder Pattern
`Config` com defaults via funções `default_*()`.

### 5. Newtype Pattern
Wrapping de tipos primitivos para type safety.

## ✅ Regras de Implementação

### 1. Sempre Use Result

```rust
// ✅ Correto
pub fn add_entry(&mut self, content: &str) -> Result<i64, Error>;

// ❌ Incorreto
pub fn add_entry(&mut self, content: &str) -> i64;  // E se falhar?
```

### 2. Validação Antes de Persistir

```rust
// Sempre valide antes de salvar
ContentValidator::validate_text(text)?;
let hash = compute_hash(text);
storage.insert(entry)?;
```

### 3. Defaults Para Tudo

```rust
#[derive(Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_max_history_items")]
    pub max_history_items: usize,
}

fn default_max_history_items() -> usize { 100 }
```

### 4. Paths Portáveis

```rust
// ✅ Use dirs crate
let data_dir = dirs::data_local_dir()
    .ok_or_else(|| anyhow!("No data dir"))?
    .join("clippit");

// ❌ Não hardcode
let data_dir = "/home/user/.local/share/clippit";  // NÃO!
```

### 5. Testes com In-Memory DB

```rust
#[test]
fn test_add_entry() {
    let mut manager = HistoryManager::new_in_memory().unwrap();
    let result = manager.add_entry("test", ContentType::Text);
    assert!(result.is_ok());
}
```

## 🚫 Anti-Patterns

❌ **UI Logic no Core**
```rust
// NÃO faça isso no clippit-core
use gtk4::prelude::*;  // NÃO!
```

❌ **Dependências Pesadas**
```rust
// Evite dependências grandes desnecessárias
// Como tokio (a menos que realmente precise)
```

❌ **Global State Mutável**
```rust
// NÃO use static mut
static mut GLOBAL_CONFIG: Option<Config> = None;  // NÃO!
```

❌ **Panic em Produção**
```rust
// Use Result, não panic
// panic!("Failed to load config");  // NÃO!
```

## 📝 Checklist para Novos Recursos

Ao adicionar funcionalidade ao `clippit-core`:

- [ ] Define tipos apropriados em `types.rs`
- [ ] Adiciona validação em `validator.rs` (se aplicável)
- [ ] Atualiza `storage.rs` se precisar de novas queries
- [ ] Atualiza `history.rs` para lógica de negócio
- [ ] Adiciona ao `Config` se for configurável
- [ ] Escreve testes unitários
- [ ] Atualiza documentação
- [ ] Verifica que não adicionou dependências de UI

## 🔗 Links Relacionados

- **Configuração**: [CONFIG-PATTERNS.md](./CONFIG-PATTERNS.md)
- **Histórico**: [HISTORY-STORAGE.md](./HISTORY-STORAGE.md)
- **Tipos**: [TYPES-DEFINITIONS.md](./TYPES-DEFINITIONS.md)
- **Validação**: [VALIDATION.md](./VALIDATION.md)
- **Visão Geral**: [../00-PROJECT-OVERVIEW.md](../00-PROJECT-OVERVIEW.md)
- **Arquitetura**: [../01-ARCHITECTURE.md](../01-ARCHITECTURE.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
