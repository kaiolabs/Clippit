#!/bin/bash

# Clippit - Setup Wayland Global Hotkey
# Este script configura um atalho global para o Clippit no Wayland
# usando as configurações do GNOME/Zorin

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔑 Configurando Atalho Global do Clippit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Load config to get hotkey
CONFIG_FILE="$HOME/.config/clippit/config.toml"

if [ -f "$CONFIG_FILE" ]; then
    MODIFIER=$(grep "show_history_modifier" "$CONFIG_FILE" | cut -d'"' -f2)
    KEY=$(grep "show_history_key" "$CONFIG_FILE" | head -n1 | cut -d'"' -f2)
    echo "📋 Atalho configurado no Clippit: $MODIFIER + $KEY"
else
    MODIFIER="super"
    KEY="v"
    echo "📋 Usando atalho padrão: $MODIFIER + $KEY"
fi

echo ""
echo "⚠️  IMPORTANTE: Global hotkeys não funcionam no Wayland"
echo "   O Wayland bloqueia hotkeys globais por segurança."
echo "   Precisamos configurar através do sistema."
echo ""

# Convert clippit key format to GNOME format
convert_key() {
    local mod=$1
    local key=$2
    
    # Convert modifier
    case $mod in
        "super"|"meta"|"win") mod_gnome="<Super>" ;;
        "ctrl"|"control") mod_gnome="<Primary>" ;;
        "alt") mod_gnome="<Alt>" ;;
        "shift") mod_gnome="<Shift>" ;;
        *) mod_gnome="<Super>" ;;
    esac
    
    # Convert key
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

GNOME_HOTKEY=$(convert_key "$MODIFIER" "$KEY")

echo "🔄 Configurando atalho no GNOME/Zorin..."
echo "   Atalho GNOME: $GNOME_HOTKEY"
echo ""

# Check if running GNOME/Zorin
if command -v gsettings &> /dev/null; then
    # Find next available custom keybinding slot
    CUSTOM_KEYS=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)
    
    # Create new custom keybinding
    NEW_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/"
    
    # Set the custom keybinding
    gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH name "Clippit - Show History"
    gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH command "/usr/local/bin/clippit-popup"
    gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH binding "$GNOME_HOTKEY"
    
    # Add to list of custom keybindings
    if [[ "$CUSTOM_KEYS" == "@as []" ]] || [[ "$CUSTOM_KEYS" == "[]" ]]; then
        NEW_LIST="['$NEW_PATH']"
    else
        # Remove the closing bracket, add new path, close again
        NEW_LIST=$(echo "$CUSTOM_KEYS" | sed "s/]$/, '$NEW_PATH']/")
    fi
    
    gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
    
    echo "✅ Atalho configurado com sucesso!"
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎉 Configuração Completa!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "🔑 Atalho registrado: $MODIFIER + $KEY"
    echo "📍 Comando: /usr/local/bin/clippit-popup"
    echo ""
    echo "🧪 Teste agora pressionando: $MODIFIER + $KEY"
    echo ""
    echo "💡 Para alterar o atalho:"
    echo "   1. Abra Configurações → Teclado → Atalhos"
    echo "   2. Procure por 'Clippit - Show History'"
    echo "   3. Clique e pressione o novo atalho desejado"
    echo ""
else
    echo "❌ gsettings não encontrado"
    echo ""
    echo "📝 Configure manualmente:"
    echo "   1. Abra Configurações do Sistema"
    echo "   2. Vá em Teclado → Atalhos do Teclado"
    echo "   3. Adicione um novo atalho personalizado:"
    echo "      Nome: Clippit - Show History"
    echo "      Comando: /usr/local/bin/clippit-popup"
    echo "      Atalho: $MODIFIER + $KEY (ou o que preferir)"
    echo ""
fi
