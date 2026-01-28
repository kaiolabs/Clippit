# clippit-dashboard - Dashboard Qt/QML

## 📍 Localização
`crates/clippit-dashboard/`

## 🏗️ Estrutura

```
qml/
├── Main.qml          # Window principal
├── components/
│   └── MenuButton.qml  # Sidebar button
└── pages/
    ├── GeneralPage.qml    # Config geral
    ├── HotkeysPage.qml    # Hotkeys
    ├── ThemePage.qml      # Temas
    └── PrivacyPage.qml    # Privacidade

src/ui/
├── general.rs        # General controller
├── hotkeys.rs        # Hotkeys controller
├── theme.rs          # Theme controller
├── privacy.rs        # Privacy controller
└── autocomplete.rs   # Autocomplete controller
```

## 📋 Páginas

### General
- Max history items (spinner)
- Poll interval (spinner)
- Text/image size limits

### Hotkeys
- Editor visual de hotkeys
- Teste em tempo real
- Detecção de conflitos

### Theme
- Selector: Dark/Light/Nord/Dracula/Gruvbox
- Preview em tempo real
- Custom colors

### Privacy
- Lista de apps ignorados
- Add/remove apps
- Clear on exit toggle

### Autocomplete
- Enable/disable
- Min chars, delay, max suggestions
- Apps ignorados
- Hotkey toggle

## 🔄 Fluxo

1. Load config via `clippit-qt-bridge`
2. Renderiza QML com Models
3. User edita
4. Save config via `Config::save()`
5. Daemon recarrega automaticamente

## 🔗 Links
- [UI Overview](./UI-OVERVIEW.md)
- [Qt Bridge](../infrastructure/QT-BRIDGE.md)
- [Config Patterns](../core/CONFIG-PATTERNS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
