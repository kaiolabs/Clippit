#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Compilando e Atualizando Clippit..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Mostrar versão atual instalada
echo "📦 Versão atual instalada:"
if [ -f /usr/local/bin/clippit-daemon ]; then
    timeout 1 /usr/local/bin/clippit-daemon --version 2>/dev/null | head -1 | sed 's/^/   /' || echo "   (não disponível ou versão antiga)"
else
    echo "   (não instalado)"
fi
echo ""

# Mostrar versão que será instalada
NOVA_VERSAO=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "🚀 Versão que será instalada: $NOVA_VERSAO"
echo ""

# Compilar tudo
echo "🏗️  Compilando em modo release..."
cargo build --release

if [ $? -ne 0 ]; then
    echo "❌ Erro na compilação!"
    exit 1
fi

echo ""
echo "✅ Compilação concluída!"
echo ""

# Parar daemon
echo "⏹️  Parando daemon..."
systemctl --user stop clippit 2>/dev/null || true

# Aguardar systemd parar completamente
sleep 1

# Limpar processos antigos (mais agressivo)
echo "🧹 Limpando processos antigos..."
killall -9 clippit-daemon 2>/dev/null || true
killall -9 clippit-popup 2>/dev/null || true
killall -9 clippit-dashboard 2>/dev/null || true
killall -9 clippit-ibus 2>/dev/null || true

# Aguardar processos terminarem
sleep 1

# Verificar se ainda há processos rodando
if ps aux | grep -E "clippit-(daemon|popup|dashboard|ibus)" | grep -v grep > /dev/null; then
    echo "⚠️  Ainda há processos rodando, matando com força..."
    pkill -9 clippit-daemon 2>/dev/null || true
    pkill -9 clippit-popup 2>/dev/null || true
    sleep 1
fi

# Limpar lock files
rm -f /tmp/clippit-popup.lock 2>/dev/null || true

# Remover binários antigos primeiro
echo "🗑️  Removendo binários antigos..."
sudo rm -f /usr/local/bin/clippit-daemon
sudo rm -f /usr/local/bin/clippit-popup
sudo rm -f /usr/local/bin/clippit-dashboard

# Instalar binários novos
echo "📦 Instalando binários novos..."
sudo cp target/release/clippit-daemon /usr/local/bin/clippit-daemon
sudo cp target/release/clippit-popup /usr/local/bin/clippit-popup
sudo cp target/release/clippit-dashboard /usr/local/bin/clippit-dashboard

# Dar permissões de execução
sudo chmod +x /usr/local/bin/clippit-daemon
sudo chmod +x /usr/local/bin/clippit-popup
sudo chmod +x /usr/local/bin/clippit-dashboard

# Instalar IBus Component (Autocomplete Global)
echo "⌨️  Instalando IBus Component (Autocomplete Global)..."
if [ -f "target/release/clippit-ibus" ]; then
    sudo cp target/release/clippit-ibus /usr/local/bin/clippit-ibus
    sudo chmod +x /usr/local/bin/clippit-ibus
    
    # Instalar XML component definition
    sudo mkdir -p /usr/share/ibus/component
    sudo cp crates/clippit-ibus/data/clippit.xml /usr/share/ibus/component/
    
    # Reiniciar IBus (se estiver rodando)
    if command -v ibus &> /dev/null; then
        ibus restart &>/dev/null &
    fi
    
    echo "✅ IBus Component instalado (configure em Settings → Keyboard → Input Sources)"
else
    echo "⚠️  clippit-ibus não encontrado, pulando instalação do IBus"
fi

# Instalar ícone em múltiplos tamanhos (importante para Wayland/GNOME)
echo "🎨 Instalando ícone..."

# Verificar se o arquivo existe
if [ ! -f "assets/logo_clippit.png" ]; then
    echo "⚠️  Arquivo de ícone não encontrado!"
else
    # Instalar em múltiplos tamanhos para melhor compatibilidade
    for size in 48 128 256 512; do
        sudo mkdir -p /usr/share/icons/hicolor/${size}x${size}/apps
        # Se tiver imagemagick, redimensiona; senão usa o original
        if command -v convert &> /dev/null; then
            convert assets/logo_clippit.png -resize ${size}x${size} /tmp/clippit_${size}.png 2>/dev/null
            sudo cp /tmp/clippit_${size}.png /usr/share/icons/hicolor/${size}x${size}/apps/clippit.png
            rm -f /tmp/clippit_${size}.png
        else
            sudo cp assets/logo_clippit.png /usr/share/icons/hicolor/${size}x${size}/apps/clippit.png
        fi
        sudo chmod 644 /usr/share/icons/hicolor/${size}x${size}/apps/clippit.png
    done
    
    # Atualizar cache de ícones
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor/ 2>/dev/null || true
    echo "✅ Ícone instalado em múltiplos tamanhos"
fi

# Instalar arquivo .desktop (importante para Wayland)
echo "📋 Instalando arquivo .desktop..."
sudo mkdir -p /usr/share/applications
sudo cp assets/clippit.desktop /usr/share/applications/clippit.desktop
sudo chmod 644 /usr/share/applications/clippit.desktop
sudo update-desktop-database /usr/share/applications/ 2>/dev/null || true

# Verificar se foram copiados
echo "✅ Verificando instalação..."
echo ""
echo "📅 Data dos binários:"
ls -lh /usr/local/bin/clippit-* --time-style=+"%Y-%m-%d %H:%M:%S" | awk '{print "   "$6, $7, $9}'
echo ""
echo "📌 Versão instalada:"
/usr/local/bin/clippit-daemon --version 2>/dev/null | head -2 | sed 's/^/   /'

# ========== CONFIGURAR AUTOCOMPLETAR GLOBAL (IBus) ==========
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⌨️  Configurando Autocompletar Global (IBus)..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Instalar componente IBus se o script existir
if [ -f "scripts/install-ibus.sh" ]; then
    echo "📦 Instalando componente IBus..."
    sudo bash scripts/install-ibus.sh
else
    echo "⚠️  Script install-ibus.sh não encontrado, pulando..."
fi

# Configurar automaticamente as fontes de entrada
echo "🔧 Configurando fontes de entrada do sistema..."

# Verificar se gsettings está disponível (GNOME/Zorin)
if command -v gsettings &> /dev/null; then
    # Obter fontes de entrada atuais
    CURRENT_SOURCES=$(gsettings get org.gnome.desktop.input-sources sources 2>/dev/null || echo "[]")
    
    # Verificar se Clippit já está adicionado
    if echo "$CURRENT_SOURCES" | grep -q "ibus.*clippit"; then
        echo "✅ Clippit já está nas fontes de entrada!"
    else
        echo "➕ Adicionando Clippit às fontes de entrada..."
        
        # Remover os colchetes e adicionar Clippit
        if [ "$CURRENT_SOURCES" = "[]" ]; then
            # Nenhuma fonte configurada, adicionar teclado padrão + clippit
            gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'br'), ('ibus', 'clippit')]"
        else
            # Já tem fontes, adicionar Clippit ao final
            NEW_SOURCES=$(echo "$CURRENT_SOURCES" | sed "s/]$/, ('ibus', 'clippit')]/")
            gsettings set org.gnome.desktop.input-sources sources "$NEW_SOURCES"
        fi
        
        echo "✅ Clippit adicionado às fontes de entrada!"
        echo ""
        echo "💡 Como usar o autocompletar:"
        echo "   1. Pressione Super+Espaço para alternar para 'Clippit'"
        echo "   2. Digite em qualquer aplicativo"
        echo "   3. Sugestões aparecem automaticamente baseadas no seu histórico!"
    fi
else
    echo "⚠️  gsettings não encontrado (sistema não é GNOME/Zorin)"
    echo "   Configure manualmente: Configurações → Teclado → Fontes de Entrada"
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Criar serviço systemd se não existir
if [ ! -f ~/.config/systemd/user/clippit.service ]; then
    echo "📦 Criando serviço systemd..."
    mkdir -p ~/.config/systemd/user
    cat > ~/.config/systemd/user/clippit.service << 'EOF'
[Unit]
Description=Clippit Clipboard Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/clippit-daemon
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
    echo "✅ Serviço systemd criado"
fi

# Recarregar systemd para garantir que pegue os novos binários
echo "🔄 Recarregando systemd..."
systemctl --user daemon-reload

# Habilitar se ainda não estiver
if ! systemctl --user is-enabled clippit &>/dev/null; then
    echo "🔧 Habilitando serviço..."
    systemctl --user enable clippit
fi

# Reiniciar daemon
echo "🚀 Iniciando daemon..."
systemctl --user start clippit

# Aguardar iniciar
sleep 2

# Verificar status
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Compilação e Atualização Completas!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
systemctl --user status clippit --no-pager
echo ""

# Ver logs recentes
echo "📋 Últimos logs:"
journalctl --user -u clippit -n 5 --no-pager | grep -i "atalho\|hotkey" || echo "   (aguardando atividade...)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Teste agora:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "   1. Pressione seu atalho para abrir o popup"
echo "   2. Selecione e copie algo do histórico"
echo "   3. Veja se aparece a notificação do sistema"
echo ""
echo "💡 Dicas:"
echo "   - Ver logs: journalctl --user -u clippit -f"
echo "   - Configurar: clippit-dashboard"
echo "   - Autocompletar: Super+Espaço → Selecione 'Clippit'"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"