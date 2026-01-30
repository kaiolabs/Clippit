#!/bin/bash
echo "🔧 Corrigindo arquivo .desktop do Clippit..."
echo ""

# Remover arquivo antigo do /usr/share (requer sudo)
if [ -f /usr/share/applications/clippit.desktop ]; then
    echo "❌ Removendo arquivo antigo em /usr/share/applications..."
    sudo rm -f /usr/share/applications/clippit.desktop
    echo "✅ Arquivo antigo removido!"
else
    echo "✓ Nenhum arquivo antigo em /usr/share/applications"
fi

# Atualizar caches
echo ""
echo "🔄 Atualizando caches..."
update-desktop-database ~/.local/share/applications
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor
xdg-desktop-menu forceupdate

# Recarregar GNOME
echo ""
echo "🔄 Recarregando menu do GNOME..."
gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell \
    --method org.gnome.Shell.Eval "Main.overview.hide(); Main.overview.show();" &>/dev/null || true

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Pronto!"
echo ""
echo "Agora pressione Super e procure por 'Clippit'"
echo "Ou vá em 'Mostrar aplicativos' (grid) → Utilitários"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
