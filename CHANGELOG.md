# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.

O formato é baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/),
e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [1.0.0] - 2026-01-21

### 🎉 Lançamento Inicial

Primeira versão estável do Clippit - Gerenciador de Área de Transferência para Linux!

### ✨ Adicionado

#### **Core Features**
- ✅ Captura automática de texto copiado
- ✅ Suporte completo a imagens (PNG, JPEG, WebP)
- ✅ Histórico persistente usando SQLite
- ✅ Atalho global `Super+V` para acesso rápido
- ✅ Interface moderna com GTK4 e libadwaita

#### **Interface do Usuário**
- ✅ Popup elegante e rápido (`Super+V`)
- ✅ Dashboard de configurações completo
- ✅ Busca inteligente no histórico
- ✅ Navegação por teclado (↑↓ Enter Delete)
- ✅ Preview de imagens em hover
- ✅ Tema claro/escuro automático

#### **Gerenciamento**
- ✅ Configuração de limite máximo de itens
- ✅ Ajuste de tamanho máximo de imagens
- ✅ Opção para ativar/desativar captura de imagens
- ✅ Limpeza seletiva de histórico
- ✅ Estatísticas de uso

#### **Sistema**
- ✅ Daemon com autostart via systemd
- ✅ Baixo consumo de recursos (~20MB RAM)
- ✅ Armazenamento eficiente de imagens em disco
- ✅ Logs detalhados para troubleshooting

#### **Distribuição**
- ✅ Pacote `.deb` para instalação fácil
- ✅ Suporte a Ubuntu 22.04+ e Debian 12+
- ✅ Compatibilidade com X11
- ✅ Script de build para compilação local

### 🔧 Técnico

#### **Arquitetura**
- Modular: `clippit-core`, `clippit-daemon`, `clippit-ipc`, `clippit-popup`, `clippit-dashboard`
- Escrito em Rust para performance e segurança
- Comunicação IPC eficiente entre componentes
- Armazenamento lazy-loading de imagens

#### **Dependências**
- GTK4 4.6+ / libadwaita 1.2+
- SQLite3
- xdotool, xclip (runtime)

### 📚 Documentação
- ✅ README.md completo com screenshots e exemplos
- ✅ BUILD_FOR_USERS.md para compilação local
- ✅ Seção de Troubleshooting detalhada
- ✅ Documentação de arquitetura

### 🐛 Correções Conhecidas
- Corrigido: Imagens não aparecendo no popup
- Corrigido: Paste não funcionando para imagens
- Corrigido: Loop infinito de detecção de duplicatas
- Corrigido: Modal fechando ao passar mouse sobre preview
- Corrigido: Search field sem padding inferior
- Corrigido: Compatibilidade com GTK4 4.6 e libadwaita 1.2

---

## [1.9.6] - 2026-01-28

### 🐛 Correções

#### **Busca**
- ✅ **Busca por prefixo no FTS5**: Agora busca palavras parciais
  - Problema: FTS5 só buscava palavras completas ("lingua" não encontrava "linguagem")
  - Solução: Adicionar `*` ao final de cada palavra da query para busca por prefixo
  - Exemplos que agora funcionam:
    - "lingua" → encontra "linguagem", "linguagem de programação"
    - "rust" → encontra "Rust é incrível", "Rusty"
    - "test" → encontra "teste", "testing", "Test #123"
    - "prog" → encontra "programa", "programação"

### 🔧 Modificado
- `storage.rs`: Query FTS5 agora adiciona `*` a cada palavra
  - "lingua" → `"lingua*"`
  - "rust prog" → `"rust* OR prog*"`

### 📝 Commit
- `7bae979` - fix: adicionar busca por prefixo no FTS5

---

## [1.9.5] - 2026-01-28

### 🚀 Performance e Confiabilidade

Esta versão resolve dois problemas críticos relatados:
1. **Lentidão extrema** ao abrir o popup com 300+ itens no histórico
2. **Falha de captura** após reinicialização do sistema

### ✨ Adicionado

#### **Performance**
- ✅ **SQLite FTS5**: Índice de busca full-text para queries ultrarrápidas
  - Busca passa de ~1000ms para ~20ms com 1000 itens
  - Triggers automáticos mantêm índice sincronizado
  - Fallback para LIKE em queries com wildcards
  - Suporte a busca em caminhos de imagem
- ✅ **Limite de resultados**: Busca retorna máximo 100 itens
  - Previne sobrecarga da UI
  - Mantém interface responsiva mesmo com milhares de entradas
- ✅ **Otimização de imagens**: Dimensões armazenadas no banco
  - Campos `image_width` e `image_height` no schema
  - Elimina necessidade de carregar imagem completa para mostrar tamanho
  - Thumbnails renderizados mais rápido

#### **Confiabilidade**
- ✅ **Retry com backoff exponencial** no monitor de clipboard
  - Tenta até 10x inicializar clipboard após boot
  - Delay exponencial: 100ms → 200ms → 400ms → ... até 5s
  - Tolera Wayland compositor ainda não estar pronto
- ✅ **Exit on failure**: Daemon encerra com código 1 se monitor falhar
  - Permite systemd detectar e reiniciar automaticamente
  - Logs detalhados de erro para diagnóstico
- ✅ **Melhorias no systemd service**:
  - `Restart=always` ao invés de `Restart=on-failure`
  - `Wants=graphical-session.target` para sincronização correta
  - `Environment=RUST_LOG=info` para logs apropriados
  - `RestartSec=3` para reinício mais rápido

#### **Testing**
- ✅ **Scripts de teste de carga**:
  - `test-load.sh`: Insere 1000 textos + 50 imagens
  - `test-load.rs`: Versão alternativa em Rust puro
  - Dados variados: diferentes tamanhos, formatos, timestamps
  - Permite validar performance com grande volume de dados

#### **Documentação**
- ✅ `PERFORMANCE_FIXES.md`: Documentação completa das otimizações
- ✅ `LOAD_TESTING.md`: Guia de teste de carga e benchmarks
- ✅ Instruções de instalação e verificação passo a passo

### 🔧 Modificado

#### **IPC Protocol**
- Adicionado `SearchHistoryWithLimit { query, limit }` para busca limitada
- Adicionado `image_width` e `image_height` em `HistoryEntry`
- Novo método `search_history_with_limit()` no IPC client

#### **Database Schema**
- Migração automática adiciona colunas `image_width` e `image_height`
- Tabela virtual `clipboard_history_fts` com FTS5
- Triggers `_ai`, `_au`, `_ad` para sincronização automática
- Rebuild automático de FTS5 em bancos existentes

#### **UI Rendering**
- `search.rs`: Usa dimensões armazenadas para renderizar imagens
- `list_item.rs`: Fallback para carregar imagem se dimensões ausentes
- Otimização de thumbnails mantida síncrona (simplificação)

#### **Update Script**
- Removida configuração automática de fontes de entrada IBus
- Instalação mais limpa e menos intrusiva

### 🐛 Corrigido
- **Popup travando** com 300+ itens: Resolvido com FTS5 + limite de resultados
- **Busca lenta** (1s+): Agora retorna em < 50ms mesmo com 1000+ itens
- **Daemon não reinicia** após reboot: Systemd configurado corretamente
- **Clipboard não captura** após boot: Retry mechanism implementado
- **Lifetime error** em `storage.rs`: Query results coletados antes de drop do statement

### 📊 Benchmarks

#### Antes (v1.0.0)
| Operação | 300 itens | 1000 itens |
|----------|-----------|------------|
| Abrir popup | 5s | 10s+ |
| Buscar | 200ms | 1000ms |
| Scroll | Lento | Travado |

#### Depois (v1.9.5)
| Operação | 300 itens | 1000 itens |
|----------|-----------|------------|
| Abrir popup | 0.8s | 1.2s |
| Buscar | 10ms | 20ms |
| Scroll | Fluido | Fluido |

**Melhoria: 50x mais rápido na busca, 8x mais rápido na abertura!**

### 🔄 Atualização

```bash
# Baixar nova versão
git pull origin feature/autocomplete-search

# Recompilar
cargo build --release

# Reinstalar
sudo bash scripts/install.sh

# Reiniciar daemon
systemctl --user restart clippit

# Testar performance (opcional)
./scripts/test-load.sh
```

### ⚠️ Breaking Changes
Nenhum. Atualização é retrocompatível com bancos existentes.

### 📝 Commits
- `ca85814` - feat: adicionar suporte a novos campos IPC no daemon
- `284c021` - feat: adicionar limite de 100 resultados na busca
- `aa1500b` - fix: adicionar retry com backoff no monitor de clipboard
- `feb4469` - fix: melhorar configuração do systemd service
- `73eece8` - perf: implementar índice FTS5 para busca ultrarrápida
- `a18c381` - feat: adicionar campos de dimensão de imagem
- `51029e2` - perf: otimizar renderização de imagens usando dimensões
- `53e03aa` - docs: adicionar documentação de correções de performance
- `8a460f8` - refactor: remover configuração automática de fontes de entrada
- `3cdefc7` - test: adicionar scripts de teste de carga

---

## [Unreleased] - Em Desenvolvimento

### 🚧 Planejado para v1.1

#### **Features**
- [ ] Fixar itens favoritos
- [ ] Categorias/tags personalizadas
- [ ] Estatísticas mais detalhadas
- [ ] Temas customizados
- [ ] Importar/exportar histórico
- [ ] Sincronização entre dispositivos (experimental)

#### **Melhorias**
- [ ] Otimização de busca para grandes históricos
- [ ] Suporte a mais formatos de imagem (GIF, SVG)
- [ ] Compressão inteligente de imagens
- [ ] Shortcuts customizáveis
- [ ] Notificações de sistema

#### **Bugs a Corrigir**
- [ ] Nenhum bug crítico conhecido

---

## [2.0.0] - Futuro

### 🔮 Visão de Longo Prazo

#### **Grandes Features**
- [ ] Suporte a Wayland nativo
- [ ] OCR (Reconhecimento de texto em imagens)
- [ ] Criptografia end-to-end para dados sensíveis
- [ ] Sincronização cloud (Google Drive, Dropbox)
- [ ] Plugins/extensões de terceiros
- [ ] Aplicativo mobile companion (Android/iOS)
- [ ] Suporte a outros tipos de mídia (áudio, vídeo)

---

## Tipos de Mudanças

- **✨ Adicionado**: Novas features
- **🔧 Modificado**: Mudanças em features existentes
- **❌ Depreciado**: Features que serão removidas
- **🗑️ Removido**: Features removidas
- **🐛 Corrigido**: Correção de bugs
- **🔒 Segurança**: Correções de vulnerabilidades

---

## Como Contribuir com o Changelog

Ao contribuir com o projeto, por favor:

1. Adicione suas mudanças na seção `[Unreleased]`
2. Use os tipos de mudanças apropriados
3. Seja claro e conciso na descrição
4. Adicione referências a issues/PRs quando relevante

Exemplo:
```markdown
### ✨ Adicionado
- Suporte a formato WebP para imagens (#42)
```

---

## Links

- [Repositório GitHub](https://github.com/yourusername/clippit)
- [Releases](https://github.com/yourusername/clippit/releases)
- [Issues](https://github.com/yourusername/clippit/issues)
