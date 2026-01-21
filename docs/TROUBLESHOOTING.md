# Troubleshooting - Clippit

Guia de resolução de problemas comuns.

---

## 🔴 Erro: "Área de texto ocupada" ao instalar

**Sintoma:**
```bash
cp: não foi possível criar arquivo comum '/home/kaio/.local/bin/clippit-daemon': Área de texto ocupada
```

**Causa:** O daemon está rodando e o Linux não permite sobrescrever binários em execução.

**Solução Rápida:**

```bash
# Usar script de reinstalação segura
./scripts/reinstall.sh
```

**Solução Manual:**

```bash
# 1. Parar o daemon
systemctl --user stop clippit
# ou
pkill clippit-daemon

# 2. Aguardar 1 segundo
sleep 1

# 3. Compilar e instalar
cargo build --release
cp target/release/clippit-daemon ~/.local/bin/
cp target/release/clippit-ui ~/.local/bin/

# 4. Reiniciar
systemctl --user start clippit
```

---

## 🔴 Daemon não inicia

**Verificar status:**
```bash
systemctl --user status clippit
```

**Ver logs:**
```bash
journalctl --user -u clippit -f
```

**Soluções:**

### 1. Socket ocupado
```bash
rm /tmp/clippit.sock
systemctl --user restart clippit
```

### 2. Permissões incorretas
```bash
chmod +x ~/.local/bin/clippit-daemon
systemctl --user restart clippit
```

### 3. Arquivo de configuração inválido
```bash
# Verificar sintaxe
cat ~/.config/clippit/config.toml

# Resetar para padrão
rm ~/.config/clippit/config.toml
cp clippit.example.toml ~/.config/clippit/config.toml
systemctl --user restart clippit
```

### 4. Banco de dados corrompido
```bash
# Backup
mv ~/.local/share/clippit/history.db ~/.local/share/clippit/history.db.backup

# Reiniciar (novo DB será criado)
systemctl --user restart clippit
```

---

## 🔴 Atalho Super+V não funciona

### 1. Verificar conflitos com sistema

```bash
# GNOME
gsettings list-recursively | grep -i "super.*v"

# KDE
kreadconfig5 --group kglobalshortcutsrc
```

### 2. Testar atalho alternativo

Edite `~/.config/clippit/config.toml`:

```toml
[hotkeys]
show_history_modifier = "ctrl+shift"
show_history_key = "v"
```

Reinicie:
```bash
systemctl --user restart clippit
```

### 3. Verificar logs

```bash
journalctl --user -u clippit -f
# Pressione Super+V e veja se aparece algo
```

### 4. Atalhos sugeridos sem conflitos

- `Ctrl+Shift+V`
- `Alt+V`
- `Ctrl+` (backtick)
- `Super+C`

---

## 🔴 Clipboard não está sendo capturado

### 1. Verificar se é X11

```bash
echo $XDG_SESSION_TYPE
```

**Deve retornar:** `x11`

**Se retornar** `wayland`: O Clippit V1.0 ainda não suporta Wayland. Use X11 ou aguarde V2.0.

### 2. Verificar se daemon está rodando

```bash
ps aux | grep clippit-daemon
```

### 3. Testar captura manualmente

```bash
# Terminal 1 - ver logs
journalctl --user -u clippit -f

# Terminal 2 - copiar texto
echo "teste clippit" | xclip -selection clipboard

# Deve aparecer nos logs: "Clipboard changed, saving to history"
```

### 4. Verificar configuração

```toml
[features]
capture_text = true  # Deve estar true
```

---

## 🔴 UI não abre ao pressionar Super+V

### 1. Verificar se UI está no PATH

```bash
which clippit-ui
# Deve retornar: /home/SEU_USUARIO/.local/bin/clippit-ui
```

### 2. Adicionar ao PATH

Edite `~/.bashrc` ou `~/.zshrc`:

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Recarregue:
```bash
source ~/.bashrc  # ou ~/.zshrc
```

### 3. Testar UI manualmente

```bash
clippit-ui
```

Se funcionar manualmente mas não com Super+V, o problema é com o atalho.

---

## 🔴 Performance: Daemon usando muita CPU

### 1. Verificar intervalo de polling

Edite `~/.config/clippit/config.toml`:

```toml
[general]
poll_interval_ms = 500  # Aumentar de 200 para 500
```

### 2. Verificar processos duplicados

```bash
ps aux | grep clippit-daemon
```

Se houver múltiplos processos:
```bash
pkill clippit-daemon
systemctl --user restart clippit
```

### 3. Ver estatísticas

```bash
top -p $(pgrep clippit-daemon)
```

**Normal:** < 1% CPU em idle, < 50MB RAM

---

## 🔴 Performance: Banco de dados muito grande

### 1. Verificar tamanho

```bash
du -h ~/.local/share/clippit/history.db
```

### 2. Reduzir limite de histórico

Edite `~/.config/clippit/config.toml`:

```toml
[general]
max_history_items = 50  # Reduzir de 100 para 50
```

### 3. Limpar histórico antigo

```bash
# ATENÇÃO: Isso apaga todo o histórico!
rm ~/.local/share/clippit/history.db
systemctl --user restart clippit
```

---

## 🔴 Configuração não está sendo aplicada

### 1. Verificar localização do arquivo

```bash
ls -la ~/.config/clippit/config.toml
```

### 2. Verificar sintaxe TOML

```bash
# Instalar validador TOML (opcional)
pip install toml

# Validar
python3 -c "import toml; toml.load(open('$HOME/.config/clippit/config.toml'))"
```

### 3. Reiniciar daemon

```bash
systemctl --user restart clippit
```

### 4. Verificar logs

```bash
journalctl --user -u clippit | grep -i "config\|error"
```

---

## 🔴 Erro: "Failed to connect to daemon"

**Sintoma:** UI mostra "Daemon not running"

### 1. Verificar se daemon está rodando

```bash
systemctl --user status clippit
```

### 2. Iniciar daemon

```bash
systemctl --user start clippit
```

### 3. Verificar socket

```bash
ls -la /tmp/clippit.sock
```

Se não existir, daemon não está rodando corretamente.

### 4. Iniciar manualmente para debug

```bash
# Parar serviço
systemctl --user stop clippit

# Iniciar manualmente com logs
RUST_LOG=debug clippit-daemon
```

---

## 🔴 Histórico vazio após reinicialização

### 1. Verificar se banco existe

```bash
ls -la ~/.local/share/clippit/history.db
```

### 2. Verificar permissões

```bash
chmod 644 ~/.local/share/clippit/history.db
```

### 3. Verificar configuração

```toml
[privacy]
clear_on_exit = false  # Deve estar false
```

---

## 🔴 Aplicativo sensível não está sendo ignorado

### 1. Descobrir nome correto do aplicativo

```bash
xprop | grep WM_CLASS
# Clique na janela do aplicativo
```

Use o segundo valor (em minúsculas).

### 2. Adicionar à configuração

Edite `~/.config/clippit/config.toml`:

```toml
[privacy]
ignored_apps = [
    "keepassxc",
    "bitwarden",
    "nome-do-app",  # Adicione aqui
]
```

### 3. Reiniciar

```bash
systemctl --user restart clippit
```

---

## 🔴 Desinstalação completa

```bash
# Parar e desabilitar
systemctl --user disable --now clippit

# Remover binários
rm ~/.local/bin/clippit-daemon
rm ~/.local/bin/clippit-ui

# Remover serviço
rm ~/.config/systemd/user/clippit.service
systemctl --user daemon-reload

# Remover dados (opcional)
rm -rf ~/.local/share/clippit

# Remover configuração (opcional)
rm -rf ~/.config/clippit

# Remover socket
rm /tmp/clippit.sock
```

---

## 📊 Comandos de Diagnóstico

### Status geral do sistema

```bash
#!/bin/bash

echo "=== Clippit Diagnostics ==="
echo ""

echo "1. Daemon Status:"
systemctl --user is-active clippit.service && echo "✓ Running" || echo "✗ Not running"

echo ""
echo "2. Binaries:"
ls -lh ~/.local/bin/clippit-* 2>/dev/null || echo "✗ Not found"

echo ""
echo "3. Socket:"
ls -la /tmp/clippit.sock 2>/dev/null && echo "✓ Exists" || echo "✗ Not found"

echo ""
echo "4. Database:"
ls -lh ~/.local/share/clippit/history.db 2>/dev/null || echo "✗ Not found"

echo ""
echo "5. Config:"
ls -la ~/.config/clippit/config.toml 2>/dev/null && echo "✓ Exists" || echo "✗ Not found"

echo ""
echo "6. Session Type:"
echo "  XDG_SESSION_TYPE=$XDG_SESSION_TYPE"

echo ""
echo "7. Recent Logs:"
journalctl --user -u clippit -n 5 --no-pager 2>/dev/null || echo "✗ No logs"

echo ""
echo "=== End Diagnostics ==="
```

Salve como `diagnose.sh` e execute:
```bash
chmod +x diagnose.sh
./diagnose.sh
```

---

## 🆘 Ainda com problemas?

Se nenhuma solução acima funcionou:

1. **Coletar informações:**
```bash
# Executar diagnóstico
./diagnose.sh > clippit-debug.txt

# Adicionar logs
journalctl --user -u clippit -n 100 >> clippit-debug.txt
```

2. **Abrir issue no GitHub** com:
   - Arquivo `clippit-debug.txt`
   - Descrição do problema
   - Passos para reproduzir
   - Sistema operacional e versão

3. **Modo debug manual:**
```bash
# Parar serviço
systemctl --user stop clippit

# Executar manualmente com logs detalhados
RUST_LOG=trace clippit-daemon 2>&1 | tee daemon-debug.log
```

---

## ✅ Reinstalação Limpa

Se tudo falhar, reinstalação limpa:

```bash
# 1. Desinstalar completamente
./scripts/uninstall.sh

# 2. Limpar tudo
rm -rf ~/.local/share/clippit
rm -rf ~/.config/clippit
rm /tmp/clippit.sock

# 3. Reinstalar
./scripts/install.sh
```

---

**Documentação relacionada:**
- [README.md](README.md) - Guia geral
- [CONFIGURATION.md](CONFIGURATION.md) - Configuração
- [DEVELOPMENT.md](DEVELOPMENT.md) - Desenvolvimento
