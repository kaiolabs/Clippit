# clippit-core - História e Storage

## 📍 Localização
- `crates/clippit-core/src/history.rs`
- `crates/clippit-core/src/storage.rs`

## 🎯 Responsabilidade

### HistoryManager
Gerencia a lógica de negócio do histórico: validação, deduplicação, pruning.

### Storage
Abstrai acesso ao banco SQLite: CRUD operations, queries otimizadas.

## 📊 Arquitetura

```
┌─────────────────────────────────────┐
│      HistoryManager                 │
│  ┌────────────────────────────┐    │
│  │ Business Logic             │    │
│  │ - Validation               │    │
│  │ - Deduplication (SHA256)   │    │
│  │ - Pruning                  │    │
│  └────────────┬───────────────┘    │
└───────────────┼────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│         Storage                     │
│  ┌────────────────────────────┐    │
│  │ Data Access                │    │
│  │ - CRUD operations          │    │
│  │ - Queries (recent, search) │    │
│  │ - Schema management        │    │
│  └────────────┬───────────────┘    │
└───────────────┼────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│        SQLite Database              │
│  ~/.local/share/clippit/history.db │
└─────────────────────────────────────┘
```

## 🗄️ Schema SQLite

```sql
CREATE TABLE clipboard_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,           -- "Text" ou "Image"
    content_text TEXT,                    -- Conteúdo textual
    content_data BLOB,                    -- DEPRECATED (legacy)
    image_path TEXT,                      -- Path: ~/.local/share/clippit/images/{hash}.png
    thumbnail_data BLOB,                  -- Thumbnail PNG 128x128
    timestamp TEXT NOT NULL               -- RFC3339 UTC: "2026-01-28T10:30:00Z"
);

CREATE INDEX idx_timestamp ON clipboard_history(timestamp DESC);
```

## 💾 Storage (storage.rs)

### Estrutura

```rust
pub struct Storage {
    conn: Connection,  // rusqlite::Connection
}

impl Storage {
    /// Cria storage com banco em arquivo
    pub fn new(db_path: &Path) -> Result<Self>;
    
    /// Cria storage com banco in-memory (testes)
    pub fn in_memory() -> Result<Self>;
    
    /// Insere nova entrada
    pub fn insert(&self, entry: &ClipboardEntry) -> Result<i64>;
    
    /// Obtém entradas recentes (com dados completos)
    pub fn get_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém metadata recentes (sem content_data de imagens)
    pub fn get_recent_metadata(&self, limit: usize) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém metadata com offset (paginação)
    pub fn get_recent_metadata_with_offset(
        &self,
        limit: usize,
        offset: usize
    ) -> Result<Vec<ClipboardEntry>>;
    
    /// Busca por query
    pub fn search(&self, query: &str) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém entrada por ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<ClipboardEntry>>;
    
    /// Deleta entrada por ID
    pub fn delete_by_id(&self, id: i64) -> Result<()>;
    
    /// Remove entradas antigas (mantém apenas `keep` mais recentes)
    pub fn prune_old(&self, keep: usize) -> Result<()>;
    
    /// Limpa tudo
    pub fn clear(&self) -> Result<()>;
}
```

### Inicialização do Schema

```rust
impl Storage {
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                content_text TEXT,
                content_data BLOB,
                image_path TEXT,
                thumbnail_data BLOB,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp 
             ON clipboard_history(timestamp DESC)",
            [],
        )?;
        
        Ok(())
    }
}
```

### Query Otimizadas

```rust
// Metadata query (SEM content_data para economizar memória)
pub fn get_recent_metadata(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
    let mut stmt = self.conn.prepare(
        "SELECT 
            id, 
            content_type, 
            content_text, 
            CASE WHEN content_type = 'Image' THEN NULL ELSE content_data END as content_data,
            image_path, 
            thumbnail_data, 
            timestamp
         FROM clipboard_history
         ORDER BY timestamp DESC
         LIMIT ?1"
    )?;
    
    // ...
}

// Search query (LIKE em text e path)
pub fn search(&self, query: &str) -> Result<Vec<ClipboardEntry>> {
    let pattern = format!("%{}%", query);
    
    let mut stmt = self.conn.prepare(
        "SELECT * FROM clipboard_history
         WHERE content_text LIKE ?1 OR image_path LIKE ?1
         ORDER BY timestamp DESC"
    )?;
    
    // ...
}
```

## 📦 HistoryManager (history.rs)

### Estrutura

```rust
pub struct HistoryManager {
    storage: Storage,
    last_hash: Option<String>,    // Hash SHA256 do último item
    max_entries: usize,            // Máximo de entradas (pruning)
}

impl HistoryManager {
    /// Cria manager com banco em arquivo
    pub fn new(max_entries: usize) -> Result<Self>;
    
    /// Cria manager com banco in-memory (testes)
    pub fn new_in_memory() -> Result<Self>;
    
    /// Adiciona entrada (valida, deduplica, faz pruning)
    pub fn add_entry(
        &mut self,
        content: &str,
        content_type: ContentType
    ) -> Result<i64>;
    
    /// Adiciona entrada de imagem
    pub fn add_entry_with_image(
        &mut self,
        image_path: String,
        thumbnail_data: Vec<u8>,
        content_type: ContentType
    ) -> Result<i64>;
    
    /// Obtém entradas recentes
    pub fn get_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém metadata recentes (otimizado)
    pub fn get_recent_metadata(&self, limit: usize) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém metadata com offset (scroll infinito)
    pub fn get_recent_metadata_with_offset(
        &self,
        limit: usize,
        offset: usize
    ) -> Result<Vec<ClipboardEntry>>;
    
    /// Busca
    pub fn search(&self, query: &str) -> Result<Vec<ClipboardEntry>>;
    
    /// Obtém por ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<ClipboardEntry>>;
    
    /// Deleta por ID
    pub fn delete_by_id(&mut self, id: i64) -> Result<()>;
    
    /// Limpa tudo
    pub fn clear(&mut self) -> Result<()>;
}
```

### Deduplicação com SHA256

```rust
impl HistoryManager {
    fn compute_hash(&self, entry: &ClipboardEntry) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        
        match entry.content_type {
            ContentType::Text => {
                hasher.update(b"text:");
                if let Some(text) = &entry.content_text {
                    hasher.update(text.as_bytes());
                }
            }
            ContentType::Image => {
                hasher.update(b"image:path:");
                if let Some(path) = &entry.image_path {
                    hasher.update(path.as_bytes());
                }
            }
        }
        
        format!("{:x}", hasher.finalize())
    }
    
    fn is_duplicate(&self, hash: &str) -> Result<bool> {
        // Verifica se hash existe nos últimos 10 itens
        let recent = self.storage.get_recent_metadata(10)?;
        
        for entry in recent {
            let entry_hash = self.compute_hash(&entry);
            if entry_hash == hash {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
}
```

### Add Entry com Validação

```rust
pub fn add_entry(
    &mut self,
    content: &str,
    content_type: ContentType
) -> Result<i64> {
    // 1. Validar conteúdo
    ContentValidator::validate_text(content)?;
    
    // 2. Criar entrada
    let entry = ClipboardEntry {
        id: 0,  // Será atribuído pelo banco
        content_type,
        content_text: Some(content.to_string()),
        content_data: None,
        image_path: None,
        thumbnail_data: None,
        timestamp: Utc::now(),
    };
    
    // 3. Verificar duplicata
    let hash = self.compute_hash(&entry);
    if self.is_duplicate(&hash)? {
        // Silenciosamente ignora duplicatas
        return Ok(0);
    }
    
    // 4. Inserir
    let entry_id = self.storage.insert(&entry)?;
    
    // 5. Atualizar último hash
    self.last_hash = Some(hash);
    
    // 6. Fazer pruning se necessário
    if self.count()? > self.max_entries {
        self.storage.prune_old(self.max_entries)?;
    }
    
    Ok(entry_id)
}
```

### Pruning Automático

```rust
// Em Storage
pub fn prune_old(&self, keep: usize) -> Result<()> {
    // Mantém apenas as `keep` entradas mais recentes
    self.conn.execute(
        "DELETE FROM clipboard_history
         WHERE id NOT IN (
             SELECT id FROM clipboard_history
             ORDER BY timestamp DESC
             LIMIT ?1
         )",
        params![keep],
    )?;
    
    Ok(())
}
```

## 🔄 Fluxo de Uso Típico

### Daemon: Adicionar Entrada

```rust
let mut manager = HistoryManager::new(100)?;

// Capturou texto
let text = clipboard.get_text()?;
manager.add_entry(&text, ContentType::Text)?;

// Capturou imagem
let image_data = clipboard.get_image()?;
let hash = compute_sha256(&image_data);
let image_path = save_image_to_disk(&image_data, &hash)?;
let thumbnail = create_thumbnail(&image_data)?;
manager.add_entry_with_image(
    image_path,
    thumbnail,
    ContentType::Image
)?;
```

### Popup: Carregar Histórico

```rust
let manager = HistoryManager::new(100)?;

// Carregamento inicial (30 itens, metadata apenas)
let entries = manager.get_recent_metadata(30)?;

// Infinite scroll (próximos 20)
let more_entries = manager.get_recent_metadata_with_offset(20, 30)?;

// Busca
let results = manager.search("rust")?;

// Carregar imagem completa quando necessário
if let Some(entry) = manager.get_by_id(42)? {
    if let Some(path) = entry.image_path {
        let image_bytes = fs::read(&path)?;
        // Exibir imagem
    }
}
```

## ✅ Regras de Implementação

### 1. Sempre Valide Antes de Inserir

```rust
// ✅ Correto
ContentValidator::validate_text(text)?;
storage.insert(&entry)?;

// ❌ Incorreto
storage.insert(&entry)?;  // E se for inválido?
```

### 2. Use Prepared Statements

```rust
// ✅ Correto (safe contra SQL injection)
conn.execute(
    "INSERT INTO clipboard_history (content_text) VALUES (?1)",
    params![text],
)?;

// ❌ NUNCA faça isso
let query = format!("INSERT INTO ... VALUES ('{}')", text);  // SQL INJECTION!
```

### 3. Metadata Queries Para Listagens

```rust
// ✅ Para listagens (economiza memória)
let entries = manager.get_recent_metadata(100)?;

// ❌ Para listagens (carrega BLOBs desnecessários)
let entries = manager.get_recent(100)?;

// ✅ Carregue dados completos apenas quando necessário
let full_entry = manager.get_by_id(selected_id)?;
```

### 4. Pruning Automático

```rust
// Sempre após inserir
let entry_id = storage.insert(&entry)?;

if self.count()? > self.max_entries {
    storage.prune_old(self.max_entries)?;
}
```

### 5. Testes com In-Memory

```rust
#[test]
fn test_add_and_retrieve() {
    let mut manager = HistoryManager::new_in_memory().unwrap();
    manager.add_entry("test", ContentType::Text).unwrap();
    
    let entries = manager.get_recent(10).unwrap();
    assert_eq!(entries.len(), 1);
}
```

## 🚫 Anti-Patterns

### ❌ String Concatenation em SQL

```rust
// NUNCA!
let query = format!("SELECT * FROM history WHERE text LIKE '%{}%'", user_input);
```

### ❌ Carregar TUDO na Memória

```rust
// Ruim para 10k+ entradas
let all_entries = manager.get_recent(100000)?;
```

### ❌ Não Fazer Pruning

```rust
// Banco crescerá infinitamente
storage.insert(&entry)?;
// Esqueceu de fazer pruning!
```

### ❌ Ignorar Erros de DB

```rust
// NÃO!
storage.insert(&entry).ok();  // Ignora erro silenciosamente
```

## 🧪 Exemplos de Testes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_duplicate_detection() {
        let mut manager = HistoryManager::new_in_memory().unwrap();
        
        manager.add_entry("duplicate", ContentType::Text).unwrap();
        manager.add_entry("duplicate", ContentType::Text).unwrap();
        
        let entries = manager.get_recent(10).unwrap();
        assert_eq!(entries.len(), 1);  // Apenas uma entrada
    }
    
    #[test]
    fn test_pruning() {
        let mut manager = HistoryManager::new_in_memory_with_max(5).unwrap();
        
        for i in 0..10 {
            manager.add_entry(&format!("entry {}", i), ContentType::Text).unwrap();
        }
        
        let entries = manager.get_recent(20).unwrap();
        assert_eq!(entries.len(), 5);  // Mantém apenas 5
    }
    
    #[test]
    fn test_search() {
        let mut manager = HistoryManager::new_in_memory().unwrap();
        
        manager.add_entry("rust programming", ContentType::Text).unwrap();
        manager.add_entry("python code", ContentType::Text).unwrap();
        manager.add_entry("rustacean", ContentType::Text).unwrap();
        
        let results = manager.search("rust").unwrap();
        assert_eq!(results.len(), 2);
    }
}
```

## 🔗 Links Relacionados

- **Core Overview**: [CORE-OVERVIEW.md](./CORE-OVERVIEW.md)
- **Tipos**: [TYPES-DEFINITIONS.md](./TYPES-DEFINITIONS.md)
- **Validação**: [VALIDATION.md](./VALIDATION.md)
- **Arquitetura**: [../01-ARCHITECTURE.md](../01-ARCHITECTURE.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
