#!/bin/bash

set -e

VERSION="1.0.0"
ARCH="amd64"
PKG_NAME="clippit_${VERSION}_${ARCH}"
BUILD_DIR="/tmp/clippit-deb-build"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Clippit - Debian Package Builder v1.0              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Clean previous build
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR/$PKG_NAME"

echo "📦 Building Clippit in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✓ Build successful"
echo ""

# Create package structure
echo "📁 Creating package structure..."
cd "$BUILD_DIR/$PKG_NAME"

# Directories
mkdir -p DEBIAN
mkdir -p usr/local/bin
mkdir -p usr/share/applications
mkdir -p usr/share/icons/hicolor/256x256/apps
mkdir -p etc/systemd/user

# Copy binaries
echo "📋 Copying binaries..."
cp "$OLDPWD/target/release/clippit-daemon" usr/local/bin/
cp "$OLDPWD/target/release/clippit-dashboard" usr/local/bin/
cp "$OLDPWD/target/release/clippit-popup" usr/local/bin/
chmod +x usr/local/bin/clippit-*

# Copy assets
echo "🎨 Copying assets..."
cp "$OLDPWD/assets/logo_clippit.png" usr/share/icons/hicolor/256x256/apps/clippit.png
cp "$OLDPWD/assets/clippit.desktop" usr/share/applications/

# Create systemd service
echo "🔧 Creating systemd service..."
cat > etc/systemd/user/clippit.service << 'EOF'
[Unit]
Description=Clippit Clipboard Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/clippit-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

# Create control file
echo "📝 Creating control file..."
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
 Features:
  - Automatic clipboard monitoring
  - Image capture and preview
  - SQLite-based persistent history
  - Global hotkey (Ctrl+;)
  - Modern GTK4/libadwaita interface
  - Low resource usage
Homepage: https://github.com/yourusername/clippit
EOF

# Create postinst script (runs after installation)
echo "📝 Creating post-installation script..."
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

# Import environment variables for systemd user services
systemctl --user import-environment DISPLAY XAUTHORITY 2>/dev/null || true

# Reload systemd user daemon
systemctl --user daemon-reload 2>/dev/null || true

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              Clippit instalado com sucesso! ✓                ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Para iniciar o Clippit:"
echo "  systemctl --user enable --now clippit"
echo ""
echo "Ou reinicie sua sessão para auto-start."
echo ""

exit 0
EOF

chmod +x DEBIAN/postinst

# Create prerm script (runs before uninstallation)
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
    echo "║             Pacote .deb criado com sucesso! ✓                ║"
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
    
    echo "📋 Conteúdo do pacote:"
    dpkg -c "${PKG_NAME}.deb" | head -20
    echo ""
    
    echo "✅ Para instalar:"
    echo "   sudo dpkg -i ${PKG_NAME}.deb"
    echo ""
    echo "✅ Para distribuir:"
    echo "   • Envie o arquivo ${PKG_NAME}.deb para seus clientes"
    echo "   • Eles só precisam executar: sudo dpkg -i ${PKG_NAME}.deb"
    echo "   • Todas as dependências serão instaladas automaticamente!"
    echo ""
else
    echo "❌ Falha ao criar pacote!"
    exit 1
fi
