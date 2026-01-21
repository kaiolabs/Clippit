# Instalação das Dependências GTK4

Para compilar a interface gráfica do Clippit, você precisa instalar as bibliotecas de desenvolvimento do GTK4 e libadwaita.

## Ubuntu/Debian/Zorin OS

```bash
sudo apt update
sudo apt install -y libgtk-4-dev libadwaita-1-dev libgraphene-1.0-dev build-essential pkg-config
```

## Fedora

```bash
sudo dnf install gtk4-devel libadwaita-devel graphene-devel
```

## Arch Linux

```bash
sudo pacman -S gtk4 libadwaita graphene
```

## Verificar Instalação

Após instalar, verifique se o pkg-config encontra as bibliotecas:

```bash
pkg-config --modversion gtk4
pkg-config --modversion libadwaita-1
pkg-config --modversion graphene-1.0
```

## Compilar Clippit

Depois de instalar as dependências:

```bash
cargo build --release
./scripts/install.sh
```

## Troubleshooting

Se ainda houver erros, certifique-se de que tem as seguintes variáveis de ambiente:

```bash
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH
```

Ou, se estiver usando 32-bit:

```bash
export PKG_CONFIG_PATH=/usr/lib/i386-linux-gnu/pkgconfig:$PKG_CONFIG_PATH
```
