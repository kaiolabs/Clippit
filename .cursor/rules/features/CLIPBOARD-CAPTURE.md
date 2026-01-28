# Clipboard Capture Feature

## 🎯 Responsabilidade
Captura automática de conteúdo copiado (texto e imagens).

## 🔄 Fluxo
1. **Monitor polling** (80ms) via arboard
2. **Detecta mudança** (compara com último conteúdo)
3. **Valida** (ContentValidator)
4. **Deduplica** (SHA256 hash)
5. **Processa** (imagens: otimiza, thumbnail)
6. **Persiste** (HistoryManager → SQLite)

## 📦 Componentes
- `daemon/monitor.rs` - Polling loop
- `core/validator.rs` - Validação
- `core/history.rs` - Persistência

## ⚙️ Configuração
```toml
[features]
capture_text = true
capture_images = true

[general]
max_text_size_mb = 10
max_image_size_mb = 50
```

## 🔗 Links
- [Monitor](../daemon/MONITOR-CLIPBOARD.md)
- [Validation](../core/VALIDATION.md)
- [Image Handling](./IMAGE-HANDLING.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
