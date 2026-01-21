#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║        Clippit Clipboard Manager - Uninstaller               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Stop and disable service
if systemctl --user is-active clippit.service &>/dev/null; then
    echo "🛑 Stopping Clippit daemon..."
    systemctl --user stop clippit.service
fi

if systemctl --user is-enabled clippit.service &>/dev/null; then
    echo "❌ Disabling autostart..."
    systemctl --user disable clippit.service
fi

# Remove systemd service
if [ -f ~/.config/systemd/user/clippit.service ]; then
    echo "🗑️  Removing systemd service..."
    rm ~/.config/systemd/user/clippit.service
    systemctl --user daemon-reload
fi

# Remove binaries
echo "🗑️  Removing binaries..."
rm -f ~/.local/bin/clippit-daemon
rm -f ~/.local/bin/clippit-ui
rm -f ~/.local/bin/clippit-dashboard
rm -f ~/.local/bin/clippit-popup

# Remove desktop entries
echo "🗑️  Removing desktop entries..."
rm -f ~/.local/share/applications/clippit.desktop
rm -f ~/.local/share/applications/clippit-dashboard.desktop
rm -f ~/.local/share/applications/clippit-popup.desktop

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
fi

# Ask about data
echo ""
read -p "Remove clipboard history data? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -d ~/.local/share/clippit ]; then
        echo "🗑️  Removing data directory..."
        rm -rf ~/.local/share/clippit
    fi
else
    echo "⏭️  Keeping data directory: ~/.local/share/clippit"
fi

# Ask about config
echo ""
read -p "Remove configuration file? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    if [ -f ~/.config/clippit/config.toml ]; then
        echo "🗑️  Removing configuration..."
        rm -f ~/.config/clippit/config.toml
        rmdir ~/.config/clippit 2>/dev/null || true
    fi
else
    echo "⏭️  Keeping configuration: ~/.config/clippit/config.toml"
fi

# Remove socket if exists
if [ -S /tmp/clippit.sock ]; then
    echo "🗑️  Removing IPC socket..."
    rm /tmp/clippit.sock
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              Uninstallation Complete! ✓                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Clippit has been removed from your system."
echo ""
