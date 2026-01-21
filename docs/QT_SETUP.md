# Guia de Setup Qt6 para Clippit

## Requisitos para Interface Gráfica

O Clippit possui duas interfaces:
1. **CLI** (clippit-ui) - Terminal, funciona sem Qt
2. **GUI** (clippit-dashboard + clippit-popup) - Interface gráfica moderna, requer Qt6

## Instalação do Qt6

### Ubuntu/Debian/Zorin OS

```bash
sudo apt update
sudo apt install qt6-base-dev qt6-declarative-dev libqt6svg6-dev
```

### Fedora

```bash
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtsvg-devel
```

### Arch Linux

```bash
sudo pacman -S qt6-base qt6-declarative qt6-svg
```

## Compilar com Qt

Depois de instalar o Qt6:

```bash
cd /path/to/Clippit
cargo build --release
```

Isso irá compilar:
- `clippit-daemon` - Daemon (sem Qt)
- `clippit-ui` - CLI (sem Qt)
- `clippit-dashboard` - Config GUI ✨ (Qt)
- `clippit-popup` - Popup visual ✨ (Qt)

## Instalar

```bash
./scripts/install.sh
```

O script instalará todos os binários disponíveis e criará entradas no menu de aplicativos.

## Verificar Instalação

```bash
# Verificar binários
ls -la ~/.local/bin/clippit-*

# Dashboard (GUI de configuração)
clippit-dashboard

# Popup (quando pressiona Super+V)
# Será lançado automaticamente pelo daemon

# CLI (fallback, sempre funciona)
clippit-ui
```

## Solução de Problemas

### Qt6 não encontrado

**Erro:**
```
Could not find Qt6
```

**Solução:**
```bash
# Instalar Qt6
sudo apt install qt6-base-dev qt6-declarative-dev

# Verificar
qmake6 --version
```

### cxx-qt build falha

**Erro:**
```
error: failed to run custom build command for `clippit-dashboard`
```

**Solução:**
```bash
# Instalar dependências de build
sudo apt install build-essential cmake ninja-build

# Limpar e recompilar
cargo clean
cargo build --release
```

### Apenas CLI funciona

Se você não conseguir instalar Qt6, o Clippit ainda funciona com a interface CLI:

```bash
# Usar CLI
clippit-ui

# Ver histórico
systemctl --user status clippit
```

## Funcionalidades por Interface

### CLI (clippit-ui)
- ✅ Ver histórico
- ✅ Selecionar itens
- ❌ Configurar atalhos
- ❌ Personalizar temas

### GUI (clippit-dashboard + clippit-popup)
- ✅ Ver histórico (popup visual bonito)
- ✅ Selecionar itens
- ✅ Configurar atalhos
- ✅ Personalizar temas
- ✅ Gerenciar privacidade
- ✅ Preview em tempo real

## Desktop Entries

Após instalação com Qt, você terá no menu:

1. **Clippit Configurações** - Abre dashboard de config
2. **Clippit Histórico** - CLI (oculto, só via terminal)

O popup visual abre automaticamente com Super+V (ou seu atalho configurado).

## Performance

- **Dashboard**: ~30MB RAM, abre em ~0.5s
- **Popup**: ~20MB RAM, abre em ~0.1s
- **Daemon**: ~10MB RAM (sem Qt)

## Próximos Passos

Após instalar:

1. Abra **Clippit Configurações** no menu de aplicativos
2. Configure seu atalho preferido
3. Personalize o tema
4. Pressione o atalho para ver o popup visual
5. Desfrute! 🎉
