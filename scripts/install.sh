#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║          Clippit Clipboard Manager - Installer               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "❌ Error: This script is only for Linux systems"
    exit 1
fi

# Check if daemon is running and stop it
if systemctl --user is-active clippit.service &>/dev/null; then
    echo "🔄 Daemon already running, stopping for safe reinstallation..."
    systemctl --user stop clippit.service
    sleep 1
    echo "✓ Daemon stopped"
    echo ""
    DAEMON_WAS_RUNNING=true
elif pgrep -x "clippit-daemon" > /dev/null; then
    echo "🔄 Daemon running manually, stopping for safe reinstallation..."
    pkill clippit-daemon
    sleep 1
    echo "✓ Daemon stopped"
    echo ""
    DAEMON_WAS_RUNNING=true
else
    DAEMON_WAS_RUNNING=false
fi

# Check if Wayland
if [ "$XDG_SESSION_TYPE" == "wayland" ]; then
    echo "✅ Wayland session detected - Clippit will work properly!"
elif [ "$XDG_SESSION_TYPE" == "x11" ]; then
    echo "⚠️  X11 session detected!"
    echo "   Clippit requires Wayland. X11 support has been removed."
    echo "   Please switch to a Wayland session to use Clippit."
fi

# Build release
echo "📦 Building Clippit in release mode..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✓ Build successful"

# Create directories
echo "📁 Creating directories..."
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/clippit
mkdir -p ~/.config/systemd/user
mkdir -p ~/.config/clippit
mkdir -p ~/.config/clippit

# Copy binaries
echo "📋 Installing binaries..."
cp -f target/release/clippit-daemon ~/.local/bin/
cp -f target/release/clippit-ui ~/.local/bin/
cp -f target/release/clippit-dashboard ~/.local/bin/ 2>/dev/null || echo "  ⚠ clippit-dashboard not built (requires Qt6)"
cp -f target/release/clippit-popup ~/.local/bin/ 2>/dev/null || echo "  ⚠ clippit-popup not built (requires Qt6)"

chmod +x ~/.local/bin/clippit-daemon
chmod +x ~/.local/bin/clippit-ui
chmod +x ~/.local/bin/clippit-dashboard 2>/dev/null || true
chmod +x ~/.local/bin/clippit-popup 2>/dev/null || true

echo "✓ Binaries installed to ~/.local/bin/"

# Check if ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo ""
    echo "⚠️  Warning: ~/.local/bin is not in your PATH"
    echo "   Add this line to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo '   export PATH="$HOME/.local/bin:$PATH"'
    echo ""
fi

# Create systemd service
echo "🔧 Creating systemd service..."
cat > ~/.config/systemd/user/clippit.service << EOF
[Unit]
Description=Clippit Clipboard Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.local/bin/clippit-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

echo "✓ Systemd service created"

# Reload systemd
systemctl --user daemon-reload

# Create default config if doesn't exist
if [ ! -f ~/.config/clippit/config.toml ]; then
    if [ -f clippit.example.toml ]; then
        echo "📝 Creating default configuration..."
        cp clippit.example.toml ~/.config/clippit/config.toml
        echo "✓ Configuration created at ~/.config/clippit/config.toml"
    fi
fi

# Ask user if they want to enable autostart
echo ""
if [ "$DAEMON_WAS_RUNNING" = true ]; then
    # If daemon was running, just restart it
    echo "🔄 Restarting Clippit daemon..."
    systemctl --user daemon-reload
    systemctl --user start clippit.service
    
    if [ $? -eq 0 ]; then
        echo "✓ Clippit daemon restarted successfully"
    else
        echo "❌ Failed to restart daemon"
        exit 1
    fi
else
    # New installation - ask user
    read -p "Enable Clippit to start automatically on login? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        echo "⏭️  Skipping autostart setup"
    else
        echo "🚀 Enabling and starting Clippit daemon..."
        systemctl --user enable clippit.service
        systemctl --user start clippit.service
        
        if [ $? -eq 0 ]; then
            echo "✓ Clippit daemon is now running"
        else
            echo "❌ Failed to start daemon"
            exit 1
        fi
    fi
fi

# Install icon and desktop entry
echo ""
echo "🎨 Installing icon and desktop entry..."
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
mkdir -p ~/.local/share/icons/hicolor

# Install icon
if [ -f "assets/logo_clippit.png" ]; then
    cp -f assets/logo_clippit.png ~/.local/share/icons/hicolor/256x256/apps/clippit.png
    echo "✓ Icon installed"
else
    echo "⚠️  Warning: assets/logo_clippit.png not found"
fi

# Create icon theme index if it doesn't exist
if [ ! -f ~/.local/share/icons/hicolor/index.theme ]; then
    cat > ~/.local/share/icons/hicolor/index.theme << 'EOF'
[Icon Theme]
Name=Hicolor
Comment=Fallback icon theme
Hidden=true
Directories=256x256/apps

[256x256/apps]
Size=256
Type=Threshold
EOF
    echo "✓ Icon theme index created"
fi

# Install desktop entry
if [ -f "assets/clippit.desktop" ]; then
    cp -f assets/clippit.desktop ~/.local/share/applications/clippit.desktop
    echo "✓ Desktop entry installed"
else
    echo "⚠️  Warning: assets/clippit.desktop not found"
fi

# Update desktop database and icon cache
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database ~/.local/share/applications/ &> /dev/null || true
    echo "✓ Desktop database updated"
fi

if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f ~/.local/share/icons/hicolor/ &> /dev/null || true
    echo "✓ Icon cache updated"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                  Installation Complete! ✓                     ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
if [ "$DAEMON_WAS_RUNNING" = true ]; then
    echo "♻️  Note: Daemon was automatically restarted with new version"
    echo ""
fi
echo "📋 Usage:"
echo "   • Daemon: systemctl --user status clippit"
echo "   • Hotkey: Press Super+V to show history"
echo "   • Manual: Run 'clippit-ui' from terminal"
echo ""
echo "⚙️  Configuration:"
echo "   Edit: ~/.config/clippit/config.toml"
echo "   Docs: See CONFIGURATION.md"
echo ""
echo "📖 View logs:"
echo "   journalctl --user -u clippit -f"
echo ""
echo "🔧 Manage service:"
echo "   systemctl --user start|stop|restart clippit"
echo ""
echo "🔄 Reinstall/Update:"
echo "   ./scripts/reinstall.sh"
echo ""
echo "🗑️  Uninstall:"
echo "   ./scripts/uninstall.sh"
echo ""
