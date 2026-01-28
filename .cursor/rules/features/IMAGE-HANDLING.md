# Image Handling Feature

## 🎯 Responsabilidade
Captura, otimização, armazenamento e preview de imagens.

## 🔄 Pipeline
1. **Captura**: arboard.get_image() → ImageData (RGBA)
2. **Converte**: ImageData → PNG bytes
3. **Valida**: tamanho, formato
4. **Hash**: SHA256 para deduplicação
5. **Otimiza**: resize se > 2048px (Lanczos3)
6. **Thumbnail**: 128x128 para preview
7. **Salva**: `~/.local/share/clippit/images/{hash}.png`
8. **Persiste**: path + thumbnail no SQLite

## 📦 Funções

```rust
fn convert_image_data_to_png(data: &ImageData) -> Result<Vec<u8>>;
fn optimize_image(bytes: &[u8]) -> Result<Vec<u8>>;
fn create_thumbnail(bytes: &[u8]) -> Result<Vec<u8>>;
fn save_image_to_file(bytes: &[u8], hash: &str) -> Result<PathBuf>;
```

## 📐 Limites
- **Max size**: 50MB
- **Formatos**: PNG, JPEG
- **Max dimensions**: 2048x2048 (auto-resize)
- **Thumbnail**: 128x128

## 🔗 Links
- [Monitor](../daemon/MONITOR-CLIPBOARD.md)
- [Validation](../core/VALIDATION.md)
- [Popup Preview](../ui/POPUP-GTK.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
