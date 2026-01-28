# Clippit-oud - Cursor Rules Documentation

Bem-vindo à documentação completa de regras e padrões do projeto Clippit-oud.

## 📚 Índice Geral

### 🎯 Fundamentos
- **[00-PROJECT-OVERVIEW.md](./00-PROJECT-OVERVIEW.md)** - Visão geral completa do projeto
- **[01-ARCHITECTURE.md](./01-ARCHITECTURE.md)** - Arquitetura detalhada com diagramas
- **[02-DEVELOPMENT-STANDARDS.md](./02-DEVELOPMENT-STANDARDS.md)** - Padrões de código e desenvolvimento

### 🧠 Core Library (`clippit-core`)
- **[CORE-OVERVIEW.md](./core/CORE-OVERVIEW.md)** - Visão geral da biblioteca compartilhada
- **[CONFIG-PATTERNS.md](./core/CONFIG-PATTERNS.md)** - Sistema de configuração TOML
- **[HISTORY-STORAGE.md](./core/HISTORY-STORAGE.md)** - Gerenciamento de histórico e SQLite
- **[TYPES-DEFINITIONS.md](./core/TYPES-DEFINITIONS.md)** - Tipos de dados principais
- **[VALIDATION.md](./core/VALIDATION.md)** - Validação de conteúdo

### 👁️ Daemon (`clippit-daemon`)
- **[DAEMON-OVERVIEW.md](./daemon/DAEMON-OVERVIEW.md)** - Visão geral do serviço de background
- **[MONITOR-CLIPBOARD.md](./daemon/MONITOR-CLIPBOARD.md)** - Monitor de clipboard
- **[HOTKEYS-SYSTEM.md](./daemon/HOTKEYS-SYSTEM.md)** - Sistema de hotkeys globais
- **[TYPING-AUTOCOMPLETE.md](./daemon/TYPING-AUTOCOMPLETE.md)** - Monitor de digitação
- **[IPC-SERVER.md](./daemon/IPC-SERVER.md)** - Servidor IPC

### 🎨 User Interfaces
- **[UI-OVERVIEW.md](./ui/UI-OVERVIEW.md)** - Visão geral das interfaces
- **[POPUP-GTK.md](./ui/POPUP-GTK.md)** - Popup GTK4 (interface principal)
- **[DASHBOARD-QT.md](./ui/DASHBOARD-QT.md)** - Dashboard Qt/QML (configurações)
- **[TOOLTIP.md](./ui/TOOLTIP.md)** - Tooltip flutuante
- **[UI-PATTERNS.md](./ui/UI-PATTERNS.md)** - Padrões de UI/UX

### 🔌 Infrastructure
- **[IPC-PROTOCOL.md](./infrastructure/IPC-PROTOCOL.md)** - Protocolo de comunicação
- **[IBUS-ENGINE.md](./infrastructure/IBUS-ENGINE.md)** - Engine IBus
- **[QT-BRIDGE.md](./infrastructure/QT-BRIDGE.md)** - Bridge Rust-QML
- **[COMMUNICATION.md](./infrastructure/COMMUNICATION.md)** - Padrões de comunicação

### ✨ Features
- **[CLIPBOARD-CAPTURE.md](./features/CLIPBOARD-CAPTURE.md)** - Captura de clipboard
- **[AUTOCOMPLETE-GLOBAL.md](./features/AUTOCOMPLETE-GLOBAL.md)** - Autocomplete global
- **[SEARCH-SUGGESTIONS.md](./features/SEARCH-SUGGESTIONS.md)** - Busca e sugestões
- **[IMAGE-HANDLING.md](./features/IMAGE-HANDLING.md)** - Manipulação de imagens
- **[INTERNATIONALIZATION.md](./features/INTERNATIONALIZATION.md)** - Sistema i18n

### 🔧 Build & Deploy
- **[BUILD-SYSTEM.md](./build-deploy/BUILD-SYSTEM.md)** - Sistema de build Cargo
- **[PACKAGING.md](./build-deploy/PACKAGING.md)** - Empacotamento .deb
- **[INSTALLATION.md](./build-deploy/INSTALLATION.md)** - Instalação e setup
- **[DEPENDENCIES.md](./build-deploy/DEPENDENCIES.md)** - Gerenciamento de dependências

### 🧪 Testing
- **[TESTING-STRATEGY.md](./testing/TESTING-STRATEGY.md)** - Estratégia de testes
- **[UNIT-TESTS.md](./testing/UNIT-TESTS.md)** - Testes unitários
- **[INTEGRATION-TESTS.md](./testing/INTEGRATION-TESTS.md)** - Testes de integração

## 🗺️ Mapa de Navegação

### Por Área de Responsabilidade

```
Configuração
├─ core/CONFIG-PATTERNS.md
└─ ui/DASHBOARD-QT.md

Dados e Persistência
├─ core/HISTORY-STORAGE.md
├─ core/TYPES-DEFINITIONS.md
└─ core/VALIDATION.md

Captura de Clipboard
├─ daemon/MONITOR-CLIPBOARD.md
├─ features/CLIPBOARD-CAPTURE.md
└─ features/IMAGE-HANDLING.md

Interfaces de Usuário
├─ ui/POPUP-GTK.md
├─ ui/DASHBOARD-QT.md
├─ ui/TOOLTIP.md
└─ ui/UI-PATTERNS.md

Autocomplete
├─ daemon/TYPING-AUTOCOMPLETE.md
├─ infrastructure/IBUS-ENGINE.md
└─ features/AUTOCOMPLETE-GLOBAL.md

Comunicação
├─ infrastructure/IPC-PROTOCOL.md
├─ infrastructure/COMMUNICATION.md
└─ daemon/IPC-SERVER.md

Build e Deploy
├─ build-deploy/BUILD-SYSTEM.md
├─ build-deploy/PACKAGING.md
├─ build-deploy/INSTALLATION.md
└─ build-deploy/DEPENDENCIES.md
```

### Por Fluxo de Uso

#### Fluxo: Adicionar Item ao Histórico
1. [MONITOR-CLIPBOARD.md](./daemon/MONITOR-CLIPBOARD.md) - Detecta mudança
2. [VALIDATION.md](./core/VALIDATION.md) - Valida conteúdo
3. [HISTORY-STORAGE.md](./core/HISTORY-STORAGE.md) - Persiste no SQLite
4. [IMAGE-HANDLING.md](./features/IMAGE-HANDLING.md) - Processa imagens (se aplicável)

#### Fluxo: Abrir Popup
1. [HOTKEYS-SYSTEM.md](./daemon/HOTKEYS-SYSTEM.md) - Detecta Super+V
2. [IPC-PROTOCOL.md](./infrastructure/IPC-PROTOCOL.md) - Comunicação
3. [POPUP-GTK.md](./ui/POPUP-GTK.md) - Renderiza interface
4. [SEARCH-SUGGESTIONS.md](./features/SEARCH-SUGGESTIONS.md) - Busca em tempo real

#### Fluxo: Autocomplete Global
1. [IBUS-ENGINE.md](./infrastructure/IBUS-ENGINE.md) - Captura digitação
2. [TYPING-AUTOCOMPLETE.md](./daemon/TYPING-AUTOCOMPLETE.md) - Processa eventos
3. [HISTORY-STORAGE.md](./core/HISTORY-STORAGE.md) - Busca sugestões
4. [TOOLTIP.md](./ui/TOOLTIP.md) - Exibe popup flutuante

#### Fluxo: Configurar Aplicação
1. [DASHBOARD-QT.md](./ui/DASHBOARD-QT.md) - Interface de config
2. [QT-BRIDGE.md](./infrastructure/QT-BRIDGE.md) - Models Rust-QML
3. [CONFIG-PATTERNS.md](./core/CONFIG-PATTERNS.md) - Load/Save config

## 📐 Convenções de Documentação

### Formato de Arquivos
Todos os arquivos seguem o formato:

```markdown
# Título da Rule

## 📍 Localização
Caminho no projeto

## 🎯 Responsabilidade
Descrição clara e concisa

## [Seções específicas]
...

## 🔗 Links Relacionados
Links para outras rules

---
**Versão**: 1.0 | **Data**: 2026-01-28
```

### Ícones Utilizados
- 📍 Localização
- 🎯 Responsabilidade/Objetivo
- 🏗️ Arquitetura/Estrutura
- 🔄 Fluxo/Processo
- 📦 Componentes/Módulos
- 📊 Diagramas/Dados
- ⚙️ Configuração
- 📝 Documentação/Notas
- ✅ Regras/Padrões Obrigatórios
- 🚫 Anti-Patterns
- 🧪 Testes
- 🔗 Links/Referências

## 🔍 Como Usar Esta Documentação

### Para Novos Desenvolvedores
1. Comece com [00-PROJECT-OVERVIEW.md](./00-PROJECT-OVERVIEW.md)
2. Leia [01-ARCHITECTURE.md](./01-ARCHITECTURE.md)
3. Revise [02-DEVELOPMENT-STANDARDS.md](./02-DEVELOPMENT-STANDARDS.md)
4. Explore as áreas específicas conforme necessidade

### Para Implementar Nova Feature
1. Identifique a área (core, daemon, ui, infrastructure)
2. Leia as rules da área correspondente
3. Siga os padrões estabelecidos
4. Consulte exemplos em arquivos existentes
5. Escreva testes seguindo [TESTING-STRATEGY.md](./testing/TESTING-STRATEGY.md)

### Para Debugging
1. Identifique o componente com problema
2. Consulte a rule específica do componente
3. Revise fluxos de dados em [ARCHITECTURE.md](./01-ARCHITECTURE.md)
4. Verifique logs e testes relacionados

### Para Code Review
1. Verifique conformidade com [DEVELOPMENT-STANDARDS.md](./02-DEVELOPMENT-STANDARDS.md)
2. Confirme que padrões específicos da área foram seguidos
3. Valide que testes foram adicionados
4. Verifique documentação inline

## 📚 Documentação Complementar

Esta documentação de rules complementa a documentação existente no projeto:

- [README.md](../README.md) - Documentação principal do usuário
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Guia de contribuição
- [DEVELOPMENT.md](../docs/DEVELOPMENT.md) - Guia de desenvolvimento
- [AUTOCOMPLETE_IMPLEMENTATION.md](../AUTOCOMPLETE_IMPLEMENTATION.md) - Implementação do autocomplete
- [ROADMAP.md](../ROADMAP.md) - Roadmap do projeto
- [docs/](../docs/) - Documentação técnica adicional

## 🔄 Manutenção Desta Documentação

### Quando Atualizar
- ✅ Ao adicionar novo crate ou módulo
- ✅ Ao modificar arquitetura significativamente
- ✅ Ao estabelecer novos padrões
- ✅ Ao adicionar nova feature importante
- ✅ Quando anti-patterns forem identificados

### Como Atualizar
1. Edite o arquivo de rule correspondente
2. Mantenha formato e estrutura consistentes
3. Atualize links cruzados se necessário
4. Atualize data de versão
5. Commit com mensagem descritiva

### Responsabilidade
- **Mantenedores**: Revisam e aprovam mudanças
- **Contribuidores**: Atualizam conforme contribuem
- **IA/Cursor**: Utiliza como contexto para assistência

## 📊 Estatísticas

- **Total de Rules**: 32 arquivos
- **Áreas Cobertas**: 8 (core, daemon, ui, infrastructure, features, build-deploy, testing, general)
- **Última Atualização**: 2026-01-28
- **Versão**: 1.0

---

**Mantido por**: Clippit Team  
**Versão**: 1.0  
**Data de Criação**: 2026-01-28

Para sugestões ou correções, abra uma issue ou PR no repositório.
