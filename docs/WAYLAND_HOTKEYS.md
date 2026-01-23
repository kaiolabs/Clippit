# 🔑 Hotkeys Globais no Wayland - Guia Completo

## ⚠️ Problema: Global Hotkeys Não Funcionam no Wayland

### O Que Acontece?

Se você está rodando Wayland (ao invés de X11), o **registro interno de hotkeys do Clippit não funcionará**. Você configurou o atalho no dashboard do Clippit, mas ao pressionar a combinação de teclas, nada acontece.

### Por Que Isso Acontece?

O protocolo **Wayland bloqueia hotkeys globais** por design de segurança. Diferente do X11, aplicativos não podem mais interceptar teclas globalmente. Isso é uma **limitação do Wayland**, não um bug do Clippit.

**Bibliotecas afetadas:**
- `global-hotkey` (Rust) - Funciona apenas no X11
- `winit` DeviceEvents - Não emitidos no Wayland
- Qualquer solução de hotkey global tradicional

### Como Verificar Se Você Está no Wayland?

```bash
echo $XDG_SESSION_TYPE
# Se retornar "wayland", você precisa configurar manualmente
```

---

## ✅ Solução: Configurar Através do Sistema

No Wayland, **o sistema operacional gerencia os hotkeys globais**, não os aplicativos individuais.

### Opção 1: Script Automático (Recomendado) ⚡

O Clippit fornece um script que configura tudo automaticamente:

```bash
cd /caminho/para/clippit
./scripts/setup-wayland-hotkey.sh
```

**O que o script faz:**
1. Lê o atalho configurado no Clippit (`~/.config/clippit/config.toml`)
2. Converte para o formato do GNOME/Zorin
3. Registra o atalho usando `gsettings`
4. Configura o comando correto (`/usr/local/bin/clippit-popup`)

### Opção 2: Configuração Manual (GUI) 🖱️

**Para GNOME/Zorin/Ubuntu:**

1. Abra **Configurações** (Settings)
2. Vá em **Teclado** → **Atalhos do Teclado** (Keyboard → Shortcuts)
3. Role até o final e clique em **➕ Adicionar Atalho**
4. Preencha:
   - **Nome**: `Clippit - Show History`
   - **Comando**: `/usr/local/bin/clippit-popup`
   - **Atalho**: Clique no campo e pressione a combinação desejada
     - Exemplo: `Super + V`
     - Exemplo: `Ctrl + Alt + V`
     - Exemplo: `Ctrl + Numpad 1`

5. Clique em **Adicionar**

**Para KDE Plasma:**

1. Abra **Configurações do Sistema**
2. Vá em **Atalhos** → **Atalhos Personalizados**
3. Clique em **Editar** → **Novo** → **Atalho Global** → **Comando/URL**
4. Na aba **Gatilho**: Defina o atalho
5. Na aba **Ação**: Digite `/usr/local/bin/clippit-popup`

**Para Sway (Tiling WM):**

Adicione ao `~/.config/sway/config`:

```
bindsym $mod+v exec /usr/local/bin/clippit-popup
```

Depois recarregue: `swaymsg reload`

### Opção 3: Via Terminal (gsettings) 💻

Para GNOME/Zorin/Ubuntu, você pode configurar via terminal:

```bash
# Definir o caminho do atalho personalizado
NEW_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/"

# Configurar o atalho
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH name "Clippit - Show History"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH command "/usr/local/bin/clippit-popup"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH binding "<Super>v"

# Adicionar à lista de atalhos personalizados
CUSTOM_KEYS=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)

if [[ "$CUSTOM_KEYS" == "@as []" ]] || [[ "$CUSTOM_KEYS" == "[]" ]]; then
    NEW_LIST="['$NEW_PATH']"
else
    NEW_LIST=$(echo "$CUSTOM_KEYS" | sed "s/]$/, '$NEW_PATH']/")
fi

gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
```

**Sintaxe de atalhos do GNOME:**
- `<Super>v` = Super (tecla Windows) + V
- `<Primary>v` = Ctrl + V
- `<Alt>v` = Alt + V
- `<Primary><Alt>v` = Ctrl + Alt + V
- `<Super><Shift>v` = Super + Shift + V
- `<Primary>KP_1` = Ctrl + Numpad 1

---

## 🔍 Verificar Configuração

### Ver Se o Atalho Foi Registrado

```bash
# Listar todos atalhos personalizados
gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings

# Ver configuração específica do Clippit
gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ name
gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ command
gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding
```

### Testar o Comando Manualmente

Antes de configurar o atalho, teste se o comando funciona:

```bash
/usr/local/bin/clippit-popup
```

Se o popup abrir e fechar imediatamente, é esperado (ele precisa receber foco). Quando chamado via hotkey global, ele permanecerá aberto.

---

## 🚨 Problemas Comuns

### 1. Atalho Não Responde

**Causas:**
- Conflito com outro atalho existente
- Atalho não foi salvo corretamente
- Serviço de atalhos do sistema não está rodando

**Soluções:**

```bash
# Verificar conflitos
gsettings list-recursively org.gnome.settings-daemon.plugins.media-keys | grep -i "super+v"

# Verificar se o atalho está registrado
gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings

# Reiniciar serviço de atalhos (GNOME)
killall gnome-shell  # Irá reiniciar automaticamente
```

### 2. "Command Not Found" ao Pressionar Atalho

**Causa:** O caminho do executável está incorreto.

**Solução:**

```bash
# Verificar onde clippit-popup está instalado
which clippit-popup

# Atualizar o comando no atalho com o caminho correto
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ command "$(which clippit-popup)"
```

### 3. Atalho com Numpad Não Funciona

**Causa:** Num Lock pode estar desligado, ou sintaxe incorreta.

**Solução:**

```bash
# Para Numpad 1, use:
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding "<Primary>KP_1"

# Verifique se Num Lock está LIGADO
# Quando Num Lock está desligado, Numpad 1 vira "End"
```

### 4. Atalho Funciona Mas Popup Fecha Imediatamente

**Causa:** Lock file antigo ou múltiplas instâncias.

**Solução:**

```bash
# Limpar lock file
rm -f /tmp/clippit-popup.lock

# Matar instâncias antigas
pkill clippit-popup

# Testar novamente
```

---

## 🔄 Remover/Alterar Atalho

### Via GUI

1. Abra **Configurações** → **Teclado** → **Atalhos**
2. Procure por "Clippit - Show History"
3. Clique e pressione **Backspace** para remover
4. Ou clique e pressione nova combinação para alterar

### Via Terminal

```bash
# Remover o atalho
gsettings reset org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ name
gsettings reset org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ command
gsettings reset org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding

# Remover da lista
CUSTOM_KEYS=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)
NEW_LIST=$(echo "$CUSTOM_KEYS" | sed "s|, '/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/'||")
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
```

---

## 📚 Referências e Mais Informações

### Por Que Wayland Bloqueia Hotkeys Globais?

Wayland foi projetado com segurança em mente. Permitir que aplicativos capturem teclas globalmente cria riscos:
- **Keyloggers**: Aplicativos maliciosos poderiam capturar senhas
- **Conflitos**: Múltiplos apps tentando registrar a mesma tecla
- **Sandboxing**: Quebra o isolamento de aplicativos Flatpak/Snap

### Alternativas Técnicas (Para Desenvolvedores)

Se você é desenvolvedor e quer alternativas:

1. **Desktop Portals** (Futuro): XDG Desktop Portals podem adicionar suporte a hotkeys globais de forma segura no futuro
2. **Compositor-Specific**: Alguns compositores (como Sway) permitem configuração de hotkeys no próprio compositor
3. **D-Bus Activation**: Registrar o aplicativo como serviço D-Bus que pode ser ativado por hotkeys do sistema

### Links Úteis

- [Wayland Security - Why No Global Hotkeys](https://wayland.freedesktop.org/architecture.html)
- [GNOME Custom Keyboard Shortcuts](https://help.gnome.org/users/gnome-help/stable/keyboard-shortcuts-set.html)
- [Issue: Global Hotkeys RFC for Wayland](https://gitlab.freedesktop.org/wayland/wayland-protocols/-/merge_requests/22)

---

## 💡 Resumo TL;DR

**Problema:** Global hotkeys não funcionam no Wayland por limitação do protocolo.

**Solução:**
1. Execute: `./scripts/setup-wayland-hotkey.sh` **OU**
2. Configure manualmente: Configurações → Teclado → Atalhos → Adicionar
   - Nome: `Clippit - Show History`
   - Comando: `/usr/local/bin/clippit-popup`
   - Atalho: Sua combinação preferida

**Testar:** Pressione o atalho configurado e o popup deve abrir! 🎉

---

**Precisa de ajuda?** Consulte o [Troubleshooting](TROUBLESHOOTING.md) ou abra uma issue no GitHub.
