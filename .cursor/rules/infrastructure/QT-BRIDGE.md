# Qt Bridge

## 📍 Localização
`crates/clippit-qt-bridge/src/`

## 🎯 Responsabilidade
Bridge Rust-QML: expõe models Rust para QML via cxx-qt.

## 📦 Models

### ConfigModel
```rust
pub struct ConfigModel {
    config: Config,
}

impl ConfigModel {
    pub fn new() -> Self;
    pub fn load_config();
    pub fn save_config() -> bool;
    pub fn get_max_history_items() -> i32;
    pub fn set_max_history_items(value: i32);
    // ...
}
```

### HistoryModel
```rust
pub struct HistoryModel {
    ipc_client: IpcClient,
}

impl HistoryModel {
    pub fn load_history(limit: i32);
    pub fn get_item_count() -> i32;
    pub fn get_item_content(index: i32) -> String;
}
```

### ThemeModel
```rust
pub struct ThemeModel {
    current_theme: String,
}

impl ThemeModel {
    pub fn load_theme(name: &str);
    pub fn save_custom_theme() -> bool;
    pub fn get_available_themes() -> Vec<String>;
}
```

## 🔗 Links
- [Dashboard Qt](../ui/DASHBOARD-QT.md)
- [Config Patterns](../core/CONFIG-PATTERNS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
