# 🏗️ Build Universal - Clippit

Guia para criar um pacote `.deb` universal que funcione em múltiplas distribuições Ubuntu/Debian.

---

## 🎯 Objetivo

Criar um **único** pacote `.deb` que funcione em:
- Ubuntu 22.04+ (Jammy, Noble)
- Debian 12+ (Bookworm)
- Linux Mint 21+
- Pop!_OS 22.04+
- Outros derivados

---

## 📋 Estratégia

### 1. Compilação Estática

Compilar com o máximo de libs estáticas possível para reduzir dependências runtime.

### 2. GLIBC Compatibility

Compilar em sistema com GLIBC mais antiga (ex: Ubuntu 22.04) para compatibilidade retroativa.

### 3. Dynamic Libs Essenciais

Apenas libs essenciais como runtime deps:
- `libgtk-4-1`
- `libadwaida-1-0`
- `libsqlite3-0`

---

## 🚀 Build Process

### Método 1: Script Automático

```bash
./scripts/build-deb-universal.sh
```

O script:
1. Compila em release com otimizações
2. Strip de símbolos de debug
3. Cria estrutura .deb
4. Gera arquivo de controle com deps mínimas
5. Empacota o .deb

### Método 2: Docker (Isolado)

```bash
# Build da imagem
docker build -f Dockerfile.clippit -t clippit-builder .

# Executar build
docker run --rm -v $(pwd):/workspace clippit-builder

# O .deb estará em /tmp/clippit-deb-build/
```

---

## 🔧 Detalhes Técnicos

### Compilação

```bash
# Release build com otimizações
RUSTFLAGS="-C target-cpu=x86-64 -C link-arg=-Wl,-z,relro,-z,now" \
    cargo build --release --target x86_64-unknown-linux-gnu
```

**Flags:**
- `target-cpu=x86-64`: Compatibilidade máxima (não usa instruções AVX2, etc)
- `-Wl,-z,relro,-z,now`: Hardening de segurança

### Strip

```bash
strip --strip-unneeded target/release/clippit-daemon
strip --strip-unneeded target/release/clippit-popup
strip --strip-unneeded target/release/clippit-dashboard
```

Remove símbolos de debug → reduz tamanho de ~50MB para ~10MB

---

## 📦 Estrutura do Pacote

```
clippit_1.0.0_amd64.deb
└── data.tar.gz
    ├── usr/
    │   └── bin/
    │       ├── clippit-daemon
    │       ├── clippit-popup
    │       └── clippit-dashboard
    ├── usr/
    │   └── share/
    │       ├── applications/
    │       │   └── clippit.desktop
    │       └── icons/
    │           └── hicolor/
    │               └── 256x256/
    │                   └── apps/
    │                       └── clippit.png
    └── lib/
        └── systemd/
            └── user/
                └── clippit.service
```

### DEBIAN/control

```
Package: clippit
Version: 1.0.0
Architecture: amd64
Maintainer: Your Name <email@example.com>
Depends: libgtk-4-1, libadwaita-1-0
Description: Clippit Clipboard Manager for Wayland
 Modern clipboard manager for Linux with Wayland support.
```

**Notas:**
- Minimal deps (apenas runtime essenciais)
- Wayland-native via arboard
- No deps de X11

---

## ✅ Validação

### Testar em Múltiplas Distros

```bash
# Ubuntu 22.04 (Jammy)
lxc launch ubuntu:22.04 test-jammy
lxc file push clippit_*.deb test-jammy/tmp/
lxc exec test-jammy -- dpkg -i /tmp/clippit_*.deb

# Ubuntu 24.04 (Noble)
lxc launch ubuntu:24.04 test-noble
lxc file push clippit_*.deb test-noble/tmp/
lxc exec test-noble -- dpkg -i /tmp/clippit_*.deb

# Debian 12 (Bookworm)
lxc launch images:debian/12 test-bookworm
lxc file push clippit_*.deb test-bookworm/tmp/
lxc exec test-bookworm -- dpkg -i /tmp/clippit_*.deb
```

### Verificar Dependências

```bash
# Ver deps declaradas
dpkg-deb -I clippit_*.deb | grep Depends

# Ver libs dinâmicas linkadas
ldd /usr/bin/clippit-daemon
ldd /usr/bin/clippit-popup
```

---

## 🐛 Troubleshooting

### Erro: GLIBC version mismatch

**Causa:** Compilado em sistema com GLIBC mais nova

**Solução:** Compilar em Ubuntu 22.04 ou mais antiga

```bash
# Verificar GLIBC do binário
ldd --version
strings /usr/bin/clippit-daemon | grep GLIBC
```

### Erro: Missing GTK symbols

**Causa:** Linkado contra GTK4 muito nova

**Solução:** Compilar em sistema com GTK4 4.6+ (Ubuntu 22.04)

### Pacote muito grande

**Causa:** Símbolos de debug não foram removidos

**Solução:**
```bash
strip --strip-unneeded target/release/clippit-*
```

---

## 📊 Tamanhos Esperados

- **Com debug symbols**: ~50MB
- **Após strip**: ~10MB
- **.deb final**: ~3-4MB (comprimido)

---

## 🔒 Checksums

Gerar checksums para release:

```bash
# SHA256
sha256sum clippit_*.deb > clippit_1.0.0_amd64.deb.sha256

# MD5
md5sum clippit_*.deb > clippit_1.0.0_amd64.deb.md5
```

---

## 📝 Release Checklist

- [ ] Compilar em Ubuntu 22.04 (base)
- [ ] Strip symbols
- [ ] Gerar .deb
- [ ] Testar em Ubuntu 22.04
- [ ] Testar em Ubuntu 24.04
- [ ] Testar em Debian 12
- [ ] Gerar checksums
- [ ] Upload para GitHub Releases

---

## 🚀 CI/CD (GitHub Actions)

`.github/workflows/build.yml`:

```yaml
name: Build .deb

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v3
      - name: Install deps
        run: sudo apt update && sudo apt install -y libgtk-4-dev libadwaita-1-dev libsqlite3-dev
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build .deb
        run: ./scripts/build-deb-universal.sh
      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: clippit-deb
          path: /tmp/clippit-deb-build/*.deb
```

---

**Problemas?** Veja [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
