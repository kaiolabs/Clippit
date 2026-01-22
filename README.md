# 📋 Clippit - Gerenciador de Área de Transferência para Linux

<div align="center">

![Clippit Logo](assets/logo_clippit.png)

**Um gerenciador de área de transferência moderno, rápido e elegante para Linux (Wayland)**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/GTK-4.6+-green.svg)](https://www.gtk.org/)

[Instalação](#-instalação) • [Uso](#-uso) • [Compilar](#-compilar-do-código-fonte) • [Contribuir](#-contribuindo)

</div>

---

## ✨ **Features**

- 📋 **Captura Automática** - Monitora e salva tudo que você copia
- 🖼️ **Suporte a Imagens** - Salva prints e imagens copiadas
- 🔍 **Busca Inteligente** - Encontre rapidamente o que procura
- ⌨️ **Atalho Global** - Pressione `Super+V` para abrir instantaneamente
- 💾 **Histórico Persistente** - Seus dados salvos em SQLite
- 🎨 **Interface Moderna** - Design limpo com GTK4 e libadwaita
- ⚡ **Ultra Rápido** - Escrito em Rust, zero latência
- 🔒 **Baixo Consumo** - Menos de 20MB de RAM
- 🗑️ **Gerenciamento Fácil** - Delete itens individualmente ou limpe tudo
- ⚙️ **Configurável** - Dashboard intuitivo para ajustar preferências

---

## 🖼️ **Screenshots**

### Popup Principal (Super+V)
![Clippit Popup](docs/screenshot-popup.png)
*Interface rápida e elegante para acessar seu histórico*

### Dashboard de Configurações
![Clippit Dashboard](docs/screenshot-dashboard.png)
*Central de controle com todas as opções*

---

## 🚀 **Instalação**

### 📦 **Opção 1: Pacote .deb (Recomendado)**

**Para Ubuntu 22.04+, Debian 12+, Linux Mint 21+:**

1. Baixe o arquivo `.deb` da [última release](releases)
2. Instale:

```bash
sudo dpkg -i clippit_1.0.0_amd64.deb
sudo apt install -f
```

3. Inicie o serviço:

```bash
systemctl --user enable --now clippit
```

**Pronto!** Pressione `Super+V` para usar! 🎉

---

### 🔧 **Opção 2: Compilar do Código-Fonte**

**Quer compilar você mesmo? Veja o guia completo:**  
👉 [BUILD_FOR_USERS.md](BUILD_FOR_USERS.md)

**Resumo rápido:**

```bash
# 1. Instalar dependências
sudo apt update && sudo apt install -y \
    curl build-essential pkg-config \
    libgtk-4-dev libadwaita-1-dev libsqlite3-dev

# 2. Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 3. Compilar
cargo build --release

# 4. Criar pacote .deb
./scripts/build-deb-simple.sh

# 5. Instalar
sudo dpkg -i clippit_*.deb
```

---

## 📖 **Uso**

### ⌨️ **Atalhos de Teclado**

| Atalho | Ação |
|--------|------|
| `Super+V` | Abrir popup do histórico |
| `↑` `↓` | Navegar pelos itens |
| `Enter` | Copiar item selecionado |
| `Delete` | Apagar item selecionado |
| `Esc` | Fechar popup |
| `Ctrl+F` | Focar na busca |

### 🎛️ **Dashboard de Configurações**

Abra o dashboard para:
- 📊 Ver estatísticas de uso
- 🗑️ Limpar todo o histórico
- 📏 Configurar tamanho máximo de imagens
- 🖼️ Ativar/desativar captura de imagens
- 🎨 Personalizar aparência

Abra via menu de aplicativos ou:
```bash
clippit-dashboard
```

### 📂 **Localização dos Dados**

```
~/.local/share/clippit/
├── history.db          # Banco de dados SQLite
└── images/            # Imagens salvas
```

---

## 🛠️ **Gerenciamento do Daemon**

```bash
# Ver status
systemctl --user status clippit

# Parar
systemctl --user stop clippit

# Reiniciar
systemctl --user restart clippit

# Desativar autostart
systemctl --user disable clippit

# Ver logs em tempo real
journalctl --user -u clippit -f
```

---

## 🏗️ **Arquitetura**

```
clippit/
├── crates/
│   ├── clippit-core/       # 🧠 Lógica de negócio e histórico
│   ├── clippit-daemon/     # 👁️ Monitor de clipboard e hotkeys
│   ├── clippit-ipc/        # 📡 Comunicação inter-processos
│   ├── clippit-popup/      # 🎨 Interface popup (Super+V)
│   └── clippit-dashboard/  # ⚙️ Dashboard de configurações
├── assets/                 # 🖼️ Ícones e recursos
├── scripts/                # 🔧 Scripts de build e instalação
└── docs/                   # 📚 Documentação
```

### 🔄 **Fluxo de Funcionamento**

```
┌─────────────┐      ┌──────────────┐      ┌────────────┐
│  Clipboard  │ ───> │    Daemon    │ ───> │  Database  │
│  (Wayland)  │      │  (Monitor)   │      │  (SQLite)  │
└─────────────┘      └──────────────┘      └────────────┘
                            │
                            │ IPC
                            ▼
                     ┌──────────────┐
                     │    Popup     │ <─── Super+V
                     │  (GTK4 UI)   │
                     └──────────────┘
```

---

## 🔧 **Requisitos de Sistema**

### **Sistema Operacional:**
- ✅ Ubuntu 22.04+ (Jammy, Noble)
- ✅ Debian 12+ (Bookworm)
- ✅ Linux Mint 21+
- ✅ Pop!_OS 22.04+
- ✅ Zorin OS 17+
- ✅ Wayland (GNOME, KDE Plasma, Sway)

### **Dependências Runtime:**
- `GTK4 4.6+` - Interface gráfica
- `libadwaita 1.2+` - Componentes modernos

### **Hardware:**
- CPU: Qualquer processador x86_64
- RAM: ~20MB (daemon + UI)
- Disco: ~50MB (instalação)

---

## 🐛 **Troubleshooting**

<details>
<summary><b>❌ Daemon não inicia</b></summary>

```bash
# Verificar se há outra instância rodando
ps aux | grep clippit-daemon

# Matar processos antigos
pkill clippit-daemon

# Remover socket antigo
rm /tmp/clippit.sock

# Reiniciar
systemctl --user restart clippit
```
</details>

<details>
<summary><b>⌨️ Atalho Super+V não funciona</b></summary>

1. Verifique conflitos com outros atalhos:
   ```bash
   gsettings list-recursively | grep -i "super+v"
   ```

2. Verifique se o daemon está rodando:
   ```bash
   systemctl --user status clippit
   ```

3. Veja os logs para erros:
   ```bash
   journalctl --user -u clippit -n 50
   ```
</details>

<details>
<summary><b>📋 Clipboard não captura</b></summary>

1. Verifique se está usando Wayland:
   ```bash
   echo $XDG_SESSION_TYPE
   # Deve mostrar: wayland
   ```

2. Reinicie o daemon:
   ```bash
   systemctl --user restart clippit
   ```

3. Verifique os logs:
   ```bash
   journalctl --user -u clippit -n 50
   ```
</details>

<details>
<summary><b>🖼️ Imagens não aparecem</b></summary>

1. Verifique permissões da pasta de imagens:
   ```bash
   ls -la ~/.local/share/clippit/images/
   ```

2. Ative captura de imagens no dashboard:
   ```bash
   clippit-dashboard
   ```
</details>

<details>
<summary><b>💾 Banco de dados corrompido</b></summary>

```bash
# Fazer backup
cp ~/.local/share/clippit/history.db ~/.local/share/clippit/history.db.bak

# Limpar e recriar
rm ~/.local/share/clippit/history.db
systemctl --user restart clippit
```
</details>

---

## 🗺️ **Roadmap**

### ✅ **v1.0 - Lançamento Inicial**
- [x] Captura de texto
- [x] Captura de imagens
- [x] Histórico persistente
- [x] Atalho global Super+V
- [x] Interface GTK4 moderna
- [x] Dashboard de configurações
- [x] Busca no histórico
- [x] Pacote .deb

### 🚧 **v1.1 - Melhorias** (Em breve)
- [ ] Fixar itens favoritos
- [ ] Categorias/tags
- [ ] Estatísticas detalhadas
- [ ] Temas customizados
- [ ] Importar/exportar histórico

### 🔮 **v2.0 - Futuro**
- [x] Suporte a Wayland (concluído em v1.1)
- [ ] Sincronização entre máquinas
- [ ] Aplicativo mobile companion
- [ ] Plugins/extensões
- [ ] OCR em imagens
- [ ] Criptografia de dados sensíveis

---

## 🤝 **Contribuindo**

Contribuições são muito bem-vindas! 🎉

### **Como Contribuir:**

1. 🍴 Faça um Fork do projeto
2. 🌱 Crie uma branch para sua feature (`git checkout -b feature/MinhaFeature`)
3. ✍️ Commit suas mudanças (`git commit -m 'Adiciona MinhaFeature'`)
4. 📤 Push para a branch (`git push origin feature/MinhaFeature`)
5. 🔃 Abra um Pull Request

### **Encontrou um Bug?**

Abra uma [issue](issues) com:
- 📝 Descrição detalhada do problema
- 🖥️ Informações do sistema (Ubuntu version, GTK version)
- 📋 Logs relevantes (`journalctl --user -u clippit`)
- 🔄 Passos para reproduzir

---

## 📄 **Licença**

Este projeto está licenciado sob a **MIT License** - veja o arquivo [LICENSE](LICENSE) para detalhes.

---

## 🙏 **Agradecimentos**

- [GTK Project](https://www.gtk.org/) - Framework UI
- [libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/) - Componentes modernos
- [Rust Community](https://www.rust-lang.org/) - Linguagem incrível
- [Wayland](https://wayland.freedesktop.org/) - Protocolo de display moderno
- [arboard](https://github.com/1Password/arboard) - Clipboard cross-platform

---

## 📞 **Contato & Suporte**

- 🐛 **Issues:** [GitHub Issues](issues)
- 💬 **Discussões:** [GitHub Discussions](discussions)
- 📧 **Email:** clippit@example.com

---

<div align="center">

**Feito com ❤️ e ☕ em Rust**

Se este projeto te ajudou, considere dar uma ⭐!

[⬆ Voltar ao topo](#-clippit---gerenciador-de-área-de-transferência-para-linux)

</div>
