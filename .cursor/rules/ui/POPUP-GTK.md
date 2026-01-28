# clippit-popup - Popup GTK4

## 📍 Localização
`crates/clippit-popup/src/`

## 🏗️ Estrutura MVC

```
src/
├── main.rs           # Entry point
├── views/            # GTK4 components
│   ├── window.rs       # Main window
│   ├── search.rs       # Search entry + popover
│   ├── list_item.rs    # List items
│   ├── buttons.rs      # Action buttons
│   └── image_preview.rs # Image preview
├── controllers/
│   ├── keyboard.rs     # Keyboard navigation
│   └── clipboard.rs    # Copy to clipboard
├── models/
│   └── entry_map.rs    # State management
└── utils/
    ├── suggestions.rs  # Autocomplete engine
    ├── theme.rs        # Theme application
    └── thumbnail.rs    # Thumbnail generation
```

## 🎨 UI Components

### Window
- Tamanho: 700x550px
- Sem decoração (libadwaita)
- Auto-close inteligente (delay 300ms)

### SearchEntry
- Busca em tempo real
- Autocomplete popover
- Navegação: ↑↓ Tab Enter Esc

### ListBox
- Infinite scroll (30 inicial + 20 on-demand)
- Skeleton loaders
- Thumbnails 128px
- Preview hover para imagens

## 🔄 Fluxo

1. Usuário pressiona Super+V
2. Daemon spawna popup
3. Popup cria lock file
4. IPC: QueryHistoryMetadata(30)
5. Renderiza ListBox
6. Scroll → carrega mais via IPC

## 📝 Keyboard Navigation

- `↑↓` - Navegar lista
- `Enter` - Copiar item
- `Delete` - Deletar item
- `Esc` - Fechar popup
- `Ctrl+F` - Focar busca
- `Tab` - Autocomplete

## 🔗 Links
- [UI Overview](./UI-OVERVIEW.md)
- [UI Patterns](./UI-PATTERNS.md)
- [IPC Protocol](../infrastructure/IPC-PROTOCOL.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
