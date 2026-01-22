# 📦 Instalação via Pacote .deb

Guia completo para instalar o Clippit usando o pacote `.deb` pré-compilado.

---

## 📋 Requisitos

### Sistema Operacional

- ✅ Ubuntu 22.04+ (Jammy, Noble)
- ✅ Debian 12+ (Bookworm)
- ✅ Linux Mint 21+
- ✅ Pop!_OS 22.04+
- ✅ Zorin OS 17+

### Display Server

- ✅ **Wayland** (nativo)
- ⚠️ X11 não é mais suportado (use Wayland)

### Dependências Runtime

O pacote `.deb` já inclui ou declara as dependências:
- `libgtk-4-1` - Interface gráfica
- `libadwaita-1-0` - Componentes modernos

---

## 🚀 Instalação

### 1. Baixar o Pacote

Baixe o arquivo `.deb` da [última release](https://github.com/seu-usuario/clippit/releases):

```bash
# Exemplo
wget https://github.com/seu-usuario/clippit/releases/download/v1.0.0/clippit_1.0.0_amd64.deb
```

### 2. Instalar

```bash
sudo dpkg -i clippit_1.0.0_amd64.deb
```

### 3. Resolver Dependências (se necessário)

Se houver dependências faltando:

```bash
sudo apt install -f
```

### 4. Iniciar o Daemon

```bash
systemctl --user enable --now clippit
```

---

## ✅ Verificação

### Verificar se está instalado

```bash
which clippit-daemon
which clippit-popup
which clippit-dashboard
```

### Verificar se daemon está rodando

```bash
systemctl --user status clippit
```

### Testar atalho

Pressione `Super + V` - o popup deve aparecer

---

## 🎯 Uso

### Atalho Global

- **`Super + V`** - Abre o histórico do clipboard

### Dashboard

```bash
clippit-dashboard
```

Ou busque por "Clippit" no menu de aplicativos.

---

## 🔧 Gerenciamento

### Ver Logs

```bash
journalctl --user -u clippit -f
```

### Reiniciar Daemon

```bash
systemctl --user restart clippit
```

### Parar Daemon

```bash
systemctl --user stop clippit
```

### Desinstalar

```bash
sudo apt remove clippit
```

### Remover Dados

```bash
rm -rf ~/.local/share/clippit
```

---

## 📂 Arquivos Instalados

```
/usr/bin/
├── clippit-daemon      # Daemon principal
├── clippit-popup       # Popup do histórico
└── clippit-dashboard   # Dashboard de configurações

~/.local/share/clippit/
├── history.db          # Banco de dados
└── images/            # Imagens salvas

~/.config/systemd/user/
└── clippit.service     # Serviço systemd
```

---

## 🐛 Troubleshooting

### Erro: "dpkg: error processing"

```bash
sudo apt install -f
```

### Daemon não inicia

```bash
# Ver erros
journalctl --user -u clippit -n 50

# Remover socket antigo
rm /tmp/clippit.sock

# Reiniciar
systemctl --user restart clippit
```

### Atalho não funciona

1. Verificar se daemon está rodando:
```bash
systemctl --user status clippit
```

2. Verificar conflitos de atalho:
```bash
gsettings list-recursively | grep -i "super+v"
```

### Clipboard não captura

1. Verificar se está no Wayland:
```bash
echo $XDG_SESSION_TYPE  # Deve mostrar "wayland"
```

2. Reiniciar daemon:
```bash
systemctl --user restart clippit
```

---

## 🔄 Atualização

Para atualizar para uma nova versão:

```bash
# Parar daemon
systemctl --user stop clippit

# Instalar nova versão
sudo dpkg -i clippit_NEW_VERSION_amd64.deb

# Reiniciar daemon
systemctl --user start clippit
```

---

## 📝 Notas

- O Clippit usa **arboard** para clipboard (Wayland-nativo)
- Notificações do sistema são usadas para feedback
- Auto-paste não está disponível no Wayland por limitações de segurança
- Use `Ctrl+V` manualmente após selecionar um item

---

**Problemas?** Veja [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
