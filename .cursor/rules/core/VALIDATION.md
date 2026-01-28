# clippit-core - Validação de Conteúdo

## 📍 Localização
`crates/clippit-core/src/validator.rs`

## 🎯 Responsabilidade

**ContentValidator** é responsável por validar todo conteúdo antes de ser persistido no histórico.

### Objetivos
- ✅ **Prevenir dados inválidos**: Rejeitar conteúdo corrompido
- ✅ **Limites de tamanho**: Evitar consumo excessivo de memória/disco
- ✅ **Formato válido**: Garantir UTF-8 válido, imagens válidas
- ✅ **Early rejection**: Falhar rápido antes de processar

## 📊 Estrutura

```rust
pub struct ContentValidator;

impl ContentValidator {
    /// Valida conteúdo textual
    pub fn validate_text(text: &str) -> Result<(), ValidationError>;
    
    /// Valida dados de imagem
    pub fn validate_image(data: &[u8]) -> Result<(), ValidationError>;
    
    /// Valida tamanho
    fn check_size(size: usize, max: usize, type_name: &str) 
        -> Result<(), ValidationError>;
}
```

## 🔍 Regras de Validação

### 1. Texto

```rust
pub fn validate_text(text: &str) -> Result<(), ValidationError> {
    // 1. Não pode ser vazio
    if text.is_empty() {
        return Err(ValidationError::TextEmpty);
    }
    
    // 2. Tamanho máximo: 10MB
    const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;
    if text.len() > MAX_TEXT_SIZE {
        return Err(ValidationError::TextTooLarge {
            size: text.len(),
            max: MAX_TEXT_SIZE,
        });
    }
    
    // 3. UTF-8 válido (sem replacement chars)
    if text.contains('�') {
        return Err(ValidationError::InvalidUtf8);
    }
    
    Ok(())
}
```

**Regras**:
- Mínimo: 1 caractere
- Máximo: 10MB (10,485,760 bytes)
- Encoding: UTF-8 válido
- Sem replacement characters (`�`)

### 2. Imagem

```rust
pub fn validate_image(data: &[u8]) -> Result<(), ValidationError> {
    // 1. Não pode ser vazio
    if data.is_empty() {
        return Err(ValidationError::ImageEmpty);
    }
    
    // 2. Tamanho máximo: 50MB
    const MAX_IMAGE_SIZE: usize = 50 * 1024 * 1024;
    if data.len() > MAX_IMAGE_SIZE {
        return Err(ValidationError::ImageTooLarge {
            size: data.len(),
            max: MAX_IMAGE_SIZE,
        });
    }
    
    // 3. Formato válido (PNG ou JPEG)
    use image::ImageFormat;
    
    let format = image::guess_format(data)
        .map_err(|_| ValidationError::UnsupportedImageFormat)?;
    
    match format {
        ImageFormat::Png | ImageFormat::Jpeg => Ok(()),
        _ => Err(ValidationError::UnsupportedImageFormat),
    }
}
```

**Regras**:
- Mínimo: 1 byte
- Máximo: 50MB (52,428,800 bytes)
- Formatos suportados: PNG, JPEG
- Header válido (magic bytes)

## 📋 Limites Configurados

```rust
// Constantes de validação
pub const MAX_TEXT_SIZE_BYTES: usize = 10 * 1024 * 1024;    // 10MB
pub const MAX_IMAGE_SIZE_BYTES: usize = 50 * 1024 * 1024;   // 50MB

// Formato de imagem suportados
pub const SUPPORTED_IMAGE_FORMATS: &[ImageFormat] = &[
    ImageFormat::Png,
    ImageFormat::Jpeg,
];
```

## 🔄 Integração com HistoryManager

```rust
impl HistoryManager {
    pub fn add_entry(
        &mut self,
        content: &str,
        content_type: ContentType
    ) -> Result<i64> {
        // SEMPRE validar antes de processar
        match content_type {
            ContentType::Text => {
                ContentValidator::validate_text(content)?;
            }
            ContentType::Image => {
                // Validação será feita com os bytes
            }
        }
        
        // Processar apenas se válido
        // ...
    }
    
    pub fn add_image_entry(
        &mut self,
        image_data: &[u8]
    ) -> Result<i64> {
        // Validar ANTES de processar/salvar
        ContentValidator::validate_image(image_data)?;
        
        // Agora sim, processar
        let optimized = optimize_image(image_data)?;
        let thumbnail = create_thumbnail(image_data)?;
        // ...
    }
}
```

## ✅ Casos de Uso

### 1. Validação Simples

```rust
use clippit_core::ContentValidator;

// Texto
match ContentValidator::validate_text(&text) {
    Ok(()) => println!("Texto válido"),
    Err(e) => eprintln!("Erro: {}", e),
}

// Imagem
match ContentValidator::validate_image(&image_bytes) {
    Ok(()) => println!("Imagem válida"),
    Err(e) => eprintln!("Erro: {}", e),
}
```

### 2. Early Return Pattern

```rust
fn process_clipboard_content(content: &str) -> Result<()> {
    // Valida logo no início
    ContentValidator::validate_text(content)?;
    
    // Continue processamento apenas se válido
    let hash = compute_hash(content);
    save_to_database(content, hash)?;
    
    Ok(())
}
```

### 3. Validação com Contexto

```rust
fn validate_with_context(
    content: &str,
    source_app: &str
) -> Result<()> {
    // Validação básica
    ContentValidator::validate_text(content)?;
    
    // Validação específica de contexto
    if source_app == "password-manager" {
        return Err(anyhow!("Passwords not allowed"));
    }
    
    Ok(())
}
```

## 🧪 Testes

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_text() {
        let text = "Hello, world!";
        assert!(ContentValidator::validate_text(text).is_ok());
    }
    
    #[test]
    fn test_empty_text() {
        let text = "";
        assert!(matches!(
            ContentValidator::validate_text(text),
            Err(ValidationError::TextEmpty)
        ));
    }
    
    #[test]
    fn test_text_too_large() {
        let huge_text = "x".repeat(11 * 1024 * 1024); // 11MB
        let result = ContentValidator::validate_text(&huge_text);
        
        assert!(matches!(
            result,
            Err(ValidationError::TextTooLarge { .. })
        ));
    }
    
    #[test]
    fn test_invalid_utf8() {
        let text = "Hello � World";  // Replacement char
        assert!(matches!(
            ContentValidator::validate_text(text),
            Err(ValidationError::InvalidUtf8)
        ));
    }
    
    #[test]
    fn test_valid_png_image() {
        // Minimal valid PNG header
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            // ... resto do PNG
        ];
        
        assert!(ContentValidator::validate_image(&png_data).is_ok());
    }
    
    #[test]
    fn test_empty_image() {
        let empty: Vec<u8> = vec![];
        assert!(matches!(
            ContentValidator::validate_image(&empty),
            Err(ValidationError::ImageEmpty)
        ));
    }
    
    #[test]
    fn test_unsupported_image_format() {
        // GIF header (não suportado)
        let gif_data = vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
        
        assert!(matches!(
            ContentValidator::validate_image(&gif_data),
            Err(ValidationError::UnsupportedImageFormat)
        ));
    }
    
    #[test]
    fn test_image_too_large() {
        let huge_image = vec![0xFF; 51 * 1024 * 1024]; // 51MB
        
        assert!(matches!(
            ContentValidator::validate_image(&huge_image),
            Err(ValidationError::ImageTooLarge { .. })
        ));
    }
}
```

## 🚫 Anti-Patterns

### ❌ Validar Depois de Processar

```rust
// NÃO!
let hash = compute_hash(text);  // Processou antes de validar
save_to_db(text, hash)?;
ContentValidator::validate_text(text)?;  // Tarde demais!

// ✅ Valide PRIMEIRO
ContentValidator::validate_text(text)?;
let hash = compute_hash(text);
save_to_db(text, hash)?;
```

### ❌ Ignorar Erros de Validação

```rust
// NÃO!
ContentValidator::validate_text(text).ok();  // Ignora erro
save_to_db(text)?;

// ✅ Propague erro
ContentValidator::validate_text(text)?;
save_to_db(text)?;
```

### ❌ Validação Inconsistente

```rust
// NÃO! Valida em alguns lugares, não em outros
fn add_entry_a(text: &str) {
    ContentValidator::validate_text(text)?;  // Valida
    // ...
}

fn add_entry_b(text: &str) {
    // Não valida - INCONSISTENTE!
    save_to_db(text)?;
}

// ✅ Valide SEMPRE no mesmo ponto (ex: HistoryManager)
```

### ❌ Magic Numbers

```rust
// NÃO!
if text.len() > 10485760 { }  // O que é esse número?

// ✅ Use constantes nomeadas
const MAX_TEXT_SIZE: usize = 10 * 1024 * 1024;
if text.len() > MAX_TEXT_SIZE { }
```

## 📝 Checklist de Validação

Antes de adicionar conteúdo ao histórico:

- [ ] Validação de tamanho (min/max)
- [ ] Validação de formato
- [ ] Validação de encoding (UTF-8)
- [ ] Early return em caso de erro
- [ ] Mensagens de erro descritivas
- [ ] Teste unitário para caso válido
- [ ] Teste unitário para casos inválidos
- [ ] Documentação atualizada

## 🔧 Customização

### Limites Configuráveis (Futuro)

```rust
// Atualmente hardcoded, mas pode ser configurável
impl ContentValidator {
    pub fn with_limits(
        max_text_mb: usize,
        max_image_mb: usize
    ) -> Self {
        Self {
            max_text_size: max_text_mb * 1024 * 1024,
            max_image_size: max_image_mb * 1024 * 1024,
        }
    }
}
```

### Validação Personalizada

```rust
pub trait CustomValidator {
    fn validate(&self, entry: &ClipboardEntry) -> Result<()>;
}

// Implementação exemplo
struct PasswordValidator;

impl CustomValidator for PasswordValidator {
    fn validate(&self, entry: &ClipboardEntry) -> Result<()> {
        if let Some(text) = &entry.content_text {
            if text.contains("password") {
                bail!("Passwords not allowed");
            }
        }
        Ok(())
    }
}
```

## 🔗 Links Relacionados

- **Core Overview**: [CORE-OVERVIEW.md](./CORE-OVERVIEW.md)
- **Tipos**: [TYPES-DEFINITIONS.md](./TYPES-DEFINITIONS.md)
- **Storage**: [HISTORY-STORAGE.md](./HISTORY-STORAGE.md)
- **Configuração**: [CONFIG-PATTERNS.md](./CONFIG-PATTERNS.md)

---

**Versão**: 1.0  
**Data**: 2026-01-28
