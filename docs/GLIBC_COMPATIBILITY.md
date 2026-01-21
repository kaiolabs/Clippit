# 🔧 Guia de Compatibilidade - glibc

## ❌ Erro: `GLIBC_2.39 not found`

Esse erro acontece quando você tenta executar um binário compilado em um sistema **mais novo** em um sistema **mais antigo**.

### 📊 Verificar versão do glibc no sistema:

```bash
ldd --version
```

**Exemplo de saída:**
```
ldd (Ubuntu GLIBC 2.31-0ubuntu9.16) 2.31
```

---

## 🎯 Soluções

### **Solução 1: Compilar Localmente** ⭐ (Recomendado)

O melhor método é compilar o Clippit **no próprio sistema** onde será usado.

#### Pré-requisitos:
```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Instalar dependências de desenvolvimento
sudo apt update
sudo apt install -y \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    xdotool \
    xclip
```

#### Compilar:
```bash
cd /caminho/do/codigo/clippit
cargo build --release
```

#### Instalar:
```bash
sudo cp target/release/clippit-daemon /usr/local/bin/
sudo cp target/release/clippit-popup /usr/local/bin/
sudo cp target/release/clippit-dashboard /usr/local/bin/
sudo chmod +x /usr/local/bin/clippit-*

# Copiar assets
sudo mkdir -p /usr/share/icons/hicolor/256x256/apps
sudo cp assets/logo_clippit.png /usr/share/icons/hicolor/256x256/apps/clippit.png
sudo cp assets/clippit.desktop /usr/share/applications/

# Criar serviço systemd
mkdir -p ~/.config/systemd/user/
cat > ~/.config/systemd/user/clippit.service << 'EOF'
[Unit]
Description=Clippit Clipboard Manager
After=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/local/bin/clippit-daemon
Restart=on-failure
RestartSec=5
Environment="DISPLAY=:0"
Environment="XAUTHORITY=%h/.Xauthority"

[Install]
WantedBy=default.target
EOF

# Habilitar e iniciar
systemctl --user daemon-reload
systemctl --user enable --now clippit
```

---

### **Solução 2: Build Compatível com Docker** 🐳

Se você tem Docker instalado, pode compilar para sistemas mais antigos:

```bash
# Executar o script de build compatível
./scripts/build-deb-compat.sh
```

Isso cria um `.deb` que funciona em:
- ✅ Ubuntu 20.04+
- ✅ Debian 11+
- ✅ Linux Mint 20+
- ✅ Qualquer sistema com glibc 2.31+

---

### **Solução 3: Compilação Estática** 📦 (Avançado)

Para criar um binário que funciona em **qualquer** sistema Linux:

```bash
# Instalar target musl
rustup target add x86_64-unknown-linux-musl

# Instalar musl-tools
sudo apt install musl-tools

# Compilar estaticamente
cargo build --release --target x86_64-unknown-linux-musl
```

⚠️ **Limitação**: Algumas dependências GTK podem não funcionar com musl.

---

## 📋 Tabela de Compatibilidade

| Sistema | glibc | Compatível com build atual? |
|---------|-------|------------------------------|
| Ubuntu 24.04 | 2.39 | ✅ Sim |
| Ubuntu 22.04 | 2.35 | ❌ Não - precisa recompilar |
| Ubuntu 20.04 | 2.31 | ❌ Não - precisa recompilar |
| Debian 12 | 2.36 | ❌ Não - precisa recompilar |
| Debian 11 | 2.31 | ❌ Não - precisa recompilar |

---

## 🎯 Recomendação para Distribuição

### **Para desenvolvedores:**

1. **Compile no sistema alvo** ou no sistema mais antigo que você quer suportar
2. **Use Docker** para criar builds compatíveis (Ubuntu 20.04)
3. **Distribua múltiplas versões**:
   - `clippit_1.0.0_ubuntu24.04_amd64.deb` (glibc 2.39)
   - `clippit_1.0.0_ubuntu20.04_amd64.deb` (glibc 2.31)

### **Para usuários finais:**

**Opção A - Instalar .deb pré-compilado:**
- Baixe a versão compatível com seu sistema

**Opção B - Compilar localmente:**
- Mais trabalhoso, mas **sempre funciona**
- Garante compatibilidade total

---

## 🔍 Diagnóstico Rápido

```bash
# 1. Verificar glibc do sistema
ldd --version

# 2. Verificar glibc necessária pelo binário
strings /usr/local/bin/clippit-daemon | grep GLIBC

# 3. Comparar versões
# Se o binário pede glibc MAIOR que a do sistema = incompatível
```

---

## 💡 Dica Pro

Para evitar esse problema no futuro, sempre compile em **sistemas mais antigos** ou use **Docker** para garantir compatibilidade máxima.

**Regra de ouro:** Compile no sistema **mais antigo** que você quer suportar!
