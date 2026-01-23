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
sudo cp target/release/clippit-tooltip /usr/local/bin/clippit-tooltip

# Dar permissões de execução
sudo chmod +x /usr/local/bin/clippit-daemon
sudo chmod +x /usr/local/bin/clippit-popup
sudo chmod +x /usr/local/bin/clippit-dashboard
sudo chmod +x /usr/local/bin/clippit-tooltip

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

# ============================================================================
# Configuração Automática de Atalho Global (Wayland)
# ============================================================================

# Função para converter formato Clippit → GNOME
convert_clippit_to_gnome_hotkey() {
    local mod=$1
    local key=$2
    
    # Converter modificador
    case $mod in
        "super"|"meta"|"win") mod_gnome="<Super>" ;;
        "ctrl"|"control") mod_gnome="<Primary>" ;;
        "alt") mod_gnome="<Alt>" ;;
        "shift") mod_gnome="<Shift>" ;;
        *) mod_gnome="<Super>" ;;
    esac
    
    # Converter tecla
    case $key in
        "kp_1"|"numpad1") key_gnome="KP_1" ;;
        "kp_2"|"numpad2") key_gnome="KP_2" ;;
        "kp_3"|"numpad3") key_gnome="KP_3" ;;
        "kp_4"|"numpad4") key_gnome="KP_4" ;;
        "kp_5"|"numpad5") key_gnome="KP_5" ;;
        "kp_6"|"numpad6") key_gnome="KP_6" ;;
        "kp_7"|"numpad7") key_gnome="KP_7" ;;
        "kp_8"|"numpad8") key_gnome="KP_8" ;;
        "kp_9"|"numpad9") key_gnome="KP_9" ;;
        "kp_0"|"numpad0") key_gnome="KP_0" ;;
        "kp_end") key_gnome="KP_End" ;;
        *) key_gnome=$(echo "$key" | tr '[:lower:]' '[:upper:]') ;;
    esac
    
    echo "${mod_gnome}${key_gnome}"
}

# Verificar se está no Wayland e se gsettings está disponível
if [ "$XDG_SESSION_TYPE" = "wayland" ] && command -v gsettings &> /dev/null; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔑 Configuração de Atalho Global (Wayland)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "   ⚠️  No Wayland, hotkeys globais precisam ser"
    echo "   configurados através do Sistema Operacional."
    echo ""
    
    # Verificar se já existe atalho configurado
    EXISTING_BINDING=""
    if gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding &>/dev/null; then
        EXISTING_BINDING=$(gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding 2>/dev/null | tr -d "'")
    fi
    
    if [ -n "$EXISTING_BINDING" ]; then
        echo "   ✅ Atalho já configurado: $EXISTING_BINDING"
        echo ""
        read -p "   Deseja reconfigurar? (s/N): " -n 1 -r RECONFIG
        echo ""
        if [[ ! $RECONFIG =~ ^[Ss]$ ]]; then
            echo "   ⏭️  Mantendo atalho existente"
            SKIP_HOTKEY_SETUP=true
        fi
    else
        echo "   Deseja configurar o atalho automaticamente agora?"
        echo ""
        read -p "   Configurar atalho? (S/n): " -n 1 -r SETUP_HOTKEY
        echo ""
        
        if [[ $SETUP_HOTKEY =~ ^[Nn]$ ]]; then
            echo "   ⏭️  Pulando configuração de atalho"
            SKIP_HOTKEY_SETUP=true
        fi
    fi
    
    if [ "$SKIP_HOTKEY_SETUP" != "true" ]; then
        echo ""
        echo "   🔄 Configurando atalho automaticamente..."
        echo ""
        
        # Carregar configuração do Clippit
        CONFIG_FILE="$HOME/.config/clippit/config.toml"
        
        if [ -f "$CONFIG_FILE" ]; then
            MODIFIER=$(grep "show_history_modifier" "$CONFIG_FILE" | cut -d'"' -f2)
            KEY=$(grep "show_history_key" "$CONFIG_FILE" | head -n1 | cut -d'"' -f2)
            echo "   📋 Atalho do Clippit: $MODIFIER + $KEY"
        else
            MODIFIER="super"
            KEY="v"
            echo "   📋 Usando atalho padrão: $MODIFIER + $KEY"
        fi
        
        # Converter para formato GNOME
        GNOME_HOTKEY=$(convert_clippit_to_gnome_hotkey "$MODIFIER" "$KEY")
        echo "   🔄 Formato GNOME: $GNOME_HOTKEY"
        echo ""
        
        # Configurar no gsettings
        NEW_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/"
        
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH name "Clippit - Show History" 2>/dev/null
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH command "/usr/local/bin/clippit-popup" 2>/dev/null
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH binding "$GNOME_HOTKEY" 2>/dev/null
        
        # Adicionar à lista de atalhos personalizados
        CUSTOM_KEYS=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings 2>/dev/null)
        
        if [[ "$CUSTOM_KEYS" == "@as []" ]] || [[ "$CUSTOM_KEYS" == "[]" ]]; then
            NEW_LIST="['$NEW_PATH']"
        else
            # Verificar se já está na lista
            if [[ "$CUSTOM_KEYS" == *"$NEW_PATH"* ]]; then
                NEW_LIST="$CUSTOM_KEYS"
            else
                NEW_LIST=$(echo "$CUSTOM_KEYS" | sed "s/]$/, '$NEW_PATH']/")
            fi
        fi
        
        gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST" 2>/dev/null
        
        if [ $? -eq 0 ]; then
            echo "   ✅ Atalho configurado com sucesso!"
            echo ""
            echo "   🎯 Teste agora: Pressione $MODIFIER + $KEY"
            HOTKEY_CONFIGURED=true
        else
            echo "   ❌ Erro ao configurar atalho automaticamente"
            echo ""
            echo "   📝 Configure manualmente:"
            echo "      Configurações → Teclado → Atalhos → Adicionar"
            echo "      Nome: Clippit - Show History"
            echo "      Comando: /usr/local/bin/clippit-popup"
            echo "      Atalho: $MODIFIER + $KEY"
        fi
    else
        echo ""
        echo "   📝 Para configurar manualmente depois:"
        echo "      Configurações → Teclado → Atalhos → Adicionar"
        echo "      Nome: Clippit - Show History"
        echo "      Comando: /usr/local/bin/clippit-popup"
        echo "      Atalho: Escolha sua combinação"
        echo ""
        echo "   💡 Ou execute: ./scripts/setup-wayland-hotkey.sh"
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

# Mensagem final de teste
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 Teste o Clippit:"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$HOTKEY_CONFIGURED" = "true" ]; then
    echo "   1. ✅ Pressione o atalho configurado para abrir"
else
    echo "   1. ⚙️  Configure o atalho (veja instruções acima)"
fi

echo "   2. 📋 Copie algo (Ctrl+C) e veja no histórico"
echo "   3. 🎨 Configure preferências: clippit-dashboard"
echo ""
echo "💡 Dicas:"
echo "   - Ver logs: journalctl --user -u clippit -f"
echo "   - Autocompletar: Super+Espaço → 'Clippit'"
echo "   - Documentação: docs/WAYLAND_HOTKEYS.md"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"