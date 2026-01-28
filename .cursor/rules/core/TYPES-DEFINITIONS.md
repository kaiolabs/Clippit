# clippit-core - Definições de Tipos

## 📍 Localização
`crates/clippit-core/src/types.rs`

## 🎯 Responsabilidade

Define todos os **tipos de dados fundamentais** usados em todo o projeto Clippit.

### Princípios
- ✅ **Type Safety**: Usar enums ao invés de strings mágicas
- ✅ **Serializable**: Todos os tipos são serializáveis (SQLite, IPC)
- ✅ **Self-Documenting**: Nomes claros e descritivos
- ✅ **Immutability**: Preferir imutabilidade quando possível

## 📊 Tipos Principais

### 1. ContentType

```rust
/// Tipo de conteúdo do clipboard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Conteúdo textual
    Text,
    
    /// Imagem (PNG, JPEG)
    Image,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Image => "Image",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Text" => Some(Self::Text),
            "Image" => Some(Self::Image),
            _ => None,
        }
    }
}
```

**Por quê enum?**
- Type-safe: impossível ter typo "Textt"
- Pattern matching exhaustivo
- Fácil adicionar novos tipos (File, Html, etc)

### 2. ClipboardEntry

```rust
/// Entrada no histórico do clipboard
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClipboardEntry {
    /// ID único (auto-increment do SQLite)
    pub id: i64,
    
    /// Tipo de conteúdo
    pub content_type: ContentType,
    
    /// Conteúdo textual (se Text)
    pub content_text: Option<String>,
    
    /// Conteúdo binário (DEPRECATED - apenas para backwards compat)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_data: Option<Vec<u8>>,
    
    /// Path para imagem no filesystem (se Image)
    pub image_path: Option<String>,
    
    /// Thumbnail PNG 128x128 (para preview rápido)
    pub thumbnail_data: Option<Vec<u8>>,
    
    /// Timestamp UTC (RFC3339)
    pub timestamp: DateTime<Utc>,
}

impl ClipboardEntry {
    /// Cria nova entrada de texto
    pub fn new_text(text: String) -> Self {
        Self {
            id: 0,  // Será atribuído pelo banco
            content_type: ContentType::Text,
            content_text: Some(text),
            content_data: None,
            image_path: None,
            thumbnail_data: None,
            timestamp: Utc::now(),
        }
    }
    
    /// Cria nova entrada de imagem
    pub fn new_image(
        image_path: String,
        thumbnail: Vec<u8>
    ) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Image,
            content_text: None,
            content_data: None,
            image_path: Some(image_path),
            thumbnail_data: Some(thumbnail),
            timestamp: Utc::now(),
        }
    }
    
    /// Obtém preview do conteúdo (primeiros 100 chars)
    pub fn preview(&self) -> String {
        match self.content_type {
            ContentType::Text => {
                self.content_text
                    .as_ref()
                    .map(|t| {
                        let preview = t.chars().take(100).collect::<String>();
                        if t.len() > 100 {
                            format!("{}...", preview)
                        } else {
                            preview
                        }
                    })
                    .unwrap_or_else(|| "[Empty]".to_string())
            }
            ContentType::Image => {
                self.image_path
                    .as_ref()
                    .map(|p| format!("[Image: {}]", p))
                    .unwrap_or_else(|| "[Image]".to_string())
            }
        }
    }
    
    /// Tamanho aproximado em bytes
    pub fn size_bytes(&self) -> usize {
        match self.content_type {
            ContentType::Text => {
                self.content_text.as_ref().map(|s| s.len()).unwrap_or(0)
            }
            ContentType::Image => {
                self.thumbnail_data.as_ref().map(|d| d.len()).unwrap_or(0)
            }
        }
    }
}
```

### 3. ValidationError

```rust
/// Erros de validação de conteúdo
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Text is too large: {size} bytes (max: {max} bytes)")]
    TextTooLarge { size: usize, max: usize },
    
    #[error("Text is empty")]
    TextEmpty,
    
    #[error("Text contains invalid UTF-8")]
    InvalidUtf8,
    
    #[error("Image is too large: {size} bytes (max: {max} bytes)")]
    ImageTooLarge { size: usize, max: usize },
    
    #[error("Image is empty")]
    ImageEmpty,
    
    #[error("Image format not supported")]
    UnsupportedImageFormat,
    
    #[error("Image processing failed: {0}")]
    ImageProcessingFailed(String),
}
```

### 4. ConfigError (Re-export)

```rust
/// Erros de configuração
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found: {0}")]
    NotFound(String),
    
    #[error("Failed to parse config: {0}")]
    ParseError(String),
    
    #[error("Invalid config value: {0}")]
    InvalidValue(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## 📦 Tipos Auxiliares

### SearchResult

```rust
/// Resultado de busca com score
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: ClipboardEntry,
    pub score: f64,  // 0.0 - 1.0
    pub matched_text: Option<String>,
}

impl SearchResult {
    pub fn new(entry: ClipboardEntry, score: f64) -> Self {
        Self {
            entry,
            score,
            matched_text: None,
        }
    }
}
```

### Pagination

```rust
/// Parâmetros de paginação
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub limit: usize,
    pub offset: usize,
}

impl Pagination {
    pub fn new(limit: usize, offset: usize) -> Self {
        Self { limit, offset }
    }
    
    pub fn first_page(limit: usize) -> Self {
        Self { limit, offset: 0 }
    }
    
    pub fn next_page(&self) -> Self {
        Self {
            limit: self.limit,
            offset: self.offset + self.limit,
        }
    }
}
```

## 🔄 Conversões

### From/Into Implementations

```rust
// SQLite Row → ClipboardEntry
impl TryFrom<&Row<'_>> for ClipboardEntry {
    type Error = rusqlite::Error;
    
    fn try_from(row: &Row) -> Result<Self, Self::Error> {
        let content_type_str: String = row.get(1)?;
        let content_type = ContentType::from_str(&content_type_str)
            .ok_or_else(|| rusqlite::Error::InvalidColumnType(
                1,
                "Invalid content_type".to_string(),
                rusqlite::types::Type::Text
            ))?;
        
        Ok(Self {
            id: row.get(0)?,
            content_type,
            content_text: row.get(2)?,
            content_data: row.get(3)?,
            image_path: row.get(4)?,
            thumbnail_data: row.get(5)?,
            timestamp: row.get(6)?,
        })
    }
}

// String → ContentType
impl FromStr for ContentType {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s).ok_or_else(|| 
            format!("Invalid content type: {}", s)
        )
    }
}
```

## ✅ Padrões de Uso

### Pattern Matching

```rust
fn process_entry(entry: &ClipboardEntry) {
    match entry.content_type {
        ContentType::Text => {
            if let Some(text) = &entry.content_text {
                println!("Text: {}", text);
            }
        }
        ContentType::Image => {
            if let Some(path) = &entry.image_path {
                println!("Image: {}", path);
            }
        }
    }
}
```

### Option Handling

```rust
// ✅ Correto - use if let ou match
if let Some(text) = &entry.content_text {
    process_text(text);
}

// ✅ Ou com unwrap_or
let text = entry.content_text
    .as_ref()
    .unwrap_or(&"".to_string());

// ❌ Evite unwrap sem justificativa
let text = entry.content_text.unwrap();  // E se for None?
```

### Constructor Pattern

```rust
// ✅ Use constructors named
let entry = ClipboardEntry::new_text("Hello".to_string());

// ❌ Evite construção manual
let entry = ClipboardEntry {
    id: 0,
    content_type: ContentType::Text,
    content_text: Some("Hello".to_string()),
    // ... tedioso e propenso a erro
};
```

## 🧪 Testes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_type_string_conversion() {
        assert_eq!(ContentType::Text.as_str(), "Text");
        assert_eq!(ContentType::Image.as_str(), "Image");
        
        assert_eq!(ContentType::from_str("Text"), Some(ContentType::Text));
        assert_eq!(ContentType::from_str("Invalid"), None);
    }
    
    #[test]
    fn test_clipboard_entry_preview() {
        let short = ClipboardEntry::new_text("Short".to_string());
        assert_eq!(short.preview(), "Short");
        
        let long_text = "a".repeat(150);
        let long = ClipboardEntry::new_text(long_text);
        assert!(long.preview().ends_with("..."));
        assert!(long.preview().len() < 105);
    }
    
    #[test]
    fn test_entry_size() {
        let entry = ClipboardEntry::new_text("Hello".to_string());
        assert_eq!(entry.size_bytes(), 5);
    }
    
    #[test]
    fn test_serialization() {
        let entry = ClipboardEntry::new_text("Test".to_string());
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: ClipboardEntry = serde_json::from_str(&json).unwrap();
        
        assert_eq!(entry.content_text, deserialized.content_text);
    }
}
```

## 🚫 Anti-Patterns

### ❌ Strings Mágicas

```rust
// NÃO!
if entry_type == "Text" { }

// ✅ Use enum
if entry.content_type == ContentType::Text { }
```

### ❌ Structs Gigantes

```rust
// Evite structs com 20+ campos
// Divida em structs menores e especializadas
```

### ❌ Mutabilidade Excessiva

```rust
// ❌ Ruim
pub struct Entry {
    pub mut text: String,  // Mutável sem necessidade
}

// ✅ Bom
pub struct Entry {
    pub text: String,  // Imutável por padrão
}
```

### ❌ Falta de Derives

```rust
// ❌ Sem derives necessários
pub struct Entry { }

// ✅ Com derives apropriados
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entry { }
```

## 📝 Checklist para Novos Tipos

- [ ] Nome claro e descritivo
- [ ] Documentação com `///`
- [ ] Derives apropriados (`Debug`, `Clone`, etc)
- [ ] Serialização se necessário (`Serialize`, `Deserialize`)
- [ ] Constructors named quando apropriado
- [ ] Conversões `From`/`Into` se útil
- [ ] Testes unitários
- [ ] Validação de invariantes

## 🔗 Links Relacionados

- **Core Overview**: [CORE-OVERVIEW.md](./CORE-OVERVIEW.md)
- **Validação**: [VALIDATION.md](./VALIDATION.md)
- **Storage**: [HISTORY-STORAGE.md](./HISTORY-STORAGE.md)
- **IPC Types**: [../infrastructure/IPC-PROTOCOL.md](../infrastructure/IPC-PROTOCOL.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
