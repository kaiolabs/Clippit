# Clippit - Guia Rápido de Início

## ⚡ Instalação Rápida (5 minutos)

### 1. Instalar Dependências Obrigatórias

**⚠️ IMPORTANTE:** Instale primeiro as ferramentas de runtime (obrigatórias):

```bash
# Ubuntu/Debian/Zorin OS
sudo apt install xdotool xclip
```

Depois instale as dependências de compilação:

```bash
# Ubuntu/Debian/Zorin OS
./scripts/install-gtk-deps.sh
# Ou manualmente:
# sudo apt install libgtk-4-dev libadwaita-1-dev build-essential

# Fedora
sudo dnf install gtk4-devel libadwaita-devel gcc

# Arch Linux
sudo pacman -S gtk4 libadwaita base-devel
```

> **Nota:** `xdotool` é usado para capturar o foco da janela e simular paste. `xclip` é usado para operações de clipboard com imagens. **Sem eles, o Clippit não funcionará corretamente.**

### 2. Clonar e Compilar

```bash
cd ~/Downloads  # ou seu diretório preferido
git clone <repo-url> clippit
cd clippit
cargo build --release
```

### 3. Instalar

```bash
./scripts/install.sh
```

Responda **Y** para:
- Habilitar inicialização automática
- Criar entrada no menu de aplicativos

## 🎯 Uso Básico

### Copiar e Usar

1. **Copie qualquer texto** (Ctrl+C normal)
2. **Pressione Super+V** para ver histórico
3. **Digite o número** do item desejado
4. **Texto é copiado** automaticamente!

### Comandos Úteis

```bash
# Ver histórico manualmente
clippit-ui

# Status do daemon
systemctl --user status clippit

# Ver logs
journalctl --user -u clippit -f

# Reiniciar daemon
systemctl --user restart clippit

# Parar daemon
systemctl --user stop clippit
```

## 🔍 Verificação Rápida

### Está Funcionando?

```bash
# 1. Verificar daemon
pgrep clippit-daemon

# 2. Verificar socket
ls -la /tmp/clippit.sock

# 3. Verificar banco de dados
ls -la ~/.local/share/clippit/history.db

# 4. Testar
echo "Test Clippit" | xclip -selection clipboard
sleep 1
clippit-ui
```

## 🐛 Problemas Comuns

### Daemon não inicia

```bash
# Remover socket antigo
rm /tmp/clippit.sock

# Iniciar manualmente para ver erros
clippit-daemon
```

### Super+V não funciona

```bash
# Verificar conflito de teclas
gsettings list-recursively | grep -i "super+v"

# Testar UI manualmente
clippit-ui
```

### Não captura clipboard

```bash
# Verificar se é X11
echo $XDG_SESSION_TYPE  # Deve ser "x11"

# Verificar logs
journalctl --user -u clippit -n 20
```

## 🗑️ Desinstalar

```bash
cd clippit
./scripts/uninstall.sh
```

## 📚 Mais Informações

- **README.md** - Documentação completa
- **DEVELOPMENT.md** - Guia para desenvolvedores
- **PROJECT_STATUS.md** - Status do projeto

## 🆘 Ajuda

Se encontrar problemas:

1. Verifique os logs: `journalctl --user -u clippit -f`
2. Execute testes: `./examples/test_daemon.sh`
3. Abra uma issue no GitHub

---

**Divirta-se usando o Clippit! 🚀**
