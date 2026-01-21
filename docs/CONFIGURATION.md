# Guia de Configuração - Clippit

## 📁 Localização do Arquivo de Configuração

```
~/.config/clippit/config.toml
```

## 🚀 Criação Automática

Na primeira execução, o Clippit cria automaticamente um arquivo de configuração com valores padrão.

Para criar/resetar manualmente:

```bash
cp clippit.example.toml ~/.config/clippit/config.toml
```

---

## ⚙️ Configurações Disponíveis

### [general] - Configurações Gerais

```toml
[general]
max_history_items = 100        # Máximo de itens no histórico
poll_interval_ms = 200         # Intervalo de polling (ms)
max_text_size = 10485760      # Tamanho máximo de texto (bytes)
max_image_size = 52428800     # Tamanho máximo de imagem (bytes)
```

**Recomendações:**
- `max_history_items`: 50-200 para uso normal
- `poll_interval_ms`: 200-500ms (menor = mais responsivo, maior CPU)

---

### [hotkeys] - Atalhos de Teclado

```toml
[hotkeys]
show_history_modifier = "super"  # super, ctrl, alt, shift
show_history_key = "v"

# Atalho alternativo (opcional)
show_history_alt_modifier = "ctrl+shift"
show_history_alt_key = "v"
```

**Modificadores disponíveis:**
- `super` - Tecla Windows/Super
- `ctrl` - Control
- `alt` - Alt
- `shift` - Shift
- Combinações: `"ctrl+shift"`, `"ctrl+alt"`, etc.

**Teclas disponíveis:**
- Letras: `a-z`
- Números: `0-9`
- Funções: `f1-f12`
- Especiais: `space`, `tab`, `escape`, etc.

**Exemplos:**

```toml
# Ctrl+Shift+V (evita conflito com Super+V do sistema)
show_history_modifier = "ctrl+shift"
show_history_key = "v"

# Alt+C
show_history_modifier = "alt"
show_history_key = "c"

# Ctrl+` (backtick, estilo terminal)
show_history_modifier = "ctrl"
show_history_key = "grave"
```

---

### [ui] - Interface do Usuário

```toml
[ui]
theme = "dark"              # "dark" ou "light"
font_family = "Nunito"
font_size = 14

[ui.window]
width = 600                 # Largura em pixels
max_height = 400           # Altura máxima
position = "center"         # center, cursor, top-right, bottom-right
opacity = 0.95             # 0.0 (transparente) a 1.0 (opaco)
```

**Temas:**
- `dark` - Tema escuro (padrão)
- `light` - Tema claro

**Posições da janela:**
- `center` - Centro da tela
- `cursor` - Próximo ao cursor do mouse
- `top-right` - Canto superior direito
- `bottom-right` - Canto inferior direito

---

### [ui.colors] - Personalização de Cores

#### Tema Escuro

```toml
[ui.colors.dark]
background = "#1e1e1e"
foreground = "#ffffff"
selection = "#264f78"
border = "#454545"
```

#### Tema Claro

```toml
[ui.colors.light]
background = "#ffffff"
foreground = "#000000"
selection = "#0078d4"
border = "#cccccc"
```

**Formato:** Cores hexadecimais (`#RRGGBB`)

---

### [features] - Funcionalidades

```toml
[features]
capture_text = true         # Capturar texto
capture_images = true       # Capturar imagens
capture_files = false       # Capturar arquivos (V2.0)
sync_enabled = false        # Sincronização cloud (V2.0)
```

---

### [privacy] - Privacidade e Segurança

```toml
[privacy]
ignore_sensitive_apps = true     # Ignorar apps sensíveis
ignored_apps = [                 # Lista de apps a ignorar
    "keepassxc",
    "bitwarden",
    "1password",
]
clear_on_exit = false            # Limpar histórico ao sair
```

**Como descobrir o nome do aplicativo:**

```bash
# No terminal
xprop | grep WM_CLASS
# Clique na janela do aplicativo
```

Adicione o segundo valor (em minúsculas) à lista `ignored_apps`.

---

### [advanced] - Avançado

```toml
[advanced]
log_level = "info"                           # error, warn, info, debug, trace
database_path = "/custom/path/history.db"    # Opcional
ipc_socket = "/tmp/clippit-custom.sock"      # Opcional
```

**Níveis de log:**
- `error` - Apenas erros críticos
- `warn` - Avisos e erros
- `info` - Informações gerais (padrão)
- `debug` - Informações de debug
- `trace` - Tudo (muito verboso)

---

## 🎨 Temas Pré-configurados

### Nord Theme

```toml
[ui.colors.dark]
background = "#2e3440"
foreground = "#d8dee9"
selection = "#5e81ac"
border = "#3b4252"
```

### Dracula Theme

```toml
[ui.colors.dark]
background = "#282a36"
foreground = "#f8f8f2"
selection = "#6272a4"
border = "#44475a"
```

### Gruvbox Theme

```toml
[ui.colors.dark]
background = "#282828"
foreground = "#ebdbb2"
selection = "#458588"
border = "#3c3836"
```

### Solarized Dark

```toml
[ui.colors.dark]
background = "#002b36"
foreground = "#839496"
selection = "#268bd2"
border = "#073642"
```

---

## 🔄 Aplicar Alterações

Após editar o arquivo de configuração:

```bash
# Reiniciar o daemon
systemctl --user restart clippit

# Ou manualmente
pkill clippit-daemon
clippit-daemon
```

---

## 🐛 Problemas Comuns

### Configuração não está sendo aplicada

```bash
# Verificar sintaxe do arquivo
cat ~/.config/clippit/config.toml

# Resetar para padrão
rm ~/.config/clippit/config.toml
clippit-daemon  # Cria novo arquivo
```

### Atalho não funciona

1. Verifique conflitos com sistema:
```bash
gsettings list-recursively | grep -i "super+v"
```

2. Teste outro atalho no config.toml

3. Veja logs:
```bash
journalctl --user -u clippit -f
```

---

## 📝 Exemplo Completo

Arquivo de configuração personalizado:

```toml
[general]
max_history_items = 200
poll_interval_ms = 250

[hotkeys]
show_history_modifier = "ctrl+shift"
show_history_key = "v"

[ui]
theme = "dark"
font_family = "Fira Code"
font_size = 13

[ui.colors.dark]
background = "#1e1e1e"
foreground = "#d4d4d4"
selection = "#264f78"
border = "#454545"

[ui.window]
width = 700
max_height = 500
position = "cursor"
opacity = 0.98

[features]
capture_text = true
capture_images = true

[privacy]
ignore_sensitive_apps = true
ignored_apps = ["keepassxc", "bitwarden"]
clear_on_exit = false

[advanced]
log_level = "info"
```

---

## 🎯 Dashboard de Configuração (V1.2)

Em desenvolvimento! Uma interface gráfica para configurar o Clippit sem editar arquivos TOML.

**Features planejadas:**
- ✨ Editor visual de atalhos
- 🎨 Seletor de temas com preview
- 🔐 Gerenciamento de apps ignorados
- 📊 Estatísticas de uso
- 🔄 Reset para padrões com um clique

Para acompanhar o desenvolvimento: veja `ROADMAP.md`
