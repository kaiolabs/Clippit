# UI Patterns - Padrões de Interface

## 🎨 Design Principles

### 1. Performance
- **Lazy loading**: Infinite scroll (30+20)
- **Skeleton loaders**: Feedback durante carregamento
- **Debounce**: Busca com 300ms delay
- **Virtual scrolling**: Renderiza apenas visível

### 2. Accessibility
- **Keyboard navigation**: Todas as ações acessíveis via teclado
- **Focus management**: Auto-focus inteligente
- **Screen reader**: Labels apropriados
- **Color contrast**: WCAG AA compliant

### 3. Responsiveness
- **Async operations**: Nunca bloqueia UI
- **Progress indicators**: Feedback visual
- **Error handling**: Mensagens claras
- **Graceful degradation**: Funciona sem features opcionais

## 📋 Common Patterns

### Skeleton Loaders

```rust
fn create_skeleton_item() -> ActionRow {
    let row = ActionRow::builder()
        .title("Loading...")
        .css_classes(vec!["skeleton-loader".to_string()])
        .build();
    row
}

// CSS
.skeleton-loader {
    animation: pulse 1.5s ease-in-out infinite;
}
```

### Infinite Scroll

```rust
let scrolled_window = ScrolledWindow::new();

scrolled_window.connect_edge_reached(move |_, pos| {
    if pos == PositionType::Bottom {
        load_more_items();
    }
});
```

### Debounced Search

```rust
let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

search_entry.connect_changed(move |entry| {
    let text = entry.text().to_string();
    tx.send(text).ok();
});

rx.attach(None, move |query| {
    // Debounce 300ms
    glib::timeout_add_local_once(Duration::from_millis(300), move || {
        perform_search(&query);
    });
    glib::ControlFlow::Continue
});
```

### Theme Application

```rust
fn apply_theme(theme: &str) {
    let manager = adw::StyleManager::default();
    
    match theme {
        "dark" => manager.set_color_scheme(ColorScheme::ForceDark),
        "light" => manager.set_color_scheme(ColorScheme::ForceLight),
        _ => manager.set_color_scheme(ColorScheme::Default),
    }
}
```

## 🚫 Anti-Patterns

❌ **Blocking UI**: Nunca bloqueie main thread
❌ **No feedback**: Sempre dê feedback visual
❌ **Hardcoded sizes**: Use relative sizes
❌ **Missing keyboard nav**: Toda ação deve ter atalho

## 🔗 Links
- [UI Overview](./UI-OVERVIEW.md)
- [Popup GTK](./POPUP-GTK.md)
- [Dashboard Qt](./DASHBOARD-QT.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
