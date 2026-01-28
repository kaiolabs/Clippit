# Search & Suggestions Feature

## 🎯 Responsabilidade
Busca em tempo real no histórico com autocomplete inteligente.

## 🔄 Busca no Histórico
```rust
// SQLite LIKE query
SELECT * FROM clipboard_history
WHERE content_text LIKE '%query%'
   OR image_path LIKE '%query%'
ORDER BY timestamp DESC
```

## 🎨 Autocomplete de Busca
1. **SuggestionEngine** extrai palavras do histórico
2. **Score** por frequência
3. **Filtra** por prefixo
4. **Exibe** em popover GTK4
5. **Tab** completa palavra

## 📦 Componentes
- `popup/views/search.rs` - SearchEntry
- `popup/utils/suggestions.rs` - SuggestionEngine
- `popup/views/suggestions_popover.rs` - Popover

## ⚙️ Configuração
```toml
[search]
max_suggestions = 5
focus_on_show = true
```

## 🔗 Links
- [Popup GTK](../ui/POPUP-GTK.md)
- [History Storage](../core/HISTORY-STORAGE.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
