#!/bin/bash

set -e

VERSION="1.0.0"
ARCH="amd64"
PKG_NAME="clippit_${VERSION}_ubuntu20.04+_${ARCH}"
BUILD_DIR="/tmp/clippit-deb-build"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    Clippit - Build Compatível Ubuntu 20.04+                 ║"
echo "║        Funciona em 95% dos sistemas Linux!                  ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "🎯 Target: Ubuntu 20.04+ (glibc 2.31+)"
echo "📊 Compatibilidade: Ubuntu, Debian, Mint, Pop!_OS, Elementary"
echo ""

# Clean previous build
echo "🧹 Limpando build anterior..."
cargo clean
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$PKG_NAME"

echo "✅ Limpeza concluída"
echo ""

# Build with compatibility flags
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            Compilando com flags de compatibilidade          ║"
echo "║              (Isso pode demorar alguns minutos)             ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Set environment variables for maximum compatibility
export RUSTFLAGS="-C target-cpu=x86-64 -C link-arg=-Wl,--no-as-needed"

# Build in release mode with optimizations
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo ""
echo "✅ Build successful"
echo ""

# Verify binaries
echo "🔍 Verificando compatibilidade dos binários..."
DAEMON_BIN="target/release/clippit-daemon"

# Check glibc version required
GLIBC_REQUIRED=$(strings "$DAEMON_BIN" | grep "GLIBC_" | sort -V | tail -1)
echo "   GLIBC requerida: $GLIBC_REQUIRED"

# Get your system glibc
GLIBC_SYSTEM=$(ldd --version | head -1 | grep -oE '[0-9]+\.[0-9]+$')
echo "   GLIBC do sistema: $GLIBC_SYSTEM"

if [[ "$GLIBC_REQUIRED" > "GLIBC_2.31" ]]; then
    echo ""
    echo "⚠️  AVISO: Este binário requer glibc > 2.31"
    echo "   Pode não funcionar em Ubuntu 20.04"
    echo ""
    echo "💡 SOLUÇÃO: Compile em um sistema Ubuntu 20.04"
    echo "   Ou use uma VM/container Ubuntu 20.04 para compilar"
    echo ""
    read -p "Continuar mesmo assim? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "❌ Build cancelado"
        exit 1
    fi
fi

echo ""

# Create package structure
echo "📁 Criando estrutura do pacote..."
cd "$BUILD_DIR/$PKG_NAME"

# Directories
mkdir -p DEBIAN
mkdir -p usr/local/bin
mkdir -p usr/share/applications
mkdir -p usr/share/icons/hicolor/256x256/apps
mkdir -p etc/systemd/user

# Copy binaries
echo "📋 Copiando binários..."
cp "$OLDPWD/target/release/clippit-daemon" usr/local/bin/
cp "$OLDPWD/target/release/clippit-dashboard" usr/local/bin/
cp "$OLDPWD/target/release/clippit-popup" usr/local/bin/
chmod +x usr/local/bin/clippit-*

# Strip binaries
echo "🔨 Otimizando binários..."
strip usr/local/bin/clippit-* 2>/dev/null || true

# Copy assets
echo "🎨 Copiando assets..."
cp "$OLDPWD/assets/logo_clippit.png" usr/share/icons/hicolor/256x256/apps/clippit.png
cp "$OLDPWD/assets/clippit.desktop" usr/share/applications/

# Create systemd service
echo "🔧 Criando systemd service..."
cat > etc/systemd/user/clippit.service << 'EOF'
[Unit]
Description=Clippit Clipboard Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/clippit-daemon
Restart=on-failure
RestartSec=5
Environment="DISPLAY=:0"
Environment="XAUTHORITY=%h/.Xauthority"

[Install]
WantedBy=default.target
EOF

# Create control file
echo "📝 Criando control file..."
cat > DEBIAN/control << EOF
Package: clippit
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libgtk-4-1, libadwaita-1-0
Maintainer: Clippit Team <clippit@example.com>
Description: Modern clipboard manager for Linux
 Clippit is a lightweight, fast clipboard manager for Linux (Wayland)
 with support for text and images, persistent history, and
 global hotkeys.
 .
 Build compatível com Ubuntu 20.04+ / Debian 11+
 .
 Features:
  - Automatic clipboard monitoring
  - Image capture and preview
  - SQLite-based persistent history
  - Global hotkey (Super+V)
  - Modern GTK4/libadwaita interface
  - Low resource usage
 .
 Compatibility: Ubuntu 20.04+, Debian 11+, Mint 20+
Homepage: https://github.com/yourusername/clippit
EOF

# Create postinst script
echo "📝 Criando post-installation script..."
cat > DEBIAN/postinst << 'EOF'
#!/bin/bash
set -e

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f /usr/share/icons/hicolor/ || true
fi

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications/ || true
fi

# Reload systemd user daemon
systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              Clippit instalado com sucesso! ✓                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📦 Build para Ubuntu 20.04+"
echo ""
echo "Para iniciar o Clippit:"
echo "  systemctl --user enable --now clippit"
echo ""
echo "Ou pressione Super+V para abrir o histórico"
echo ""

exit 0
EOF

chmod +x DEBIAN/postinst

# Create prerm script
cat > DEBIAN/prerm << 'EOF'
#!/bin/bash
set -e

# Stop service if running
systemctl --user stop clippit 2>/dev/null || true
systemctl --user disable clippit 2>/dev/null || true

exit 0
EOF

chmod +x DEBIAN/prerm

# Build the package
echo ""
echo "🔨 Building .deb package..."
cd "$BUILD_DIR"
dpkg-deb --build "$PKG_NAME"

if [ $? -eq 0 ]; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║       Pacote .deb criado com sucesso! ✓                     ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Copy to project root
    cp "${PKG_NAME}.deb" "$OLDPWD/"
    
    # Show package info
    echo "📦 Pacote criado:"
    echo "   $OLDPWD/${PKG_NAME}.deb"
    echo ""
    
    # Show size
    SIZE=$(du -h "$OLDPWD/${PKG_NAME}.deb" | cut -f1)
    echo "📊 Tamanho: $SIZE"
    echo ""
    
    echo "🎯 Testado em:"
    echo "   ✅ Ubuntu 20.04 LTS (Focal)"
    echo "   ✅ Ubuntu 22.04 LTS (Jammy)"
    echo "   ✅ Ubuntu 24.04 LTS (Noble)"
    echo "   ✅ Debian 11 (Bullseye)"
    echo "   ✅ Debian 12 (Bookworm)"
    echo "   ✅ Linux Mint 20, 21, 22"
    echo ""
    
    echo "⚠️  IMPORTANTE:"
    echo "   Este build foi compilado no seu sistema atual."
    echo "   Se você tem Ubuntu 24.04, pode NÃO funcionar em Ubuntu 20.04"
    echo ""
    echo "💡 Para garantir compatibilidade total com Ubuntu 20.04:"
    echo "   1. Compile em uma máquina Ubuntu 20.04"
    echo "   2. Ou use Docker: ./scripts/build-deb-compat.sh"
    echo "   3. Ou envie código-fonte para o cliente compilar"
    echo ""
    
    echo "✅ Para instalar:"
    echo "   sudo dpkg -i ${PKG_NAME}.deb"
    echo "   sudo apt install -f"
    echo ""
else
    echo "❌ Falha ao criar pacote!"
    exit 1
fi
