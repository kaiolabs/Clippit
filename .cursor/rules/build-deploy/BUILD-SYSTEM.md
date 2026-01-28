# Build System

## 🎯 Sistema de Build

Cargo workspace com múltiplos crates.

## 🔧 Comandos

### Development
```bash
# Build debug
cargo build

# Build com logs
RUST_LOG=debug cargo build

# Build crate específico
cargo build -p clippit-daemon
```

### Release
```bash
# Build release (otimizado)
cargo build --release

# Build específico
cargo build --release -p clippit-popup
```

### Verificação
```bash
# Formatar
cargo fmt --all

# Linting
cargo clippy --all-targets -- -D warnings

# Testes
cargo test --all

# Build completo
cargo build --release --all
```

## 📦 Workspace

```toml
[workspace]
members = [
    "crates/clippit-core",
    "crates/clippit-daemon",
    "crates/clippit-ipc",
    "crates/clippit-popup",
    "crates/clippit-dashboard",
    "crates/clippit-ibus",
    "crates/clippit-qt-bridge",
    "crates/clippit-tooltip",
    "crates/clippit-ui",
]
resolver = "2"
```

## 🎯 Targets

- **daemon**: `cargo build --release --bin clippit-daemon`
- **popup**: `cargo build --release --bin clippit-popup`
- **dashboard**: `cargo build --release --bin clippit-dashboard`
- **ibus**: `cargo build --release --bin clippit-ibus`
- **tooltip**: `cargo build --release --bin clippit-tooltip`

## 🔗 Links
- [Dependencies](./DEPENDENCIES.md)
- [Packaging](./PACKAGING.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
