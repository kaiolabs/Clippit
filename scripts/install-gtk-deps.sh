#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      Clippit - Instalador de Dependências GTK4              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Detect OS
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
else
    echo "❌ Não foi possível detectar o sistema operacional"
    exit 1
fi

echo "🔍 Sistema detectado: $PRETTY_NAME"
echo ""

case "$OS" in
    ubuntu|debian|zorin|pop|linuxmint)
        echo "📦 Instalando dependências para Ubuntu/Debian..."
        sudo apt update
        sudo apt install -y \
            libgtk-4-dev \
            libadwaita-1-dev \
            libgraphene-1.0-dev \
            build-essential \
            pkg-config
        ;;
    
    fedora)
        echo "📦 Instalando dependências para Fedora..."
        sudo dnf install -y \
            gtk4-devel \
            libadwaita-devel \
            graphene-devel \
            gcc \
            pkg-config
        ;;
    
    arch|manjaro)
        echo "📦 Instalando dependências para Arch Linux..."
        sudo pacman -S --needed --noconfirm \
            gtk4 \
            libadwaita \
            graphene \
            base-devel \
            pkg-config
        ;;
    
    *)
        echo "❌ Sistema operacional não suportado: $OS"
        echo ""
        echo "Instale manualmente as seguintes bibliotecas:"
        echo "  - GTK4 (>= 4.10)"
        echo "  - libadwaita (>= 1.4)"
        echo "  - graphene (>= 1.10)"
        echo "  - build-essential / base-devel"
        echo "  - pkg-config"
        exit 1
        ;;
esac

echo ""
echo "✅ Dependências instaladas com sucesso!"
echo ""
echo "🔍 Verificando instalação..."

# Verify installation
if pkg-config --exists gtk4; then
    GTK_VERSION=$(pkg-config --modversion gtk4)
    echo "  ✓ GTK4: $GTK_VERSION"
else
    echo "  ✗ GTK4: não encontrado"
    exit 1
fi

if pkg-config --exists libadwaita-1; then
    ADWAITA_VERSION=$(pkg-config --modversion libadwaita-1)
    echo "  ✓ libadwaita: $ADWAITA_VERSION"
else
    echo "  ✗ libadwaita: não encontrado"
    exit 1
fi

if pkg-config --exists graphene-gobject-1.0; then
    GRAPHENE_VERSION=$(pkg-config --modversion graphene-gobject-1.0)
    echo "  ✓ graphene: $GRAPHENE_VERSION"
else
    echo "  ✗ graphene: não encontrado"
    exit 1
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║            Pronto para compilar o Clippit! 🚀               ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "Próximos passos:"
echo "  1. cargo build --release"
echo "  2. ./scripts/install.sh"
echo ""
