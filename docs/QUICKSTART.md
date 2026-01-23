# 🚀 Quickstart - Clippit

Guia rápido para começar a usar o Clippit em menos de 5 minutos.

---

## 📦 Instalação Rápida

### 1. Instalar Dependências

```bash
sudo apt install libgtk-4-1 libadwaita-1-0
```

### 2. Baixar e Instalar

```bash
# Baixe o .deb da última release
sudo dpkg -i clippit_*.deb
sudo apt install -f
```

### 3. Iniciar o Daemon

```bash
systemctl --user enable --now clippit
```

### 4. Configurar Atalho Global (Wayland) ⚠️ IMPORTANTE

**No Wayland, hotkeys globais devem ser configurados pelo sistema operacional.**

Execute o script automático:

```bash
./scripts/setup-wayland-hotkey.sh
```

Ou configure manualmente:
1. Abra **Configurações** → **Teclado** → **Atalhos**
2. Clique em **+** para adicionar
3. Configure:
   - Nome: `Clippit - Show History`
   - Comando: `/usr/local/bin/clippit-popup`
   - Atalho: Pressione a combinação desejada (ex: `Super+V`)

---

## ✅ Verificar Instalação

### Verificar se daemon está rodando

```bash
systemctl --user status clippit
```

### Verificar se é Wayland

```bash
echo $XDG_SESSION_TYPE  # Deve mostrar "wayland"
```

---

## 🎯 Uso Básico

### Abrir Histórico

Pressione o **atalho que você configurou** (ex: `Super + V` ou `Ctrl + Numpad1`)

⚠️ **Lembre-se**: No Wayland, você precisa ter configurado o atalho nas Configurações do Sistema primeiro!

### Navegar

- `↑` `↓` - Navegar pelos itens
- `Enter` - Copiar item selecionado
- `Ctrl+V` - Colar manualmente
- `Delete` - Apagar item
- `Esc` - Fechar

### Testar Clipboard

```bash
# Copiar algo
echo "Teste Clippit" | wl-copy

# Verificar se foi capturado
# Pressione Super+V para ver o histórico
```

---

## ⚙️ Dashboard de Configurações

```bash
clippit-dashboard
```

No dashboard você pode:
- Ver estatísticas
- Limpar histórico
- Configurar captura de imagens
- Personalizar atalhos

---

## 🔧 Gerenciamento

### Ver Logs

```bash
journalctl --user -u clippit -f
```

### Reiniciar

```bash
systemctl --user restart clippit
```

### Parar

```bash
systemctl --user stop clippit
```

### Desativar Autostart

```bash
systemctl --user disable clippit
```

---

## 📂 Localização dos Arquivos

```
~/.local/share/clippit/
├── history.db          # Banco de dados SQLite
└── images/            # Imagens salvas
```

---

## ❓ Problemas Comuns

### Atalho não funciona

```bash
# Verificar se daemon está rodando
systemctl --user status clippit

# Ver logs
journalctl --user -u clippit -n 50
```

### Clipboard não captura

```bash
# Verificar se está no Wayland
echo $XDG_SESSION_TYPE

# Reiniciar daemon
systemctl --user restart clippit
```

---

## 🎉 Pronto!

Agora você pode:
1. Copiar qualquer coisa (Ctrl+C)
2. Pressionar `Super+V` para ver histórico
3. Selecionar item e pressionar `Enter`
4. Pressionar `Ctrl+V` para colar

---

**Veja mais em:** [README.md](../README.md) | [FEATURES.md](FEATURES.md)
