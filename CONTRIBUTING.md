# 🤝 Guia de Contribuição

Obrigado por considerar contribuir com o Clippit! ❤️

Este documento fornece diretrizes para contribuir com o projeto. Seguir estas diretrizes ajuda a comunicar que você respeita o tempo dos desenvolvedores que gerenciam e desenvolvem este projeto de código aberto.

---

## 📋 Índice

- [Código de Conduta](#-código-de-conduta)
- [Como Posso Contribuir?](#-como-posso-contribuir)
- [Primeiros Passos](#-primeiros-passos)
- [Processo de Desenvolvimento](#-processo-de-desenvolvimento)
- [Guia de Estilo](#-guia-de-estilo)
- [Estrutura do Projeto](#-estrutura-do-projeto)
- [Testes](#-testes)
- [Documentação](#-documentação)
- [Pull Requests](#-pull-requests)
- [Comunidade](#-comunidade)

---

## 📜 Código de Conduta

Este projeto e todos que participam dele são regidos pelo nosso Código de Conduta. Ao participar, espera-se que você mantenha este código. Por favor, reporte comportamento inaceitável para [clippit@example.com](mailto:clippit@example.com).

### Nossos Valores

- **Seja respeitoso**: Trate todos com respeito e consideração
- **Seja colaborativo**: Trabalhe junto com outros contribuidores
- **Seja construtivo**: Forneça feedback construtivo
- **Seja inclusivo**: Dê boas-vindas a todos, independentemente de experiência

---

## 🎯 Como Posso Contribuir?

Existem muitas formas de contribuir com o Clippit:

### 🐛 Reportar Bugs

Encontrou um bug? Ajude-nos a corrigi-lo!

**Antes de reportar:**
- ✅ Verifique se o bug já foi reportado nas [Issues](https://github.com/yourusername/clippit/issues)
- ✅ Verifique se você está usando a versão mais recente
- ✅ Colete informações sobre o bug

**Como reportar:**
1. Abra uma [nova issue](https://github.com/yourusername/clippit/issues/new)
2. Use um título claro e descritivo
3. Descreva os passos para reproduzir
4. Forneça informações do sistema:
   ```bash
   # Versão do Ubuntu/Debian
   lsb_release -a
   
   # Versão do GTK4
   pkg-config --modversion gtk4
   
   # Logs do Clippit
   journalctl --user -u clippit -n 50
   ```
5. Se possível, adicione screenshots ou vídeos

**Template de Bug Report:**
```markdown
## Descrição do Bug
[Descrição clara e concisa]

## Passos para Reproduzir
1. Vá para '...'
2. Clique em '...'
3. Role até '...'
4. Veja o erro

## Comportamento Esperado
[O que deveria acontecer]

## Comportamento Atual
[O que está acontecendo]

## Screenshots
[Se aplicável]

## Informações do Sistema
- OS: [ex: Ubuntu 22.04]
- GTK4: [ex: 4.6.9]
- Clippit: [ex: 1.0.0]

## Logs
```
[Cole os logs aqui]
```
```

### ✨ Sugerir Features

Tem uma ideia para melhorar o Clippit?

**Antes de sugerir:**
- ✅ Verifique se já não foi sugerida
- ✅ Verifique se está alinhada com os objetivos do projeto

**Como sugerir:**
1. Abra uma [nova issue](https://github.com/yourusername/clippit/issues/new)
2. Use a label `enhancement`
3. Descreva a feature em detalhes
4. Explique por que seria útil
5. Forneça exemplos de uso

### 📝 Melhorar Documentação

Documentação é crucial! Você pode ajudar:
- Corrigindo erros de digitação
- Melhorando explicações
- Adicionando exemplos
- Traduzindo para outros idiomas

### 💻 Contribuir com Código

Quer contribuir com código? Ótimo! Veja as seções abaixo.

---

## 🚀 Primeiros Passos

### Pré-requisitos

**Ferramentas Necessárias:**
```bash
# Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Dependências de desenvolvimento
sudo apt install \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libsqlite3-dev
```

### Configurando o Ambiente

1. **Fork o repositório**
   ```bash
   # No GitHub, clique em "Fork"
   ```

2. **Clone seu fork**
   ```bash
   git clone https://github.com/SEU-USERNAME/clippit.git
   cd clippit
   ```

3. **Adicione o repositório original como upstream**
   ```bash
   git remote add upstream https://github.com/yourusername/clippit.git
   ```

4. **Instale as dependências**
   ```bash
   cargo build
   ```

5. **Execute os testes**
   ```bash
   cargo test
   ```

### Rodando em Desenvolvimento

```bash
# Terminal 1: Execute o daemon
RUST_LOG=clippit_daemon=debug cargo run --bin clippit-daemon

# Terminal 2: Execute o popup
RUST_LOG=clippit_popup=debug cargo run --bin clippit-popup

# Ou execute o dashboard
cargo run --bin clippit-dashboard
```

---

## 🔧 Processo de Desenvolvimento

### 1. Escolha uma Issue

- Procure issues com labels `good first issue` ou `help wanted`
- Comente na issue dizendo que vai trabalhar nela
- Aguarde confirmação de um mantenedor

### 2. Crie uma Branch

```bash
# Atualize seu fork
git checkout main
git pull upstream main

# Crie uma branch para sua feature/fix
git checkout -b feature/minha-feature
# ou
git checkout -b fix/meu-bugfix
```

**Convenção de Nomes:**
- `feature/nome-da-feature` - Para novas features
- `fix/nome-do-bug` - Para correções de bugs
- `docs/descricao` - Para documentação
- `refactor/descricao` - Para refatorações
- `test/descricao` - Para testes

### 3. Desenvolva

- Faça commits pequenos e frequentes
- Siga o [Guia de Estilo](#-guia-de-estilo)
- Adicione testes para novas features
- Atualize a documentação se necessário

### 4. Teste

```bash
# Execute todos os testes
cargo test

# Teste manualmente
cargo run --bin clippit-daemon

# Verifique linting
cargo clippy -- -D warnings

# Verifique formatação
cargo fmt --check
```

### 5. Commit

Siga o [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Formato
<tipo>(<escopo>): <descrição>

# Exemplos
feat(popup): adiciona suporte a GIF
fix(daemon): corrige memory leak no monitor
docs(readme): atualiza instruções de instalação
refactor(core): melhora performance do histórico
test(ipc): adiciona testes de integração
```

**Tipos:**
- `feat`: Nova feature
- `fix`: Correção de bug
- `docs`: Documentação
- `style`: Formatação, ponto e vírgula, etc
- `refactor`: Refatoração de código
- `test`: Adicionando testes
- `chore`: Manutenção, dependências

### 6. Push e Pull Request

```bash
# Push para seu fork
git push origin feature/minha-feature

# No GitHub, abra um Pull Request
```

---

## 🎨 Guia de Estilo

### Rust

**Formatação:**
```bash
# Use rustfmt
cargo fmt
```

**Linting:**
```bash
# Use clippy
cargo clippy -- -D warnings
```

**Convenções:**
- Use snake_case para funções e variáveis
- Use PascalCase para tipos e traits
- Use SCREAMING_SNAKE_CASE para constantes
- Documente funções públicas com `///`
- Use `?` ao invés de `unwrap()` sempre que possível
- Evite `panic!()` exceto em casos irrecuperáveis

**Exemplo:**
```rust
/// Adiciona uma entrada ao histórico.
///
/// # Arguments
///
/// * `content` - O conteúdo a ser adicionado
/// * `entry_type` - O tipo da entrada (Text ou Image)
///
/// # Returns
///
/// `Result<i64, Error>` - O ID da entrada ou erro
///
/// # Examples
///
/// ```
/// let id = add_entry("Hello", EntryType::Text)?;
/// ```
pub fn add_entry(content: &str, entry_type: EntryType) -> Result<i64, Error> {
    // Implementação
}
```

### Git Commits

- Use mensagens descritivas
- Primeira linha: máximo 50 caracteres
- Corpo: máximo 72 caracteres por linha
- Use português ou inglês (seja consistente)

**Exemplo:**
```
feat(popup): adiciona preview de imagem em hover

Implementa preview de imagem quando o usuário passa o mouse
sobre um item de imagem no histórico. O preview é exibido
ao lado do item com um fade-in suave.

Closes #42
```

### Documentação

- Use Markdown
- Adicione exemplos sempre que possível
- Mantenha o README.md atualizado
- Documente decisões arquiteturais importantes

---

## 📁 Estrutura do Projeto

```
clippit/
├── crates/                      # Crates Rust
│   ├── clippit-core/           # Lógica de negócio
│   │   ├── src/
│   │   │   ├── history.rs      # Gerenciamento de histórico
│   │   │   └── config.rs       # Configurações
│   │   └── Cargo.toml
│   │
│   ├── clippit-daemon/         # Daemon de monitoramento
│   │   ├── src/
│   │   │   ├── main.rs         # Entry point
│   │   │   ├── monitor.rs      # Monitor de clipboard
│   │   │   └── hotkey.rs       # Gerenciamento de hotkeys
│   │   └── Cargo.toml
│   │
│   ├── clippit-ipc/            # Comunicação IPC
│   ├── clippit-popup/          # Interface popup
│   └── clippit-dashboard/      # Dashboard de config
│
├── assets/                      # Assets (ícones, etc)
├── scripts/                     # Scripts de build/instalação
├── docs/                        # Documentação adicional
│
├── README.md                    # Documentação principal
├── CONTRIBUTING.md              # Este arquivo
├── CHANGELOG.md                 # Histórico de mudanças
├── LICENSE                      # Licença MIT
└── Cargo.toml                   # Workspace Cargo

```

### Onde Adicionar Código

| Feature | Crate | Arquivo |
|---------|-------|---------|
| Novo tipo de entrada | `clippit-core` | `history.rs` |
| Nova configuração | `clippit-core` | `config.rs` |
| Monitoramento de clipboard | `clippit-daemon` | `monitor.rs` |
| Novo hotkey | `clippit-daemon` | `hotkey.rs` |
| UI do popup | `clippit-popup` | `src/views/` |
| UI do dashboard | `clippit-dashboard` | `src/ui/` |

---

## 🧪 Testes

### Rodando Testes

```bash
# Todos os testes
cargo test

# Testes de um crate específico
cargo test -p clippit-core

# Um teste específico
cargo test test_add_entry

# Com output detalhado
cargo test -- --nocapture
```

### Escrevendo Testes

**Testes Unitários:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_text_entry() {
        let mut history = History::new();
        let result = history.add_entry("test", EntryType::Text);
        assert!(result.is_ok());
    }
}
```

**Testes de Integração:**
```rust
// tests/integration_test.rs
#[test]
fn test_daemon_popup_communication() {
    // Setup
    let daemon = start_daemon();
    let popup = start_popup();
    
    // Test
    daemon.copy_text("hello");
    let entries = popup.get_entries();
    
    // Assert
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].content, "hello");
}
```

---

## 📚 Documentação

### Documentando Código

Use `///` para documentação pública:

```rust
/// Estrutura que representa uma entrada no histórico.
///
/// # Fields
///
/// * `id` - ID único da entrada
/// * `content` - Conteúdo da entrada
/// * `entry_type` - Tipo (Text ou Image)
/// * `timestamp` - Quando foi criada
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub content: String,
    pub entry_type: EntryType,
    pub timestamp: DateTime<Utc>,
}
```

### Atualizando README

Ao adicionar features, atualize:
- Lista de features
- Instruções de uso
- Screenshots (se aplicável)

### CHANGELOG.md

Adicione suas mudanças em `[Unreleased]`:

```markdown
### ✨ Adicionado
- Suporte a formato GIF (#123)
```

---

## 🔀 Pull Requests

### Checklist

Antes de submeter um PR, verifique:

- [ ] ✅ Código segue o guia de estilo
- [ ] ✅ Testes passando (`cargo test`)
- [ ] ✅ Sem warnings (`cargo clippy`)
- [ ] ✅ Código formatado (`cargo fmt`)
- [ ] ✅ Documentação atualizada
- [ ] ✅ CHANGELOG.md atualizado
- [ ] ✅ Commits seguem Conventional Commits

### Template de PR

```markdown
## Descrição
[Descrição clara do que foi feito]

## Tipo de Mudança
- [ ] 🐛 Bug fix (mudança que corrige um issue)
- [ ] ✨ Nova feature (mudança que adiciona funcionalidade)
- [ ] 💥 Breaking change (fix ou feature que quebra compatibilidade)
- [ ] 📝 Documentação

## Como Testar
1. [Passo 1]
2. [Passo 2]
3. [Passo 3]

## Screenshots
[Se aplicável]

## Issues Relacionadas
Closes #[número da issue]

## Checklist
- [ ] Testes passando
- [ ] Código documentado
- [ ] CHANGELOG atualizado
```

### Processo de Review

1. Um mantenedor irá revisar seu PR
2. Pode haver pedidos de mudanças
3. Faça as mudanças solicitadas
4. Após aprovação, seu PR será merged!

---

## 👥 Comunidade

### Onde Pedir Ajuda

- 💬 **GitHub Discussions**: Para discussões gerais
- 🐛 **GitHub Issues**: Para bugs e features
- 📧 **Email**: clippit@example.com

### Mantenedores

- [@yourusername](https://github.com/yourusername) - Mantenedor Principal

### Contribuidores

Veja a lista completa de [contribuidores](https://github.com/yourusername/clippit/graphs/contributors)!

---

## 🎉 Reconhecimento

Toda contribuição, grande ou pequena, é valiosa! Contribuidores serão:

- Listados no README.md
- Mencionados no CHANGELOG.md
- Incluídos na lista de Contributors do GitHub

---

## 📞 Precisa de Ajuda?

Não hesite em pedir ajuda! Estamos aqui para ajudar:

- Comente na issue que você está trabalhando
- Abra uma discussão no GitHub Discussions
- Envie um email para clippit@example.com

---

## 🙏 Agradecimentos

Obrigado por contribuir com o Clippit! Sua ajuda faz toda a diferença! ❤️

---

<div align="center">

**Happy Coding!** 🚀

[⬆ Voltar ao topo](#-guia-de-contribuição)

</div>
