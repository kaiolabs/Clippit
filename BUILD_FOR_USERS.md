# 📦 Clippit - Como Compilar no Seu Sistema

Este guia ensina como compilar o Clippit no **seu próprio sistema Ubuntu/Debian**.

---

## 📋 **Requisitos**

- Ubuntu 22.04+ ou Debian 12+
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
    xdotool \
    xclip
```

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

# Ou usar o atalho
# Pressione Super+V para abrir o histórico
```

---

## ❓ **Problemas?**

### Erro: `GTK4 não encontrado`
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev
```

### Erro: `xdotool não encontrado`
```bash
sudo apt install xdotool xclip
```

### O `.deb` não foi criado
- Verifique se todas as dependências foram instaladas
- Execute novamente: `./scripts/build-deb-simple.sh`

---

## 📝 **Resumo**

```bash
# 1. Instalar dependências
sudo apt update && sudo apt install -y curl build-essential pkg-config libgtk-4-dev libadwaita-1-dev libsqlite3-dev xdotool xclip

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

---

**Pronto! O Clippit está instalado e funcionando! 🎉**

Pressione `Super+V` para abrir o histórico do clipboard.
