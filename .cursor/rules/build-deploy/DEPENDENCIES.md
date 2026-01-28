# Dependencies

## 📦 Dependências do Workspace

Definidas em `Cargo.toml` raiz:

```toml
[workspace.dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
rusqlite = { version = "0.31", features = ["bundled"] }
arboard = { version = "3.6", features = ["wayland-data-control"] }
global-hotkey = "0.7"
rdev = "0.5"
gtk4 = "0.8"
libadwaita = "0.6"
# ...
```

## 🔧 Runtime Dependencies

### Debian/Ubuntu
```bash
sudo apt install \
    libgtk-4-1 \
    libadwaita-1-0 \
    libsqlite3-0 \
    xdotool \
    yad \
    ibus
```

### Build Dependencies
```bash
sudo apt install \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    qt6-base-dev \
    qt6-declarative-dev
```

## 📚 Principais Bibliotecas

| Crate | Dependência | Propósito |
|-------|-------------|-----------|
| core | rusqlite | Banco de dados |
| core | serde, toml | Configuração |
| daemon | tokio | Async runtime |
| daemon | arboard | Clipboard |
| daemon | global-hotkey | Hotkeys |
| daemon | rdev | Keyboard events |
| popup | gtk4, libadwaita | UI |
| dashboard | cxx-qt | Qt bindings |
| ibus | zbus | DBus/IBus |

## 🔗 Links
- [Build System](./BUILD-SYSTEM.md)
- [Installation](./INSTALLATION.md)
- [Cargo.toml](../../Cargo.toml)

---
**Versão**: 1.0 | **Data**: 2026-01-28
