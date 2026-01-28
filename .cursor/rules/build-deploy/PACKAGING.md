# Packaging

## 📦 Empacotamento .deb

Scripts disponíveis em `scripts/`:

### build-deb-simple.sh
Build básico para teste local.

```bash
./scripts/build-deb-simple.sh
```

### build-deb-universal.sh
Build universal compatível com múltiplas versões de Ubuntu/Debian.

```bash
./scripts/build-deb-universal.sh
```

### build-deb-ubuntu20.sh
Build específico para Ubuntu 20.04 (glibc 2.31).

## 📋 Estrutura do Pacote

```
clippit_1.9.4_amd64.deb
├── usr/local/bin/
│   ├── clippit-daemon
│   ├── clippit-popup
│   ├── clippit-dashboard
│   ├── clippit-ibus
│   └── clippit-tooltip
├── usr/share/applications/
│   └── clippit.desktop
├── usr/share/ibus/component/
│   └── clippit.xml
└── /etc/systemd/user/
    └── clippit.service
```

## 🔧 Build Manual

```bash
# 1. Build release
cargo build --release

# 2. Criar estrutura
mkdir -p /tmp/clippit-deb/usr/local/bin

# 3. Copiar binários
cp target/release/clippit-* /tmp/clippit-deb/usr/local/bin/

# 4. Criar DEBIAN/control
# 5. Build .deb
dpkg-deb --build /tmp/clippit-deb clippit_1.9.4_amd64.deb
```

## 🔗 Links
- [Build System](./BUILD-SYSTEM.md)
- [Installation](./INSTALLATION.md)
- [BUILD_FOR_USERS.md](../../BUILD_FOR_USERS.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
