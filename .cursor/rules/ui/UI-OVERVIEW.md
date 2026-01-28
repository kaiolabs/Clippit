# User Interfaces - Overview

## 📍 Localização
- `crates/clippit-popup/` - Popup GTK4
- `crates/clippit-dashboard/` - Dashboard Qt/QML
- `crates/clippit-tooltip/` - Tooltip GTK4

## 🎯 Responsabilidades

### clippit-popup (GTK4 + libadwaita)
Interface principal para visualizar e buscar histórico.
- Listagem de itens
- Busca em tempo real
- Navegação por teclado
- Preview de imagens
- Autocomplete de busca

### clippit-dashboard (Qt6 + QML)
Dashboard de configurações.
- Tabs: General, Hotkeys, Theme, Privacy, Autocomplete
- Editor visual de configurações
- Preview de temas
- Estatísticas de uso

### clippit-tooltip (GTK4)
Tooltip flutuante para autocomplete.
- Window sem decoração
- Auto-close após 3s
- Estilo minimalista

## 🏗️ Arquitetura

```
UIs (Clientes IPC)
├── clippit-popup
│   ├── views/      (GTK4 components)
│   ├── controllers/  (lógica)
│   ├── models/     (estado)
│   └── utils/      (helpers)
│
├── clippit-dashboard
│   ├── qml/        (QML UI)
│   └── src/ui/     (Rust controllers)
│
└── clippit-tooltip
    └── src/        (GTK4 simples)
```

## 📦 Dependências Comuns

### Popup
- gtk4, libadwaita
- clippit-ipc (comunicação)
- clippit-core (tipos)
- fuzzy-matcher (busca)

### Dashboard
- cxx-qt (Qt bindings)
- clippit-qt-bridge (models)
- clippit-core, clippit-ipc

### Tooltip
- gtk4 (minimal)

## 🔗 Links
- [Popup GTK](./POPUP-GTK.md)
- [Dashboard Qt](./DASHBOARD-QT.md)
- [Tooltip](./TOOLTIP.md)
- [UI Patterns](./UI-PATTERNS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
