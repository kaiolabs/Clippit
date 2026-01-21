# Clippit - Guia de Instalação (Pacote .deb)

## 🚀 Instalação Rápida

### 1. Baixe o pacote

Baixe o arquivo `clippit_1.0.0_amd64.deb` fornecido.

### 2. Instale com um único comando

```bash
sudo dpkg -i clippit_1.0.0_amd64.deb
```

### 3. Inicie o Clippit

```bash
systemctl --user enable --now clippit
```

Ou simplesmente **reinicie sua sessão** para iniciar automaticamente!

---

## ✅ O que é instalado automaticamente

O pacote `.deb` instala e configura:

- ✅ **Binários do Clippit** (`/usr/local/bin/`)
  - `clippit-daemon` - Serviço em background
  - `clippit-dashboard` - Interface de configuração
  - `clippit-popup` - Popup de histórico

- ✅ **Dependências necessárias**
  - `xdotool` - Captura de foco e simulação de paste
  - `xclip` - Operações de clipboard com imagens
  - `libgtk-4-1` - Interface GTK4
  - `libadwaita-1-0` - Componentes visuais modernos

- ✅ **Integração com o sistema**
  - Ícone no menu de aplicativos
  - Serviço systemd para auto-start
  - Atalho global `Ctrl+;`

---

## 🎯 Como usar

### Copiar e Colar do Histórico

1. **Copie qualquer texto ou imagem** (Ctrl+C normal)
2. **Pressione `Ctrl+;`** para ver o histórico
3. **Navegue com ↑↓** e **pressione Enter** para colar
4. **Digite para buscar** no histórico

### Configurar o Clippit

Abra o dashboard de configurações:

```bash
clippit-dashboard
```

Ou procure por "Clippit" no menu de aplicativos.

---

## 🔧 Comandos Úteis

### Ver status do serviço
```bash
systemctl --user status clippit
```

### Reiniciar o serviço
```bash
systemctl --user restart clippit
```

### Ver logs
```bash
journalctl --user -u clippit -f
```

### Desinstalar
```bash
sudo dpkg -r clippit
```

---

## 📋 Requisitos do Sistema

- **Sistema Operacional:** Ubuntu 20.04+, Debian 11+, Zorin OS 16+, ou derivados
- **Arquitetura:** amd64 (64-bit)
- **Display Server:** X11 (Wayland não suportado ainda)
- **Memória:** ~10MB RAM
- **Espaço em disco:** ~30MB

---

## 🐛 Solução de Problemas

### O atalho não funciona

Verifique se o daemon está rodando:
```bash
systemctl --user status clippit
```

### Não cola no aplicativo correto

Verifique se xdotool está instalado:
```bash
which xdotool
```

### Imagens não são copiadas

Verifique se xclip está instalado:
```bash
which xclip
```

---

## 🆘 Suporte

- **Logs:** `journalctl --user -u clippit -f`
- **Configuração:** `~/.config/clippit/config.toml`
- **Histórico:** `~/.local/share/clippit/history.db`

---

## 🎉 Pronto!

O Clippit está instalado e pronto para uso. Aproveite seu novo gerenciador de clipboard!

**Dica:** Pressione `Ctrl+;` a qualquer momento para acessar seu histórico de clipboard! 🚀
