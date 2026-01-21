#!/bin/bash

set -e

echo "🎨 Atualizando ícone do Clippit..."
echo ""

# Check if logo exists
if [ ! -f "assets/logo_clippit.png" ]; then
    echo "❌ Erro: assets/logo_clippit.png não encontrado!"
    exit 1
fi

# Create icon directory if doesn't exist
mkdir -p ~/.local/share/icons/hicolor/256x256/apps

# Copy new icon
echo "📋 Copiando nova logo..."
cp -f assets/logo_clippit.png ~/.local/share/icons/hicolor/256x256/apps/clippit.png
echo "✓ Logo copiada"

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
    echo "✓ Icon theme index criado"
fi

# Update icon cache
echo "🔄 Atualizando caches..."
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f ~/.local/share/icons/hicolor/ &> /dev/null || true
    echo "✓ Cache de ícones atualizado"
fi

if command -v update-desktop-database &> /dev/null; then
    update-desktop-database ~/.local/share/applications/ &> /dev/null || true
    echo "✓ Desktop database atualizado"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ ÍCONE ATUALIZADO COM SUCESSO!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 IMPORTANTE:"
echo "   • Feche e abra o menu de aplicativos para ver a mudança"
echo "   • Pode levar alguns segundos para o cache atualizar"
echo "   • Se não aparecer, execute: killall plasmashell (KDE)"
echo "   • Ou: killall gnome-shell (GNOME)"
echo ""
