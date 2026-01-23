#!/usr/bin/env bash
# Script de instalação do Clippit IBus Component

set -e

echo "========================================="
echo "  Clippit IBus Component Installer"
echo "========================================="
echo ""

# Verificar se está rodando com privilégios necessários
if [ "$EUID" -ne 0 ]; then 
    echo "⚠️  Este script precisa ser executado como root (sudo)"
    exit 1
fi

# Compilar o crate clippit-ibus
echo "📦 Compilando clippit-ibus..."
cd "$(dirname "$0")/.."
cargo build --release --package clippit-ibus

# Copiar binário
echo "📋 Instalando binário..."
cp target/release/clippit-ibus /usr/local/bin/
chmod +x /usr/local/bin/clippit-ibus

# Copiar XML component definition
echo "📄 Instalando component definition..."
mkdir -p /usr/share/ibus/component
cp crates/clippit-ibus/data/clippit.xml /usr/share/ibus/component/

# Copiar logo (se necessário)
if [ -f "assets/logo_clippit.png" ]; then
    echo "🎨 Instalando ícone..."
    mkdir -p /usr/local/share/clippit
    cp assets/logo_clippit.png /usr/local/share/clippit/
fi

# Reiniciar IBus
echo "🔄 Reiniciando IBus..."
if command -v ibus &> /dev/null; then
    # Tentar reiniciar para o usuário que invocou sudo
    REAL_USER="${SUDO_USER:-$USER}"
    sudo -u "$REAL_USER" ibus restart &
fi

echo ""
echo "✅ Instalação concluída!"
echo ""
echo "Para ativar o Clippit Autocomplete:"
echo "1. Abra Configurações do Sistema (Settings)"
echo "2. Vá em Teclado → Fontes de Entrada (Keyboard → Input Sources)"
echo "3. Clique em '+' para adicionar"
echo "4. Procure por 'Clippit Autocomplete'"
echo "5. Adicione e ative"
echo ""
echo "Configure no Dashboard: clippit-dashboard"
echo ""
