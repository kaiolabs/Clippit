# 📦 Clippit - Como Compilar no Seu Sistema

Este guia ensina como compilar o Clippit no **seu próprio sistema Ubuntu/Debian** com **suporte nativo ao Wayland**.

---

## 📋 **Requisitos**

- Ubuntu 22.04+ ou Debian 12+
- Wayland (suportado nativamente no GNOME 42+)
- Conexão com internet

---

## 🚀 **Instalação - Apenas 2 Comandos**

### **1. Instalar dependências:**

```bash
sudo apt update && sudo apt install -y \
    curl \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    libdbus-1-dev \
    libnotify-bin \
    xdotool \
    yad \
    ibus
```

**Nota:** `xdotool`, `yad` e `ibus` são necessários para o **autocomplete global**.

### **2. Instalar Rust:**

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
```

---

## 🔨 **Compilar e Instalar**

### **1. Baixar o código:**

Extraia o arquivo `.zip` ou `.tar.gz` que você recebeu e entre na pasta:

```bash
cd clippit
```

### **2. Compilar e criar pacote .deb:**

```bash
./scripts/build-deb-simple.sh
```

**Aguarde ~5-10 minutos** enquanto compila.

### **3. Instalar:**

```bash
sudo dpkg -i clippit_*.deb
sudo apt install -f
```

---

## ✅ **Iniciar o Clippit**

```bash
# Ativar serviço
systemctl --user enable --now clippit

# Testar o atalho
# Pressione Ctrl+Numpad1 para abrir o histórico (padrão)
# Ou configure outro atalho com: clippit-dashboard
```

### **🎯 Recursos Disponíveis**

#### **1. Histórico de Clipboard**
- Pressione o atalho configurado (padrão: `Ctrl+Numpad1`)
- Navegue com setas ou digite para pesquisar
- Clique ou pressione Enter para copiar

#### **2. Autocomplete Global do Sistema 🚀 NOVO!**
O Clippit agora oferece **autocomplete global** baseado no seu histórico, funcionando em **qualquer aplicativo**!

**Como funciona:**
1. Digite qualquer palavra em qualquer aplicativo (gedit, navegador, terminal, etc.)
2. Um popup "fantasma" aparece com sugestões do seu histórico
3. Pressione **Tab** para aceitar a sugestão → texto completo é digitado automaticamente!
4. Continue digitando normalmente (o popup não rouba o foco)

**Exemplos:**
- Digite `"cód"` → sugere `"código"`, `"códigos"`, etc.
- Digite `"dese"` → sugere `"desenvolvimento"`, `"desempenho"`, etc.

**Configuração:**
```bash
clippit-dashboard
# Vá na aba "Autocompletar"
# Configure: mínimo de caracteres, atraso, apps ignorados, etc.
```

---

## 🔄 **Atualizar o Clippit (Para Desenvolvedores)**

Se você está desenvolvendo e precisa testar mudanças rapidamente, use o script de atualização:

```bash
# Compilar e atualizar automaticamente
./update-clippit.sh
```

**O que o script faz:**
- ✅ Compila em modo release
- ✅ Para o daemon em execução
- ✅ Remove binários antigos
- ✅ Instala novos binários
- ✅ Instala ícone e arquivo .desktop
- ✅ Recarrega systemd
- ✅ Reinicia o daemon
- ✅ Mostra versão instalada

**Após atualizar:**
- O daemon reinicia automaticamente
- Use o atalho para testar o popup
- Se os ícones não aparecerem, faça logout/login

---

## ❓ **Problemas?**

### Erro: `GTK4 não encontrado`
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

### O `.deb` não foi criado
- Verifique se todas as dependências foram instaladas
- Execute novamente: `./scripts/build-deb-simple.sh`

### Ícones não aparecem no GNOME
```bash
# Atualizar caches
sudo gtk-update-icon-cache -f /usr/share/icons/hicolor/
sudo update-desktop-database /usr/share/applications/

# Reiniciar indexador do GNOME
tracker3 reset -r
tracker3 daemon -s

# Se ainda não funcionar, faça logout/login
```

### Notificações não aparecem
```bash
# Instalar libnotify (necessário para notificações do sistema)
sudo apt install libnotify-bin
```

### Atalho não funciona
- Verifique se há conflito com atalhos do sistema
- Configure outro atalho usando `clippit-dashboard`
- No Wayland, alguns atalhos podem precisar de permissão via portal

### Autocomplete não aparece
```bash
# Verificar se xdotool e yad estão instalados
sudo apt install xdotool yad

# Verificar se o recurso está ativado
clippit-dashboard  # Aba "Autocompletar" → ativar

# Ver logs para diagnóstico
journalctl --user -u clippit -f
```

### Popup do autocomplete rouba o foco
- O popup deve aparecer como "fantasma" (overlay)
- Certifique-se de que `yad` está atualizado: `sudo apt upgrade yad`
- Alternativa: desative notificações visuais e use apenas Tab

### Autocomplete não injeta texto
```bash
# Verificar se xdotool funciona
xdotool type "teste"

# Se não funcionar, pode ser limitação do Wayland
# Algumas apps Wayland-native podem bloquear injeção de texto
# Funciona melhor em apps X11/XWayland
```

---

## 📝 **Resumo - Instalação Completa**

```bash
# 1. Instalar dependências
sudo apt update && sudo apt install -y \
    curl build-essential pkg-config \
    libgtk-4-dev libadwaita-1-dev \
    libsqlite3-dev libdbus-1-dev \
    libnotify-bin xdotool yad ibus

# 2. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 3. Compilar
cd clippit
./scripts/build-deb-simple.sh

# 4. Instalar
sudo dpkg -i clippit_*.deb
sudo apt install -f

# 5. Iniciar
systemctl --user enable --now clippit
```

## 📝 **Resumo - Atualização Rápida (Dev)**

```bash
# Para desenvolvedores que já têm tudo instalado:
cd clippit
./update-clippit.sh
```

---

**Pronto! O Clippit está instalado e funcionando! 🎉**

Pressione `Ctrl+Numpad1` (ou o atalho configurado) para abrir o histórico do clipboard.

## 🌊 **Sobre o Wayland**

O Clippit agora é **nativo do Wayland**, o que significa:
- ✅ Mais seguro e moderno
- ✅ Melhor integração com GNOME
- ✅ Funciona nativamente sem X11
- ⚠️ Não tem auto-paste (limitação de segurança do Wayland)
- 💡 Use `Ctrl+V` para colar após selecionar um item

### **🎯 Autocomplete Global no Wayland**

O **autocomplete global** funciona tanto em **X11** quanto em **Wayland**, mas com algumas diferenças:

#### **✅ Em X11/XWayland:**
- Injeção de texto funciona perfeitamente
- Popup posicionado precisamente no cursor
- Funciona em 100% dos aplicativos

#### **⚠️ Em Wayland puro:**
- Injeção de texto pode não funcionar em apps Wayland-native (limitação de segurança)
- Funciona bem em apps XWayland (maioria dos apps)
- Popup pode não ser posicionado exatamente no cursor
- **Solução:** Use apps via XWayland ou aguarde suporte nativo do Wayland

**Apps testados que funcionam:**
- ✅ gedit, Firefox, Chrome, VS Code, Terminal GNOME
- ✅ LibreOffice, Thunderbird, Discord
- ⚠️ GNOME Text Editor (Wayland-native) - limitado

**Dica:** Para melhor compatibilidade, force apps em modo XWayland:
```bash
# Exemplo: forçar gedit em XWayland
GDK_BACKEND=x11 gedit
```
