#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║    Clippit - Build .deb compatível com sistemas antigos     ║"
echo "║              (Ubuntu 22.04+ / Debian 12+)                    ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker não encontrado!"
    echo "   Instale Docker: https://docs.docker.com/engine/install/"
    exit 1
fi

echo "📦 Criando ambiente de build em Ubuntu 22.04..."
echo ""

# Create Dockerfile in current directory
cat > Dockerfile.clippit << 'EOF'
FROM ubuntu:22.04

# Prevent interactive prompts
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    xdotool \
    xclip \
    dpkg-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Verify GTK4 installation
RUN pkg-config --modversion gtk4 && \
    echo "✅ GTK4 $(pkg-config --modversion gtk4) encontrado em $(pkg-config --variable=prefix gtk4)"

WORKDIR /build

# Build script - ensure PKG_CONFIG_PATH is set for cargo
CMD ["bash", "-c", "\
    export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig && \
    echo '🔍 Verificando ambiente de build...' && \
    echo '   GTK4: '$(pkg-config --modversion gtk4) && \
    echo '   Rust: '$(rustc --version) && \
    echo '   Cargo: '$(cargo --version) && \
    echo '' && \
    echo '🚀 Compilando Clippit (pode demorar 10-15 minutos)...' && \
    echo '' && \
    cargo build --release 2>&1 | tee /tmp/build.log && \
    echo '' && \
    echo '📦 Criando pacote .deb...' && \
    ./scripts/build-deb.sh \
"]
EOF

# Build Docker image
echo "🔨 Construindo imagem Docker..."
docker build -t clippit-builder:ubuntu22.04 -f Dockerfile.clippit .

if [ $? -ne 0 ]; then
    echo "❌ Falha ao construir imagem Docker!"
    exit 1
fi

echo ""
echo "✅ Imagem Docker criada!"
echo ""
echo "🚀 Compilando Clippit em Ubuntu 22.04..."
echo ""

# Run build in Docker
docker run --rm \
    -v "$(pwd)":/build \
    -v "$HOME/.cargo/registry:/root/.cargo/registry" \
    clippit-builder:ubuntu22.04

if [ $? -eq 0 ]; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║        ✅ Build compatível criado com sucesso!               ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "📦 Pacote criado: clippit_1.0.0_amd64.deb"
    echo ""
    echo "✅ Este pacote funciona em:"
    echo "   • Ubuntu 22.04+ (Jammy, Noble)"
    echo "   • Debian 12+ (Bookworm)"
    echo "   • Linux Mint 21+"
    echo "   • Pop!_OS 22.04+"
    echo "   • E outras distribuições com GTK4 e glibc 2.35+"
    echo ""
    echo "⚠️  Nota: Ubuntu 20.04 NÃO é suportado (não tem GTK4)"
    echo ""
    
    # Cleanup
    rm -f Dockerfile.clippit
else
    echo "❌ Falha no build!"
    rm -f Dockerfile.clippit
    exit 1
fi
