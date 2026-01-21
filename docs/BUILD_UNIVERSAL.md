# 📦 Build Universal - Clippit

## 🎯 O que é?

Um **pacote .deb UNIVERSAL** que funciona em **qualquer distribuição Linux**, independente da versão do glibc!

### ✅ Compatível com:
- Ubuntu 20.04, 22.04, 24.04+
- Debian 11, 12+
- Linux Mint 20, 21, 22+
- Fedora (qualquer versão)
- openSUSE (qualquer versão)
- Arch Linux
- **QUALQUER distribuição com kernel 3.2+**

---

## 🚀 Como Usar

### 1️⃣ **Executar o script:**

```bash
./scripts/build-deb-universal.sh
```

### 2️⃣ **Aguardar compilação:**

O script vai:
- ✅ Instalar target musl (se necessário)
- ✅ Instalar musl-tools (se necessário)
- ✅ Compilar com linkagem estática
- ✅ Criar pacote .deb universal
- ✅ Otimizar tamanho dos binários

**Tempo estimado:** 5-10 minutos (primeira vez)

### 3️⃣ **Pacote criado:**

```
clippit_1.0.0_universal_amd64.deb
```

---

## 📋 Pré-requisitos

### No seu PC (para compilar):

```bash
# Rust (se não tiver)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Dependências de build
sudo apt install -y \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    musl-tools
```

### No PC do cliente (para instalar):

**Apenas:**
```bash
sudo apt install xdotool xclip
```

**Pronto!** Não precisa de mais nada! 🎉

---

## 🔍 Como Funciona?

### **Compilação Estática com musl**

O script compila o Clippit usando **musl libc** ao invés de glibc:

```rust
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl
```

### **Vantagens:**

1. ✅ **Sem dependência de glibc** → funciona em qualquer sistema
2. ✅ **Binários menores** → strip remove símbolos desnecessários
3. ✅ **Distribuição simples** → um único .deb para todos
4. ✅ **Compatibilidade máxima** → suporta sistemas muito antigos

### **Limitações:**

- ⚠️ Ainda depende de GTK4/libadwaita em **runtime** (não compilado estaticamente)
- ⚠️ Mas GTK4 está disponível em praticamente todas as distribuições modernas

---

## 📊 Comparação de Builds

| Build Type | Compatibilidade | Tamanho | Dependências |
|------------|-----------------|---------|--------------|
| **Normal** | Mesma versão glibc | ~30MB | glibc + GTK4 + deps |
| **Docker (Ubuntu 20.04)** | glibc 2.31+ | ~30MB | glibc 2.31+ + GTK4 |
| **Universal (musl)** | QUALQUER | ~25MB | Apenas GTK4 runtime |

---

## 🛠️ Troubleshooting

### **Erro: "musl-gcc not found"**

```bash
sudo apt install musl-tools
```

### **Erro ao compilar GTK4**

Algumas bibliotecas podem não compilar estaticamente. Neste caso:

1. Use o **build Docker** (Ubuntu 20.04):
   ```bash
   ./scripts/build-deb-compat.sh
   ```

2. Ou distribua **código-fonte** para compilar no alvo

### **Build muito lento**

A primeira compilação com musl demora mais. Builds subsequentes são mais rápidos.

Para limpar e recomeçar:
```bash
cargo clean
./scripts/build-deb-universal.sh
```

---

## 🧪 Testar o Pacote

### **No seu sistema:**

```bash
# Instalar
sudo dpkg -i clippit_1.0.0_universal_amd64.deb
sudo apt install -f  # resolver dependências

# Testar
systemctl --user start clippit
systemctl --user status clippit
```

### **Verificar se é estático:**

```bash
ldd /usr/local/bin/clippit-daemon
```

**Resultado esperado:**
- Se totalmente estático: `not a dynamic executable`
- Se híbrido: apenas GTK4 e libs essenciais

---

## 📤 Distribuir

### **Para clientes:**

1. Envie apenas o arquivo:
   ```
   clippit_1.0.0_universal_amd64.deb
   ```

2. Instrução de instalação:
   ```bash
   sudo dpkg -i clippit_1.0.0_universal_amd64.deb
   sudo apt install -f
   systemctl --user enable --now clippit
   ```

3. **Pronto!** Funciona em qualquer distribuição! 🎉

---

## 💡 Dicas

### **Reduzir tamanho ainda mais:**

```bash
# Adicionar ao Cargo.toml de cada crate:
[profile.release]
opt-level = "z"  # Otimizar para tamanho
lto = true       # Link-time optimization
codegen-units = 1
strip = true     # Strip automático
```

### **Build mais rápido:**

```bash
# Usar compilação paralela
cargo build --release --target x86_64-unknown-linux-musl -j$(nproc)
```

---

## 🎯 Quando Usar Este Build?

✅ **Use este build se:**
- Você precisa distribuir para múltiplas distribuições
- Seus clientes têm sistemas diferentes (Ubuntu, Debian, Fedora, etc.)
- Você quer evitar problemas de compatibilidade de glibc
- Você quer a solução mais universal possível

❌ **Use outro método se:**
- Você controla o sistema alvo → compile localmente
- Você só tem um tipo de distribuição → use build Docker
- GTK4 estático é obrigatório → use AppImage (coming soon)

---

## 📚 Recursos

- [musl libc](https://www.musl-libc.org/)
- [Rust musl target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-linux-musl.html)
- [Static linking in Rust](https://doc.rust-lang.org/edition-guide/rust-2018/platform-and-target-support/musl-support-for-fully-static-binaries.html)

---

**✨ Build universal = Máxima compatibilidade!**
