#!/bin/bash
set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Compilando e Atualizando Clippit..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Mostrar versão atual instalada
echo "📦 Versão atual instalada:"
if [ -f ~/.local/bin/clippit-daemon ]; then
    timeout 1 ~/.local/bin/clippit-daemon --version 2>/dev/null | head -1 | sed 's/^/   /' || echo "   (não disponível ou versão antiga)"
elif [ -f /usr/local/bin/clippit-daemon ]; then
    timeout 1 /usr/local/bin/clippit-daemon --version 2>/dev/null | head -1 | sed 's/^/   /' || echo "   (versão antiga em /usr/local/bin)"
else
    echo "   (não instalado)"
fi
echo ""

# Mostrar versão que será instalada
NOVA_VERSAO=$(grep "^version" Cargo.toml | head -1 | cut -d'"' -f2)
echo "🚀 Versão que será instalada: $NOVA_VERSAO"
echo ""

# Verificar TODAS as dependências do sistema
echo "🔍 Verificando dependências do sistema..."
DEPS_TO_INSTALL=()

# 1. Tesseract OCR (necessário para feature OCR v1.10.0+)
if ! command -v tesseract &> /dev/null; then
    echo "⚠️  Tesseract OCR não instalado (necessário para OCR)"
    DEPS_TO_INSTALL+=(tesseract-ocr libtesseract-dev libleptonica-dev tesseract-ocr-por tesseract-ocr-eng)
elif ! pkg-config --exists lept; then
    echo "⚠️  libleptonica-dev não instalado (necessário para compilação OCR)"
    DEPS_TO_INSTALL+=(libleptonica-dev)
elif ! pkg-config --exists tesseract; then
    echo "⚠️  libtesseract-dev não instalado (necessário para compilação OCR)"
    DEPS_TO_INSTALL+=(libtesseract-dev)
else
    echo "✅ Tesseract OCR instalado"
    
    # Verificar idiomas
    if ! tesseract --list-langs 2>/dev/null | grep -q "por"; then
        echo "⚠️  Dados português não instalados"
        DEPS_TO_INSTALL+=(tesseract-ocr-por)
    fi
    if ! tesseract --list-langs 2>/dev/null | grep -q "eng"; then
        echo "⚠️  Dados inglês não instalados"
        DEPS_TO_INSTALL+=(tesseract-ocr-eng)
    fi
fi

# 2. wmctrl (necessário para gerenciamento de foco do popup v1.11.2+)
if ! command -v wmctrl &> /dev/null; then
    echo "⚠️  wmctrl não instalado (necessário para foco do popup)"
    DEPS_TO_INSTALL+=(wmctrl)
else
    echo "✅ wmctrl instalado"
fi

# Instalar todas as dependências faltantes de uma vez
if [ ${#DEPS_TO_INSTALL[@]} -gt 0 ]; then
    echo ""
    echo "📦 Instalando dependências faltantes: ${DEPS_TO_INSTALL[*]}"
    
    # Tentar instalar com sudo
    if sudo -n true 2>/dev/null; then
        # sudo sem senha disponível
        sudo apt-get update -qq
        sudo apt-get install -y "${DEPS_TO_INSTALL[@]}"
        
        if [ $? -eq 0 ]; then
            echo "✅ Todas as dependências instaladas com sucesso!"
        else
            echo "❌ Falha ao instalar algumas dependências"
            echo "   Execute manualmente: sudo apt-get install -y ${DEPS_TO_INSTALL[*]}"
        fi
    else
        # Precisa de senha
        echo ""
        echo "⚠️  Instalação de dependências requer senha sudo"
        echo "   Execute manualmente: sudo apt-get install -y ${DEPS_TO_INSTALL[*]}"
        echo ""
        echo "⏭️  Continuando compilação sem instalar dependências..."
        echo "   (algumas funcionalidades podem não funcionar corretamente)"
        echo ""
    fi
else
    echo "✅ Todas as dependências já instaladas!"
fi

echo ""
echo "📋 Versões instaladas:"
echo "   Tesseract: $(tesseract --version 2>&1 | head -1)"
echo "   wmctrl: $(wmctrl -v 2>&1 | head -1)"
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
killall -9 clippit-tooltip 2>/dev/null || true

# Aguardar processos terminarem
sleep 1

# Verificar se ainda há processos rodando e matar com força
if ps aux | grep -E "clippit-(daemon|popup|dashboard|ibus|tooltip)" | grep -v grep > /dev/null; then
    echo "⚠️  Ainda há processos rodando, matando com força..."
    pkill -9 clippit-daemon 2>/dev/null || true
    pkill -9 clippit-popup 2>/dev/null || true
    pkill -9 clippit-dashboard 2>/dev/null || true
    pkill -9 clippit-ibus 2>/dev/null || true
    pkill -9 clippit-tooltip 2>/dev/null || true
    sleep 1
fi

# Limpar lock files
rm -f /tmp/clippit-popup.lock 2>/dev/null || true

# Criar diretórios se não existirem
echo "📁 Criando diretórios..."
mkdir -p ~/.local/bin
mkdir -p ~/.local/share/clippit

# Remover binários antigos de ~/.local/bin
echo "🗑️  Removendo binários antigos..."
rm -f ~/.local/bin/clippit-daemon 2>/dev/null || true
rm -f ~/.local/bin/clippit-popup 2>/dev/null || true
rm -f ~/.local/bin/clippit-dashboard 2>/dev/null || true
rm -f ~/.local/bin/clippit-tooltip 2>/dev/null || true

# Remover de /usr/local/bin também (instalações antigas - requer sudo)
if [ -f /usr/local/bin/clippit-daemon ] || [ -f /usr/local/bin/clippit-popup ]; then
    echo "   ⚠️  Detectadas instalações antigas em /usr/local/bin"
    if sudo -n true 2>/dev/null; then
        sudo rm -f /usr/local/bin/clippit-* 2>/dev/null || true
        echo "   ✅ Limpeza de /usr/local/bin concluída"
    else
        echo "   ⏭️  Pulando limpeza /usr/local/bin (requer sudo)"
        echo "   💡 Execute manualmente: sudo rm -f /usr/local/bin/clippit-*"
    fi
fi

# Instalar binários novos em ~/.local/bin (NÃO requer sudo!)
echo "📦 Instalando binários atualizados em ~/.local/bin..."
cp -f target/release/clippit-daemon ~/.local/bin/
cp -f target/release/clippit-popup ~/.local/bin/
cp -f target/release/clippit-dashboard ~/.local/bin/
cp -f target/release/clippit-tooltip ~/.local/bin/

# Dar permissões de execução
chmod +x ~/.local/bin/clippit-daemon
chmod +x ~/.local/bin/clippit-popup
chmod +x ~/.local/bin/clippit-dashboard
chmod +x ~/.local/bin/clippit-tooltip

# Instalar IBus Component (Autocomplete Global) - requer sudo
echo "⌨️  Instalando IBus Component (Autocomplete Global)..."
if [ -f "target/release/clippit-ibus" ]; then
    if sudo -n true 2>/dev/null; then
        # sudo disponível - instalar normalmente
        if ! sudo cp target/release/clippit-ibus /usr/local/bin/clippit-ibus 2>/dev/null; then
            echo "⚠️  Arquivo em uso, forçando atualização..."
            sudo fuser -k /usr/local/bin/clippit-ibus 2>/dev/null || true
            sleep 1
            sudo cp target/release/clippit-ibus /usr/local/bin/clippit-ibus
        fi
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
        echo "⏭️  Pulando instalação IBus (requer sudo)"
        echo "   💡 Para autocomplete global, execute:"
        echo "      sudo cp target/release/clippit-ibus /usr/local/bin/"
        echo "      sudo cp crates/clippit-ibus/data/clippit.xml /usr/share/ibus/component/"
    fi
else
    echo "⚠️  clippit-ibus não encontrado, pulando instalação do IBus"
fi

# Instalar ícone em múltiplos tamanhos (importante para Wayland/GNOME)
echo "🎨 Instalando ícone..."

# Verificar se o arquivo existe
if [ ! -f "assets/logo_clippit.png" ]; then
    echo "⚠️  Arquivo de ícone não encontrado!"
elif sudo -n true 2>/dev/null; then
    # sudo disponível - instalar em /usr/share
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
else
    # sem sudo - instalar em ~/.local
    mkdir -p ~/.local/share/icons/hicolor/256x256/apps
    cp -f assets/logo_clippit.png ~/.local/share/icons/hicolor/256x256/apps/clippit.png
    gtk-update-icon-cache -f ~/.local/share/icons/hicolor/ 2>/dev/null || true
    echo "✅ Ícone instalado em ~/.local/share/icons"
fi

# Instalar arquivo .desktop (importante para Wayland)
# O GNOME é iniciado com um PATH sem ~/.local/bin: com "Exec=clippit-dashboard"
# ele não acha o programa e esconde o app da busca. Por isso o caminho completo.
echo "🖥️  Instalando arquivo .desktop..."
DESKTOP_TMP=$(mktemp)
sed -e "s|^Exec=clippit-dashboard|Exec=$HOME/.local/bin/clippit-dashboard|" \
    -e "/^Exec=/a TryExec=$HOME/.local/bin/clippit-dashboard" \
    assets/clippit.desktop > "$DESKTOP_TMP"
if sudo -n true 2>/dev/null; then
    sudo mkdir -p /usr/share/applications
    sudo cp "$DESKTOP_TMP" /usr/share/applications/clippit.desktop
    sudo chmod 644 /usr/share/applications/clippit.desktop
    sudo update-desktop-database /usr/share/applications/ 2>/dev/null || true
    echo "✅ Arquivo .desktop instalado"
else
    mkdir -p ~/.local/share/applications
    cp -f "$DESKTOP_TMP" ~/.local/share/applications/clippit.desktop
    update-desktop-database ~/.local/share/applications/ 2>/dev/null || true
    echo "✅ Arquivo .desktop instalado em ~/.local/share/applications"
fi
rm -f "$DESKTOP_TMP"

# Popup residente: pré-carregar no login para o atalho abrir instantaneamente
echo "⚡ Configurando popup pré-carregado (autostart)..."
mkdir -p ~/.config/autostart
sed "s|^Exec=clippit-popup|Exec=$HOME/.local/bin/clippit-popup|" \
    assets/clippit-popup-autostart.desktop > ~/.config/autostart/clippit-popup.desktop
# Iniciar já agora em background (o anterior foi encerrado acima)
if [ -n "$WAYLAND_DISPLAY" ] || [ -n "$DISPLAY" ]; then
    setsid ~/.local/bin/clippit-popup --background >/dev/null 2>&1 < /dev/null &
    echo "✅ Popup residente iniciado em background"
fi

# Verificar se foram copiados
echo "✅ Verificando instalação..."
echo ""
echo "📅 Data dos binários:"
ls -lh /usr/local/bin/clippit-* --time-style=+"%Y-%m-%d %H:%M:%S" | awk '{print "   "$6, $7, $9}'
echo ""
echo "📌 Versão instalada:"
/usr/local/bin/clippit-daemon --version 2>/dev/null | head -2 | sed 's/^/   /'


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
ExecStart=%h/.local/bin/clippit-daemon
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
    
    if [ -n "$EXISTING_BINDING" ] && [ "$EXISTING_BINDING" != "@as []" ]; then
        echo "   ✅ Atalho já configurado: $EXISTING_BINDING"
        echo "   💡 Para alterar: clippit-dashboard → Hotkeys"
        echo "   ⏭️  Mantendo sua configuração (não será sobrescrita)"
        SKIP_HOTKEY_SETUP=true
        HOTKEY_CONFIGURED="true"
        return 0
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
        # Use full path to ensure GNOME can find the binary (GNOME doesn't use user's PATH)
        gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH command "$HOME/.local/bin/clippit-popup" 2>/dev/null
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
            echo "      Comando: clippit-popup"
            echo "      Atalho: $MODIFIER + $KEY"
        fi
    else
        echo ""
        echo "   📝 Para configurar manualmente depois:"
        echo "      Configurações → Teclado → Atalhos → Adicionar"
        echo "      Nome: Clippit - Show History"
        echo "      Comando: clippit-popup"
        echo "      Atalho: Escolha sua combinação"
        echo ""
        echo "   💡 Ou execute: ./scripts/setup-wayland-hotkey.sh"
    fi
    
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
fi

# Forçar atualização do cache de apps e ícones
echo ""
echo "🔄 Atualizando cache de aplicativos e ícones..."
update-desktop-database ~/.local/share/applications 2>/dev/null || true
gtk-update-icon-cache -f -t ~/.local/share/icons/hicolor 2>/dev/null || true
xdg-desktop-menu forceupdate 2>/dev/null || true

# Recarregar GNOME Shell overview (se disponível)
if command -v gdbus &> /dev/null; then
    gdbus call --session --dest org.gnome.Shell --object-path /org/gnome/Shell \
        --method org.gnome.Shell.Eval "Main.overview.hide(); Main.overview.show();" &>/dev/null || true
    echo "✅ Menu de aplicativos atualizado!"
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