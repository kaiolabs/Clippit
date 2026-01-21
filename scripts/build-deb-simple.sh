#!/bin/bash

set -e

VERSION="1.0.0"
ARCH="amd64"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║           Clippit - Build .deb (Compilação Local)           ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

echo "📍 Compilando no sistema atual..."
echo "   (O pacote gerado funcionará em sistemas similares ao seu)"
echo ""

# Check GTK4
if ! pkg-config --exists gtk4; then
    echo "❌ GTK4 não encontrado!"
    echo "   Instale com: sudo apt install libgtk-4-dev libadwaita-1-dev"
    exit 1
fi

GTK_VERSION=$(pkg-config --modversion gtk4)
echo "✅ GTK4 encontrado: v${GTK_VERSION}"
echo ""

# Clean previous build
echo "🧹 Limpando build anterior..."
cargo clean
rm -f clippit_*.deb

# Build with release optimizations
echo "🔨 Compilando Clippit..."
echo "   (Isso pode demorar alguns minutos)"
echo ""

cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Falha na compilação!"
    exit 1
fi

echo ""
echo "✅ Compilação concluída!"
echo ""

# Create .deb package
echo "📦 Criando pacote .deb..."
./scripts/build-deb.sh

if [ $? -eq 0 ]; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║              ✅ Pacote .deb criado com sucesso!              ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    
    # Show package info
    DEB_FILE=$(ls -t clippit_*.deb 2>/dev/null | head -1)
    if [ -n "$DEB_FILE" ]; then
        SIZE=$(du -h "$DEB_FILE" | cut -f1)
        echo "📦 Pacote: $DEB_FILE"
        echo "📊 Tamanho: $SIZE"
        echo ""
        
        # Get system info
        OS_NAME=$(lsb_release -is 2>/dev/null || echo "Linux")
        OS_VERSION=$(lsb_release -rs 2>/dev/null || echo "Unknown")
        GLIBC_VERSION=$(ldd --version | head -1 | grep -oE '[0-9]+\.[0-9]+$')
        
        echo "🖥️  Compilado em:"
        echo "   • Sistema: $OS_NAME $OS_VERSION"
        echo "   • glibc: $GLIBC_VERSION"
        echo "   • GTK4: $GTK_VERSION"
        echo ""
        
        echo "✅ Este pacote funcionará em:"
        echo "   • $OS_NAME $OS_VERSION (garantido)"
        echo "   • Versões mais recentes do $OS_NAME"
        echo "   • Outras distros com glibc $GLIBC_VERSION+ e GTK4"
        echo ""
        
        echo "⚠️  Compatibilidade:"
        echo "   • ✅ Sistemas iguais ou mais novos que o seu"
        echo "   • ❌ Sistemas mais antigos (glibc/GTK4 incompatíveis)"
        echo ""
        
        echo "💡 Para máxima compatibilidade:"
        echo "   • Compile em Ubuntu 22.04 (funciona em 22.04+)"
        echo "   • Ou distribua código-fonte para usuários compilarem"
        echo ""
        
        echo "📥 Para instalar:"
        echo "   sudo dpkg -i $DEB_FILE"
        echo "   sudo apt install -f"
        echo ""
    fi
else
    echo "❌ Falha ao criar pacote!"
    exit 1
fi
