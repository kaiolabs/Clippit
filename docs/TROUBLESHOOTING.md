# 🔧 Troubleshooting - Clippit

Guia de solução de problemas comuns do Clippit.

---

## 🔍 Diagnóstico Inicial

### 1. Verificar Sistema

```bash
# Verificar se é Wayland
echo $XDG_SESSION_TYPE  # Deve retornar "wayland"

# Verificar compositor
echo $XDG_CURRENT_DESKTOP  # GNOME, KDE, Sway, etc
```

⚠️ **X11 não é mais suportado** - migre para Wayland

### 2. Verificar Instalação

```bash
# Verificar binários
which clippit-daemon
which clippit-popup
which clippit-dashboard

# Verificar serviço
systemctl --user status clippit
```

### 3. Ver Logs

```bash
# Logs recentes
journalctl --user -u clippit -n 50

# Logs em tempo real
journalctl --user -u clippit -f
```

---

## 🐛 Problemas Comuns

### 1. Daemon Não Inicia

#### Sintomas
```bash
$ systemctl --user status clippit
● clippit.service - Clippit Clipboard Manager
   Loaded: loaded
   Active: failed
```

#### Soluções

**A. Remover socket antigo**
```bash
rm /tmp/clippit.sock
systemctl --user restart clippit
```

**B. Verificar permissões**
```bash
ls -la ~/.local/share/clippit/
# Deve estar com seu usuário

# Se necessário
chmod -R 755 ~/.local/share/clippit/
```

**C. Recriar diretórios**
```bash
mkdir -p ~/.local/share/clippit/images
systemctl --user restart clippit
```

---

### 2. Atalho Não Funciona ⚠️ WAYLAND

#### ⚠️ IMPORTANTE: Limitação do Wayland

**Global hotkeys não funcionam diretamente no Wayland!** 

O protocolo Wayland bloqueia hotkeys globais por questões de segurança. Aplicativos não podem registrar atalhos globais como no X11. Você **DEVE** configurar o atalho através das configurações do sistema.

#### Sintomas
- Pressionar o atalho configurado não abre o popup
- Nenhum evento de "Hotkey pressed" nos logs

#### ✅ Solução Definitiva: Configurar no Sistema

**Opção 1: Script Automático (Recomendado)**

```bash
# Executar script de configuração
./scripts/setup-wayland-hotkey.sh
```

O script:
- Lê o atalho configurado no Clippit
- Registra automaticamente nas configurações do GNOME/Zorin
- Configura o comando correto

**Opção 2: Configurar Manualmente**

1. Abra **Configurações** (Settings)
2. Vá em **Teclado** → **Atalhos do Teclado** (Keyboard → Shortcuts)
3. Role até o final e clique em **+** para adicionar
4. Configure:
   - **Nome**: `Clippit - Show History`
   - **Comando**: `/usr/local/bin/clippit-popup`
   - **Atalho**: Pressione a combinação desejada (ex: `Super+V` ou `Ctrl+Numpad1`)

**Opção 3: Via gsettings (Terminal)**

```bash
# Criar atalho personalizado
NEW_PATH="/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/"

gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH name "Clippit - Show History"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH command "/usr/local/bin/clippit-popup"
gsettings set org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:$NEW_PATH binding "<Super>v"

# Adicionar à lista
CUSTOM_KEYS=$(gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings)
NEW_LIST=$(echo "$CUSTOM_KEYS" | sed "s/]$/, '$NEW_PATH']/")
gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "$NEW_LIST"
```

#### Verificar Configuração

```bash
# Ver se o atalho foi registrado
gsettings get org.gnome.settings-daemon.plugins.media-keys custom-keybindings

# Ver detalhes do atalho Clippit
gsettings get org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/ binding
```

#### Outras Soluções (se ainda não funcionar)

**A. Verificar se daemon está rodando**
```bash
systemctl --user status clippit
```

**B. Verificar conflitos de atalho**
```bash
# GNOME - Listar todos atalhos com a mesma combinação
gsettings list-recursively org.gnome.settings-daemon.plugins.media-keys | grep -i "super+v"

# Se houver conflito, desabilite o outro atalho nas Configurações
```

**C. Testar popup manualmente**
```bash
# Se funcionar manualmente, confirma que é só o hotkey
/usr/local/bin/clippit-popup
```

**D. Ver logs de hotkey (para debug)**
```bash
# Nota: O daemon ainda registra hotkey internamente, mas não funciona no Wayland
journalctl --user -u clippit -f | grep -i hotkey
```

**E. Verificar sessão Wayland**
```bash
echo $XDG_SESSION_TYPE  # Deve retornar "wayland"

# Se retornar "x11", hotkeys globais funcionariam normalmente
```

---

### 3. Clipboard Não Captura

#### Sintomas
- Copiar algo (Ctrl+C) não aparece no histórico

#### Soluções

**A. Verificar se daemon está rodando**
```bash
systemctl --user status clippit
```

**B. Testar clipboard manualmente**
```bash
# Copiar algo
echo "teste clippit" | wl-copy

# Ver se capturou
journalctl --user -u clippit -n 20 | grep -i "clipboard"
```

**C. Reiniciar daemon**
```bash
systemctl --user restart clippit
```

**D. Verificar permissões Wayland**
```bash
# Alguns compositors precisam de configuração extra
# Exemplo (Sway):
# Adicione ao ~/.config/sway/config:
# exec systemctl --user import-environment WAYLAND_DISPLAY XDG_CURRENT_DESKTOP
```

---

### 4. Imagens Não Aparecem

#### Sintomas
- Copiar imagem não salva no histórico

#### Soluções

**A. Verificar se captura de imagens está habilitada**
```bash
# Abrir dashboard
clippit-dashboard

# Ir em "Privacidade" e habilitar "Capturar imagens"
```

**B. Verificar espaço em disco**
```bash
df -h ~/.local/share/clippit/
```

**C. Verificar permissões do diretório de imagens**
```bash
ls -la ~/.local/share/clippit/images/
chmod -R 755 ~/.local/share/clippit/images/
```

---

### 5. Popup Não Abre ou Fecha Imediatamente

#### Sintomas
- Popup abre e fecha instantaneamente
- Popup não aparece

#### Soluções

**A. Verificar lock file**
```bash
# Remover lock antigo
rm /tmp/clippit-popup.lock
```

**B. Matar processos pendentes**
```bash
pkill clippit-popup
```

**C. Testar manualmente**
```bash
# Ver erros diretamente
/usr/bin/clippit-popup
```

**D. Verificar GTK/Wayland**
```bash
# Instalar dependências GTK4
sudo apt install libgtk-4-1 libadwaita-1-0
```

---

### 6. Notificações Não Aparecem

#### Sintomas
- Ao clicar em "Copiar", não aparece notificação

#### Soluções

**A. Verificar daemon de notificações**
```bash
# GNOME
ps aux | grep notification-daemon

# KDE
ps aux | grep plasma-notify
```

**B. Testar notificações manualmente**
```bash
notify-send "Teste" "Se você vê isso, notificações funcionam"
```

**C. Reinstalar notify-daemon**
```bash
# GNOME
sudo apt install --reinstall gnome-shell

# KDE
sudo apt install --reinstall plasma-workspace
```

---

### 7. Erro de Banco de Dados

#### Sintomas
```
Error: database is locked
Error: unable to open database file
```

#### Soluções

**A. Fechar todas instâncias**
```bash
systemctl --user stop clippit
pkill clippit
```

**B. Remover locks**
```bash
rm ~/.local/share/clippit/history.db-shm
rm ~/.local/share/clippit/history.db-wal
```

**C. Recriar banco (⚠️ perde histórico)**
```bash
mv ~/.local/share/clippit/history.db ~/.local/share/clippit/history.db.bak
systemctl --user start clippit
```

---

### 8. Alta CPU/Memória

#### Sintomas
- Daemon consome muita CPU ou RAM

#### Soluções

**A. Verificar tamanho do histórico**
```bash
# Ver tamanho do banco
du -h ~/.local/share/clippit/history.db

# Limpar histórico antigo (dashboard)
clippit-dashboard
```

**B. Desabilitar captura de imagens**
```bash
# No dashboard: Privacidade → Desabilitar "Capturar imagens"
```

**C. Limitar histórico**
```bash
# Editar ~/.config/clippit/config.toml
[privacy]
max_history_size = 50  # Reduzir de 100 para 50
```

---

## 🔬 Diagnóstico Avançado

### Habilitar Debug Logs

```bash
# Parar daemon
systemctl --user stop clippit

# Rodar manualmente com debug
RUST_LOG=debug /usr/bin/clippit-daemon

# Em outro terminal, testar
echo "teste" | wl-copy
```

### Verificar IPC

```bash
# Ver socket
ls -la /tmp/clippit.sock

# Testar comunicação (requer socat)
echo '{"command":"Ping"}' | socat - UNIX-CONNECT:/tmp/clippit.sock
```

### Strace (Debugging Avançado)

```bash
# Rastrear chamadas do sistema
strace -e trace=open,read,write /usr/bin/clippit-daemon
```

---

## 🆘 Reinstalação Limpa

Se nada funcionar, faça uma reinstalação limpa:

```bash
# 1. Parar daemon
systemctl --user stop clippit
systemctl --user disable clippit

# 2. Remover dados
rm -rf ~/.local/share/clippit
rm -rf ~/.config/clippit
rm /tmp/clippit.sock
rm /tmp/clippit-popup.lock

# 3. Desinstalar
sudo apt remove --purge clippit

# 4. Reinstalar
sudo dpkg -i clippit_*.deb
sudo apt install -f

# 5. Iniciar
systemctl --user enable --now clippit
```

---

## 📝 Reportar Bug

Se o problema persiste, reporte com:

```bash
# Coletar informações
echo "=== SYSTEM INFO ===" > clippit-debug.txt
uname -a >> clippit-debug.txt
echo $XDG_SESSION_TYPE >> clippit-debug.txt
echo $XDG_CURRENT_DESKTOP >> clippit-debug.txt

echo "=== CLIPPIT VERSION ===" >> clippit-debug.txt
clippit-daemon --version >> clippit-debug.txt

echo "=== SERVICE STATUS ===" >> clippit-debug.txt
systemctl --user status clippit >> clippit-debug.txt

echo "=== LOGS ===" >> clippit-debug.txt
journalctl --user -u clippit -n 100 >> clippit-debug.txt

# Envie clippit-debug.txt no issue
```

---

**Ainda com problemas?** Abra um [issue no GitHub](https://github.com/seu-usuario/clippit/issues)
