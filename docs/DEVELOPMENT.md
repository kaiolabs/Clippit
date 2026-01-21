# Guia de Desenvolvimento - Clippit

## Arquitetura do Sistema

O Clippit é dividido em 4 crates principais:

### clippit-core
Core logic do aplicativo com gerenciamento de histórico e armazenamento.

**Módulos:**
- `storage.rs`: Interface com SQLite
- `history.rs`: Gerenciamento de histórico com anti-duplicação
- `validator.rs`: Validação de conteúdo (texto e imagem)
- `types.rs`: Tipos compartilhados

**Testes:**
```bash
cargo test -p clippit-core
```

### clippit-ipc
Comunicação entre daemon e UI via Unix sockets.

**Módulos:**
- `protocol.rs`: Definição de mensagens IPC
- `server.rs`: Servidor IPC (usado pelo daemon)
- `client.rs`: Cliente IPC (usado pela UI)

**Socket:** `/tmp/clippit.sock`

### clippit-daemon
Background service que monitora o clipboard e gerencia hotkeys.

**Componentes:**
- `monitor.rs`: Monitoramento do clipboard X11 (polling a cada 200ms)
- `hotkey.rs`: Gerenciamento do atalho Super+V
- `main.rs`: Orquestração e servidor IPC

**Iniciar daemon:**
```bash
RUST_LOG=clippit_daemon=debug cargo run --bin clippit-daemon
```

### clippit-ui
Interface do usuário (CLI no MVP, Qt/QML planejado para V2).

**Funcionalidades:**
- Conexão com daemon via IPC
- Exibição de histórico
- Seleção e cópia de itens

**Executar UI:**
```bash
cargo run --bin clippit-ui
```

## Fluxo de Dados

```
[Usuário copia texto]
         ↓
[X11 Clipboard] ← [Clipboard Monitor] (polling)
         ↓
[Content Validator]
         ↓
[History Manager] → [SQLite Storage]
         ↓
[Usuário: Super+V]
         ↓
[Hotkey Handler] → [IPC] → [UI]
         ↓
[UI query history] → [IPC] → [History Manager]
         ↓
[Usuário seleciona] → [IPC] → [Daemon] → [X11 Clipboard]
```

## Desenvolvimento

### Setup

```bash
# Instalar dependências (Ubuntu/Debian)
sudo apt install libx11-dev libxcb1-dev libsqlite3-dev

# Clonar e compilar
git clone <repo>
cd clippit
cargo build
```

### Executar em modo dev

Terminal 1 (daemon):
```bash
RUST_LOG=clippit_daemon=debug cargo run --bin clippit-daemon
```

Terminal 2 (UI):
```bash
cargo run --bin clippit-ui
```

### Testes

```bash
# Todos os testes
cargo test

# Teste específico de crate
cargo test -p clippit-core

# Com output
cargo test -- --nocapture

# Teste específico
cargo test test_duplicate_detection
```

### Verificação de código

```bash
# Check (compilação sem binários)
cargo check --all

# Clippy (linter)
cargo clippy --all

# Format
cargo fmt --all
```

## Debugging

### Logs do daemon

```bash
# Via systemd
journalctl --user -u clippit -f

# Direto (modo dev)
RUST_LOG=debug cargo run --bin clippit-daemon
```

### Debug do IPC

```bash
# Verificar se socket existe
ls -la /tmp/clippit.sock

# Testar conexão
cargo run --bin clippit-ui
```

### Debug do clipboard

```bash
# Verificar X11
echo $XDG_SESSION_TYPE  # Deve ser "x11"

# Copiar texto de teste
echo "test" | xclip -selection clipboard

# Ver logs do daemon
RUST_LOG=clippit_daemon=debug cargo run --bin clippit-daemon
```

## Estrutura de Banco de Dados

```sql
CREATE TABLE clipboard_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type TEXT NOT NULL,
    content_text TEXT,
    content_data BLOB,
    timestamp TEXT NOT NULL
);

CREATE INDEX idx_timestamp ON clipboard_history(timestamp DESC);
```

**Localização:** `~/.local/share/clippit/history.db`

## Adicionando Funcionalidades

### 1. Novo tipo de conteúdo

1. Adicionar em `clippit-core/src/types.rs`:
```rust
pub enum ContentType {
    Text,
    Image,
    File, // Novo tipo
}
```

2. Adicionar validação em `validator.rs`
3. Atualizar `monitor.rs` para capturar o novo tipo
4. Atualizar `storage.rs` se necessário

### 2. Nova mensagem IPC

1. Adicionar em `clippit-ipc/src/protocol.rs`:
```rust
pub enum IpcMessage {
    // ... existentes
    NewMessage { params: String },
}
```

2. Implementar handler em `clippit-daemon/src/main.rs`
3. Adicionar método em `clippit-ipc/src/client.rs`

### 3. Novo comando UI

Adicionar função em `clippit-ui/src/ui.rs`

## Performance

### Métricas esperadas

- Uso de memória: < 50MB em idle
- Tempo de resposta UI: < 100ms
- CPU em idle: < 1%
- Latência IPC: < 1ms

### Profiling

```bash
# Com flamegraph
cargo install flamegraph
sudo flamegraph cargo run --release --bin clippit-daemon

# Com perf
cargo build --release
perf record ./target/release/clippit-daemon
perf report
```

## Troubleshooting Comum

### Daemon não inicia

```bash
# Verificar se já está rodando
ps aux | grep clippit-daemon

# Remover socket antigo
rm /tmp/clippit.sock

# Verificar permissões
ls -la ~/.local/share/clippit/
```

### Hotkey não funciona

```bash
# Verificar conflitos
dconf read /org/gnome/desktop/wm/keybindings/

# Testar com outro atalho (editar hotkey.rs)
```

### Tests falhando

```bash
# Limpar build
cargo clean

# Rebuild
cargo build

# Run tests com output
cargo test -- --nocapture
```

## Roadmap Técnico

### V1.1 (Próximas melhorias)
- [ ] Interface Qt/QML completa
- [ ] Suporte a imagens completo (não só validação)
- [ ] Busca por texto no histórico
- [ ] Configuração via arquivo TOML

### V2.0
- [ ] Suporte Wayland
- [ ] Sincronização entre máquinas
- [ ] Criptografia de dados sensíveis
- [ ] Plugin system

## Contribuindo

1. Fork o projeto
2. Crie uma branch: `git checkout -b feature/nova-funcionalidade`
3. Commit: `git commit -am 'Adiciona nova funcionalidade'`
4. Push: `git push origin feature/nova-funcionalidade`
5. Abra um Pull Request

### Checklist PR

- [ ] Código compila sem warnings
- [ ] Testes adicionados para nova funcionalidade
- [ ] Testes passando
- [ ] Documentação atualizada
- [ ] CHANGELOG.md atualizado
