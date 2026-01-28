# Installation

## 📦 Instalação via .deb

```bash
# Instalar pacote
sudo dpkg -i clippit_1.9.4_amd64.deb

# Resolver dependências
sudo apt install -f

# Habilitar serviço
systemctl --user enable --now clippit
```

## 🔧 Instalação Manual

### Pré-requisitos
```bash
sudo apt install \
    libgtk-4-1 \
    libadwaita-1-0 \
    libsqlite3-0 \
    xdotool \
    yad \
    ibus
```

### Build e Install
```bash
# Build
cargo build --release

# Copiar binários
sudo cp target/release/clippit-* /usr/local/bin/

# Copiar assets
sudo mkdir -p /usr/local/share/clippit
sudo cp assets/logo_clippit.png /usr/local/share/clippit/

# Instalar systemd service
mkdir -p ~/.config/systemd/user/
cp assets/clippit.service ~/.config/systemd/user/

# Instalar desktop entry
sudo cp assets/clippit.desktop /usr/share/applications/

# Instalar IBus component
sudo bash scripts/install-ibus.sh
```

### Iniciar
```bash
systemctl --user daemon-reload
systemctl --user enable --now clippit
```

## 🗑️ Desinstalação

```bash
# Via apt
sudo apt remove clippit

# Manual
./scripts/uninstall.sh
```

## 🔗 Links
- [Packaging](./PACKAGING.md)
- [INSTALL_PACKAGE.md](../../docs/INSTALL_PACKAGE.md)
- [QUICKSTART.md](../../docs/QUICKSTART.md)

---
**Versão**: 1.0 | **Data**: 2026-01-28
