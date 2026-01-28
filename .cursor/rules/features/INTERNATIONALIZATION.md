# Internacionalização (i18n)

## 🎯 Responsabilidade
Sistema de traduções multi-idioma usando rust-i18n.

## 📂 Estrutura
```
crates/clippit-core/locales/
├── en.yml    # Inglês (default)
└── pt.yml    # Português
```

## 📝 Uso

```rust
use rust_i18n::t;

// Tradução simples
let title = t!("popup.title");

// Com interpolação
let msg = t!("messages.deleted", count = 5);

// Definir idioma
clippit_core::set_language("pt");
```

## 📋 Estrutura YAML

```yaml
# en.yml
popup:
  title: "Clipboard History"
  search_placeholder: "Search..."
  
menu:
  copy: "Copy"
  delete: "Delete"
  
messages:
  deleted: "{count} items deleted"
  error: "Error: {message}"
```

## ⚙️ Configuração

```toml
[ui]
language = "en"  # ou "pt"
```

## 🌍 Idiomas Suportados
- ✅ Inglês (en)
- ✅ Português (pt)
- 🔮 Mais idiomas: contribuições bem-vindas

## 🔗 Links
- [Config Patterns](../core/CONFIG-PATTERNS.md)
- [UI Patterns](../ui/UI-PATTERNS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
