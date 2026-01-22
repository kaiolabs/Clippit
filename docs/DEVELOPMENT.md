# 🛠️ Development Guide - Clippit

Guia completo para desenvolvedores que querem contribuir com o Clippit.

---

## 🏗️ Arquitetura

### Visão Geral

```
┌─────────────────┐
│  clippit-daemon │ ◄─── Systemd user service
│                 │
│  ┌───────────┐  │
│  │  Monitor  │  │ ◄─── Wayland Clipboard (arboard)
│  └───────────┘  │
│                 │
│  ┌───────────┐  │
│  │  Hotkey   │  │ ◄─── Desktop Portals
│  └───────────┘  │
│                 │
│  ┌───────────┐  │
│  │ IPC Server│  │ ◄─── Unix Socket (/tmp/clippit.sock)
│  └───────────┘  │
└─────────────────┘
         ▲
         │ IPC (JSON)
         ▼
┌─────────────────┐
│  clippit-popup  │ ◄─── GTK4 + libadwaita
└─────────────────┘

┌─────────────────┐
│clippit-dashboard│ ◄─── Qt6 (QML)
└─────────────────┘
```

### Componentes

#### 1. **clippit-daemon** (Rust)

Daemon principal que roda em background.

**Responsabilidades:**
- `monitor.rs`: Monitoramento do clipboard Wayland (polling a cada 80ms)
- `hotkey.rs`: Gerenciamento de hotkeys globais via desktop portals
- `main.rs`: IPC server (Unix socket), orquestração

**Fluxo:**
```
[Wayland Clipboard] ← [Clipboard Monitor] (arboard polling)
                    ↓
              [HistoryManager] → SQLite + filesystem
                    ↑
              [IPC Server] ←→ [Popup/Dashboard]
```

#### 2. **clippit-popup** (Rust + GTK4)

Interface de popup do histórico.

**Responsabilidades:**
- `views/`: Componentes GTK4 (window, list_item, buttons)
- `controllers/`: Lógica (keyboard, clipboard)
- `models/`: Estado (entry_map)

**Fluxo:**
```
[Usuário seleciona] → [IPC] → [Daemon] → [Wayland Clipboard (arboard)]
                                        ↓
                              [System Notification]
```

#### 3. **clippit-core** (Rust)

Biblioteca compartilhada.

**Módulos:**
- `config.rs`: Configuração (TOML)
- `history.rs`: HistoryManager (SQLite)
- `types.rs`: ClipboardEntry, ContentType
- `storage.rs`: Gerenciamento de imagens
- `validator.rs`: Validações

---

## 🚀 Setup de Desenvolvimento

### 1. Instalar Dependências

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Dependências de desenvolvimento
sudo apt install \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev \
    qt6-base-dev \
    qt6-declarative-dev \
    libqt6svg6-dev
```

### 2. Clonar Repositório

```bash
git clone https://github.com/seu-usuario/clippit.git
cd clippit
```

### 3. Compilar

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### 4. Executar

```bash
# Daemon (em um terminal)
cargo run --bin clippit-daemon

# Popup (em outro terminal)
cargo run --bin clippit-popup

# Dashboard
cargo run --bin clippit-dashboard
```

---

## 🧪 Testing

### Testes Unitários

```bash
cargo test
```

### Testes Manuais

#### Testar Clipboard Monitor

```bash
# Terminal 1: Rodar daemon com logs
RUST_LOG=debug cargo run --bin clippit-daemon

# Terminal 2: Copiar algo
echo "teste" | wl-copy

# Verificar logs no Terminal 1
```

#### Testar Popup

```bash
# Rodar popup
cargo run --bin clippit-popup

# Navegar com ↑↓
# Pressionar Enter
# Verificar se copiou
```

#### Testar Hotkey

```bash
# Registrar hotkey (precisa do daemon rodando)
# Pressionar Super+V
# Ver se popup abre
```

---

## 📦 Build para Produção

### Compilação Otimizada

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

### Gerar .deb

```bash
./scripts/build-deb.sh
```

O pacote será gerado em `/tmp/clippit-deb-build/`

---

## 🔍 Debugging

### Logs Verbosos

```bash
# Daemon com debug
RUST_LOG=debug cargo run --bin clippit-daemon

# Popup com debug
RUST_LOG=debug cargo run --bin clippit-popup
```

### GTK Inspector

```bash
# Habilitar GTK Inspector
GTK_DEBUG=interactive cargo run --bin clippit-popup
```

### Valgrind (Memory Leaks)

```bash
valgrind --leak-check=full target/release/clippit-daemon
```

---

## 📐 Convenções de Código

### Rust Style

```bash
# Formatar código
cargo fmt

# Linter
cargo clippy

# Verificar antes de commit
cargo fmt && cargo clippy && cargo test
```

### Commits

Formato: `tipo(escopo): mensagem`

Tipos:
- `feat`: Nova funcionalidade
- `fix`: Correção de bug
- `docs`: Documentação
- `refactor`: Refatoração
- `test`: Testes

Exemplos:
```
feat(popup): adiciona preview de imagens
fix(daemon): corrige detecção de duplicatas
docs(readme): atualiza instruções de instalação
```

---

## 🗂️ Estrutura do Projeto

```
clippit/
├── crates/
│   ├── clippit-core/        # Biblioteca compartilhada
│   │   ├── src/
│   │   │   ├── config.rs
│   │   │   ├── history.rs
│   │   │   ├── types.rs
│   │   │   └── ...
│   │   └── Cargo.toml
│   │
│   ├── clippit-daemon/      # Daemon principal
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── monitor.rs
│   │   │   └── hotkey.rs
│   │   └── Cargo.toml
│   │
│   ├── clippit-popup/       # Popup GTK4
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── views/
│   │   │   ├── controllers/
│   │   │   └── models/
│   │   └── Cargo.toml
│   │
│   ├── clippit-dashboard/   # Dashboard Qt6
│   │   └── ...
│   │
│   └── clippit-ipc/         # IPC library
│       └── ...
│
├── scripts/                 # Build scripts
│   ├── build-deb.sh
│   └── install.sh
│
├── docs/                    # Documentação
│   ├── DEVELOPMENT.md
│   └── ...
│
├── Cargo.toml              # Workspace
└── README.md
```

---

## 🔧 Tecnologias Utilizadas

### Backend

- **Rust 1.75+** - Linguagem principal
- **tokio** - Runtime assíncrono
- **rusqlite** - Banco de dados SQLite
- **serde** - Serialização/deserialização
- **arboard** - Clipboard cross-platform (Wayland-native)
- **global-hotkey** - Hotkeys globais (desktop portals)

### Frontend (Popup)

- **GTK4** - Toolkit UI
- **libadwaita** - Componentes modernos
- **gtk-rs** - Bindings Rust para GTK

### Frontend (Dashboard)

- **Qt6** - Framework UI
- **QML** - UI declarativa
- **cxx-qt** - Bindings Rust para Qt

---

## 🌐 Internacionalização (i18n)

### Adicionar Nova Tradução

1. Criar arquivo em `crates/clippit-core/locales/`:

```yaml
# locales/es.yml
popup:
  title: "Historial del portapapeles"
  copy_button_tooltip: "Copiar"
  # ...
```

2. Usar no código:

```rust
use rust_i18n::t;

let title = t!("popup.title");
```

---

## 📊 Performance

### Profiling

```bash
# CPU profiling
cargo flamegraph --bin clippit-daemon

# Heap profiling
cargo bloat --release --bin clippit-daemon
```

### Benchmarks

```bash
cargo bench
```

---

## 🔐 Security

### Considerações

- **Wayland**: Clipboard via arboard (wl-clipboard-rs), seguro e nativo
- **SQLite**: Banco local, sem acesso remoto
- **IPC**: Unix socket local (`/tmp/clippit.sock`)
- **Permissions**: Daemon roda como usuário (não root)

### Sanitization

- Inputs são validados antes de salvar no banco
- Paths são canonicalizados antes de uso
- SQL usa prepared statements (SQLi-safe)

---

## 🤝 Contribuindo

### Fluxo

1. Fork o repositório
2. Crie branch: `git checkout -b feat/minha-feature`
3. Commit: `git commit -m 'feat: adiciona X'`
4. Push: `git push origin feat/minha-feature`
5. Abra Pull Request

### Checklist PR

- [ ] Código formatado (`cargo fmt`)
- [ ] Sem warnings de clippy (`cargo clippy`)
- [ ] Testes passando (`cargo test`)
- [ ] Documentação atualizada
- [ ] CHANGELOG.md atualizado

---

## 📚 Recursos

- [Rust Book](https://doc.rust-lang.org/book/)
- [GTK4 Docs](https://docs.gtk.org/gtk4/)
- [libadwaita Docs](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- [arboard](https://github.com/1Password/arboard)
- [Wayland Protocol](https://wayland.freedesktop.org/)

---

**Dúvidas?** Abra um [issue](https://github.com/seu-usuario/clippit/issues) ou entre no [Discord](#)
