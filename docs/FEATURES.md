# ✨ Features do Clippit

Documentação completa de todas as funcionalidades do Clippit.

---

## 🎯 Funcionalidades Principais

### 📋 Histórico de Clipboard

- **Captura automática** de tudo que você copia
- **Armazenamento persistente** em SQLite
- **Texto e imagens** suportados
- **Busca em tempo real** no histórico
- **Navegação por teclado** (↑↓)

### 🖼️ Suporte a Imagens

- Captura imagens copiadas
- Thumbnails na interface
- Preview expandido
- Armazenamento eficiente (deduplica por hash)

### 🔍 Busca Inteligente

- Busca incremental ao digitar
- Filtro em tempo real
- Destaque de correspondências
- Busca case-insensitive

---

## ⌨️ Atalhos

### Global

- **`Super + V`** - Abre popup do histórico

### Dentro do Popup

- **`↑` `↓`** - Navegar pelos itens
- **`Enter`** - Copiar item selecionado para clipboard
- **`Delete`** - Apagar item do histórico
- **`Esc`** - Fechar popup
- **`Digite qualquer coisa`** - Buscar no histórico

---

## 🎨 Interface

### Popup

- **Interface moderna** com libadwaita
- **Tema automático** (light/dark)
- **Auto-fechamento inteligente** ao perder foco
- **Notificações do sistema** para feedback
- **Animações suaves**

### Dashboard

```bash
clippit-dashboard
```

- Estatísticas de uso
- Configurações de privacidade
- Personalização de atalhos
- Limpeza de histórico
- Temas e aparência

---

## 🔒 Privacidade

### Opções Configuráveis

- **Desativar captura de imagens** (apenas texto)
- **Limitar tamanho do histórico** (ex: últimos 100 itens)
- **Limpar histórico** (manual ou automático)
- **Exclusão de itens sensíveis** (Delete no item)

### Dados Locais

- Tudo fica em `~/.local/share/clippit/`
- Nenhum dado sai da sua máquina
- SQLite local sem conexão externa

---

## 🚀 Performance

### Clipboard Monitor

- Polling eficiente (80ms de intervalo)
- Detecção de duplicatas (evita spam)
- Consumo mínimo de CPU/RAM
- Hashing de imagens para deduplica

### Armazenamento

- **SQLite** para histórico de texto
- **Sistema de arquivos** para imagens
- Índices otimizados para busca rápida
- Compressão de thumbnails

---

## 🔌 Integração com Sistema

### Wayland

- **Clipboard nativo** via arboard (wl-clipboard-rs)
- **Global hotkeys** via desktop portals
- **Notificações do sistema** para feedback
- Compatível com GNOME, KDE, Sway, Hyprland

### Systemd

```bash
# Auto-start no login
systemctl --user enable clippit

# Ver status
systemctl --user status clippit

# Logs
journalctl --user -u clippit -f
```

---

## 🛠️ Configuração

### Arquivo de Configuração

`~/.config/clippit/config.toml`

```toml
[ui]
language = "pt"
theme = "dark"

[privacy]
enable_image_capture = true
max_history_size = 100

[hotkeys]
toggle_popup = "Super+V"
```

---

## 📊 Estatísticas (Dashboard)

- Total de itens salvos
- Tamanho do banco de dados
- Itens de texto vs imagens
- Uso de espaço em disco

---

## 🔄 Sincronização de Clipboard

### Comportamento

1. Você copia algo (Ctrl+C)
2. Clippit detecta mudança
3. Salva no histórico (SQLite)
4. Pressione `Super+V` para ver histórico
5. Selecione item e pressione `Enter`
6. Item é copiado para clipboard
7. **Notificação do sistema** confirma ação
8. Pressione `Ctrl+V` para colar

---

## 🎯 Casos de Uso

### Programação

- Gerenciar snippets de código
- Histórico de comandos copiados
- URLs e documentação

### Design

- Copiar múltiplas imagens
- Gerenciar screenshots
- Histórico de cores (hex codes)

### Produtividade

- Copiar textos longos
- Gerenciar múltiplos clipboards
- Buscar conteúdos copiados anteriormente

---

## 🧪 Recursos Avançados

### IPC (Inter-Process Communication)

- Comunicação daemon ↔ popup via Unix socket
- Protocolo JSON eficiente
- Lock files para evitar múltiplas instâncias

### Detecção de Duplicatas

- Texto: comparação direta
- Imagens: SHA-256 hash

### Auto-Close Inteligente

- Popup fecha ao perder foco (500ms de debounce)
- Previne fechamento acidental

---

## 📝 Limitações Conhecidas

### Wayland Security Model

- **Não há auto-paste** (limitação de segurança do Wayland)
- Usuário precisa pressionar `Ctrl+V` manualmente
- Notificação do sistema indica quando copiar

### Performance

- Imagens grandes (>10MB) podem ser lentas para preview
- Histórico muito grande (>1000 itens) pode impactar busca

---

## 🔮 Roadmap

Veja [ROADMAP.md](../ROADMAP.md) para funcionalidades planejadas.

---

**Problemas?** Veja [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
