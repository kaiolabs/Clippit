#!/bin/bash

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║      Clippit Clipboard Manager - Reinstalação Segura        ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Verificar se está rodando via systemd
if systemctl --user is-active clippit.service &>/dev/null; then
    echo "🛑 Parando serviço systemd..."
    systemctl --user stop clippit.service
    echo "✓ Serviço parado"
elif pgrep -x "clippit-daemon" > /dev/null; then
    echo "🛑 Parando daemon em execução..."
    pkill -9 clippit-daemon
    sleep 1
    echo "✓ Daemon parado"
else
    echo "✓ Nenhum daemon em execução"
fi

# Aguardar para garantir que o processo foi finalizado
sleep 1

# Build release
echo ""
echo "📦 Compilando em modo release..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Compilação falhou!"
    exit 1
fi

echo "✓ Compilação concluída"

# Criar diretórios
echo ""
echo "📁 Verificando diretórios..."
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/clippit
mkdir -p ~/.config/systemd/user
mkdir -p ~/.config/clippit

# Copiar binários
echo ""
echo "📋 Instalando binários..."
cp -f target/release/clippit-daemon ~/.local/bin/
cp -f target/release/clippit-ui ~/.local/bin/

chmod +x ~/.local/bin/clippit-daemon
chmod +x ~/.local/bin/clippit-ui

echo "✓ Binários instalados"

# Verificar arquivo de configuração
echo ""
if [ ! -f ~/.config/clippit/config.toml ]; then
    echo "📝 Criando arquivo de configuração padrão..."
    cp clippit.example.toml ~/.config/clippit/config.toml
    echo "✓ Configuração criada em ~/.config/clippit/config.toml"
else
    echo "✓ Arquivo de configuração já existe"
fi

# Verificar systemd service
if [ -f ~/.config/systemd/user/clippit.service ]; then
    echo ""
    echo "🔄 Recarregando systemd..."
    systemctl --user daemon-reload
    
    echo "🚀 Reiniciando serviço..."
    systemctl --user restart clippit.service
    
    if [ $? -eq 0 ]; then
        echo "✓ Serviço reiniciado com sucesso"
    else
        echo "❌ Erro ao reiniciar serviço"
        exit 1
    fi
else
    echo ""
    echo "⚠️  Serviço systemd não configurado"
    echo "   Execute './scripts/install.sh' para configuração completa"
fi

echo ""
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║              Reinstalação Concluída! ✓                       ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""
echo "📊 Status do serviço:"
systemctl --user status clippit.service --no-pager | head -10
echo ""
echo "📋 Comandos úteis:"
echo "   Ver logs:     journalctl --user -u clippit -f"
echo "   Parar:        systemctl --user stop clippit"
echo "   Reiniciar:    systemctl --user restart clippit"
echo "   Status:       systemctl --user status clippit"
echo ""
