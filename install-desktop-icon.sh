#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎨 Instalando Ícone e .desktop do Clippit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 1. Remover arquivos antigos (requer sudo para /usr/share)
echo "🧹 Limpando instalações antigas..."
sudo rm -f /usr/share/applications/clippit.desktop 2>/dev/null || true
sudo rm -f /usr/local/bin/clippit-* 2>/dev/null || true
rm -f ~/.local/share/applications/clippit.desktop 2>/dev/null || true
echo "✅ Arquivos antigos removidos"

# 2. Criar diretórios necessários
echo ""
echo "📁 Criando diretórios..."
mkdir -p ~/.local/share/applications
mkdir -p ~/.local/share/icons/hicolor/256x256/apps
echo "✅ Diretórios criados"

# 3. Copiar ícone
echo ""
echo "🎨 Instalando ícone..."
if [ -f "assets/logo_clippit.png" ]; then
    cp assets/logo_clippit.png ~/.local/share/icons/hicolor/256x256/apps/clippit.png
    chmod 644 ~/.local/share/icons/hicolor/256x256/apps/clippit.png
    echo "✅ Ícone instalado: ~/.local/share/icons/hicolor/256x256/apps/clippit.png"
else
    echo "❌ Arquivo assets/logo_clippit.png não encontrado!"
    exit 1
fi

# 4. Criar arquivo .desktop
echo ""
echo "🖥️  Criando arquivo .desktop..."
cat > ~/.local/share/applications/clippit.desktop << 'EOF'
[Desktop Entry]
Type=Application
Name=Clippit
GenericName=Clipboard Manager
Comment=Modern clipboard manager with OCR for Wayland and X11
Icon=clippit
Exec=clippit-dashboard
Terminal=false
Categories=Utility;System;
Keywords=clipboard;manager;history;copy;paste;wayland;ocr;
StartupNotify=true
StartupWMClass=Clippit
X-GNOME-UsesNotifications=true
X-GNOME-Autostart-enabled=false
EOF

chmod 644 ~/.local/share/applications/clippit.desktop
echo "✅ Arquivo .desktop criado: ~/.local/share/applications/clippit.desktop"

# 5. Atualizar caches
echo ""
echo "🔄 Atualizando caches do sistema..."
update-desktop-database ~/.local/share/applications 2>/dev/null
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null
xdg-desktop-menu forceupdate 2>/dev/null
echo "✅ Caches atualizados"

# 6. Recarregar GNOME Shell (se disponível)
echo ""
echo "🔄 Recarregando menu do GNOME..."
if command -v gdbus &> /dev/null; then
    gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell \
        --method org.gnome.Shell.Eval "Main.overview.hide(); Main.overview.show();" &>/dev/null || true
    echo "✅ Menu do GNOME recarregado"
fi

# 7. Verificar instalação
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Instalação Concluída!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 Arquivos instalados:"
echo "   🎨 Ícone: ~/.local/share/icons/hicolor/256x256/apps/clippit.png"
echo "   🖥️  Desktop: ~/.local/share/applications/clippit.desktop"
echo ""
echo "🎯 Como encontrar o app:"
echo "   1. Pressione Super (tecla Windows)"
echo "   2. Digite 'Clippit' na busca"
echo "   3. Ou clique em 'Mostrar aplicativos' (grid 3x3) → Utilitários"
echo ""
echo "💡 Se ainda não aparecer:"
echo "   - Faça logout e login novamente"
echo "   - Ou reinicie o sistema"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
