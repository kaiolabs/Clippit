# Roadmap - Clippit

## ✅ V1.0 - MVP (Concluído)

**Status:** Lançado - 19 de Janeiro, 2026

- [x] Daemon de clipboard para X11
- [x] Histórico persistente em SQLite
- [x] Interface CLI funcional
- [x] Atalho global Super+V
- [x] IPC via Unix sockets
- [x] Scripts de instalação/desinstalação
- [x] Documentação completa
- [x] Testes unitários

---

## 🔄 V1.1 - Configuração e Melhorias (Em Desenvolvimento)

**Previsão:** Fevereiro 2026

### Sistema de Configuração ✅
- [x] Estrutura de configuração TOML
- [x] Arquivo de exemplo (clippit.example.toml)
- [x] Documentação completa (CONFIGURATION.md)
- [ ] Aplicar configurações no daemon
- [ ] Aplicar configurações na UI
- [ ] Hot-reload de configurações

### Interface Melhorada
- [ ] **Interface Qt/QML** (substituindo CLI)
  - [ ] Popup flutuante moderno
  - [ ] Animações suaves
  - [ ] Navegação por teclado e mouse
  - [ ] Preview rico de conteúdo
  - [ ] Ícones para tipos de conteúdo

### Funcionalidades
- [ ] **Suporte completo a imagens**
  - [ ] Captura de imagens do clipboard
  - [ ] Preview de thumbnails
  - [ ] Exibição em tamanho real
  
- [ ] **Busca no histórico**
  - [ ] Campo de busca no popup
  - [ ] Busca instantânea (fuzzy search)
  - [ ] Filtros por tipo (texto/imagem)

---

## 🎨 V1.2 - Dashboard de Configuração

**Previsão:** Março 2026

### Interface de Configuração

#### 1. Janela Principal de Configurações
```
┌─────────────────────────────────────────┐
│  Clippit - Configurações                │
├─────────────┬───────────────────────────┤
│             │                           │
│  Geral      │  [Conteúdo da seção]     │
│  Atalhos    │                           │
│  Interface  │                           │
│  Privacidade│                           │
│  Avançado   │                           │
│             │                           │
│ ───────────────────────────────         │
│  Sobre      │     [Cancelar] [Salvar]  │
└─────────────┴───────────────────────────┘
```

#### 2. Seção de Atalhos

**Funcionalidades:**
- [x] Editor visual de atalhos
- [ ] Detecção de conflitos com sistema
- [ ] Teste de atalho em tempo real
- [ ] Sugestões de atalhos alternativos
- [ ] Atalhos múltiplos por ação
- [ ] Perfis de atalhos (gamer, produtividade, etc)

**Interface:**
```
┌─────────────────────────────────────────┐
│  Configurar Atalhos                     │
├─────────────────────────────────────────┤
│                                         │
│  Mostrar Histórico:                     │
│  ┌────────────────────┐  [Testar]     │
│  │ Super + V          │  ✓ Funciona   │
│  └────────────────────┘                │
│                                         │
│  ⚠ Conflito detectado com:             │
│     - Sistema: Nenhum                  │
│                                         │
│  Atalho Alternativo (opcional):        │
│  ┌────────────────────┐  [Testar]     │
│  │ Ctrl+Shift + V     │               │
│  └────────────────────┘                │
│                                         │
│  ┌─────────────────────────────────┐  │
│  │ Sugestões:                      │  │
│  │  • Ctrl+Shift+V                 │  │
│  │  • Alt+V                        │  │
│  │  • Ctrl+` (backtick)            │  │
│  └─────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

#### 3. Seção de Interface/Temas

**Funcionalidades:**
- [ ] Seletor visual de temas
- [ ] Preview em tempo real
- [ ] Editor de cores customizado
- [ ] Galeria de temas da comunidade
- [ ] Exportar/importar temas
- [ ] Tema automático (seguir sistema)

**Interface:**
```
┌─────────────────────────────────────────┐
│  Personalizar Interface                 │
├─────────────────────────────────────────┤
│                                         │
│  Tema: [Dark ▼]  [Light]  [Auto]       │
│                                         │
│  Temas Pré-configurados:               │
│  ┌───────┐ ┌───────┐ ┌───────┐        │
│  │ Dark  │ │ Nord  │ │Dracula│        │
│  │ ◉     │ │       │ │       │        │
│  └───────┘ └───────┘ └───────┘        │
│  ┌───────┐ ┌───────┐ ┌───────┐        │
│  │Gruvbox│ │Solar  │ │Custom │        │
│  │       │ │       │ │   +   │        │
│  └───────┘ └───────┘ └───────┘        │
│                                         │
│  Preview:                              │
│  ┌─────────────────────────────────┐  │
│  │ [Demo do popup com tema]        │  │
│  │ • Item 1                        │  │
│  │ • Item 2 (selecionado)          │  │
│  └─────────────────────────────────┘  │
│                                         │
│  Fonte: [Nunito ▼]  Tamanho: [14]     │
│  Opacidade: ████████░░ 90%             │
└─────────────────────────────────────────┘
```

#### 4. Seção de Privacidade

**Funcionalidades:**
- [ ] Lista de apps ignorados
- [ ] Autodetecção de apps sensíveis
- [ ] Adicionar app por seleção de janela
- [ ] Regras por padrão (regex)
- [ ] Modo privado temporário

**Interface:**
```
┌─────────────────────────────────────────┐
│  Privacidade e Segurança               │
├─────────────────────────────────────────┤
│                                         │
│  ☑ Ignorar apps sensíveis              │
│  ☑ Detectar gerenciadores de senha     │
│                                         │
│  Aplicativos Ignorados:                │
│  ┌─────────────────────────────────┐  │
│  │ • keepassxc          [×]        │  │
│  │ • bitwarden          [×]        │  │
│  │ • 1password          [×]        │  │
│  └─────────────────────────────────┘  │
│                                         │
│  [+ Adicionar Manualmente]             │
│  [🎯 Selecionar Janela]                │
│                                         │
│  ☐ Limpar histórico ao sair            │
│  ☐ Criptografar dados sensíveis (V2)  │
└─────────────────────────────────────────┘
```

#### 5. Estatísticas e Insights

**Funcionalidades:**
- [ ] Total de itens copiados
- [ ] Tipos mais copiados
- [ ] Aplicativos mais usados
- [ ] Gráficos de uso por hora/dia
- [ ] Itens mais reutilizados
- [ ] Economia de tempo estimada

**Interface:**
```
┌─────────────────────────────────────────┐
│  Estatísticas de Uso                    │
├─────────────────────────────────────────┤
│                                         │
│  Últimos 30 dias:                      │
│  ┌─────────────────────────────────┐  │
│  │ Total copiado:    1,247 itens   │  │
│  │ Texto:           1,180 (95%)    │  │
│  │ Imagens:            67 (5%)     │  │
│  │ Reutilizações:     342          │  │
│  └─────────────────────────────────┘  │
│                                         │
│  Uso por hora:                         │
│  ▁▂▃▅▇█▇▅▃▂▁▁▂▃▄▅▆▇▆▅▄▃▂▁           │
│                                         │
│  Apps mais usados:                     │
│  1. VS Code        ████████ 45%       │
│  2. Firefox        ████ 23%           │
│  3. Terminal       ██ 15%             │
│                                         │
│  [Exportar Dados] [Limpar Tudo]       │
└─────────────────────────────────────────┘
```

### Comando para Abrir Configurações

```bash
# Via terminal
clippit-config

# Via atalho (configurável)
Super+Shift+C
```

---

## 🚀 V1.3 - Funcionalidades Avançadas

**Previsão:** Abril 2026

### Favoritos e Organização
- [ ] **Fixar itens importantes**
  - [ ] Seção dedicada no popup
  - [ ] Atalho rápido para fixar
  - [ ] Organização por ordem
  
- [ ] **Categorias e Tags**
  - [ ] Auto-categorização (código, URLs, emails)
  - [ ] Tags manuais
  - [ ] Filtros por categoria/tag
  
- [ ] **Coleções**
  - [ ] Agrupar itens relacionados
  - [ ] Compartilhar coleções

### Busca Avançada
- [ ] Filtros combinados
- [ ] Busca por data/hora
- [ ] Busca por aplicativo origem
- [ ] Regex support
- [ ] Busca em imagens (OCR) - experimental

### Atalhos Múltiplos
- [ ] Atalho para busca direta
- [ ] Atalho para último item
- [ ] Atalho para favoritos
- [ ] Atalhos numéricos (Ctrl+1, Ctrl+2, etc)

---

## 🌍 V2.0 - Expansão de Plataforma

**Previsão:** Q3 2026

### Suporte Multi-Plataforma
- [x] **Wayland Support** (✅ Concluído em v1.1)
  - [x] Protocolo wl-clipboard via arboard
  - [x] Wayland nativo com toast notifications
  - [x] Remoção completa de dependências X11
  
- [ ] **Windows** (opcional)
  - [ ] Backend Win32 API
  - [ ] Instalador MSI
  
- [ ] **macOS** (opcional)
  - [ ] Backend NSPasteboard
  - [ ] .dmg installer

### Sincronização Cloud
- [ ] Sync entre dispositivos
- [ ] Providers suportados:
  - [ ] Nextcloud
  - [ ] Own cloud
  - [ ] Syncthing
  - [ ] Custom server
- [ ] End-to-end encryption
- [ ] Resolução de conflitos

### Segurança Avançada
- [ ] Criptografia AES-256
- [ ] Senha mestra
- [ ] Biometria (quando disponível)
- [ ] Auto-lock após inatividade
- [ ] Modo incógnito

### Tipos de Conteúdo
- [ ] **Arquivos**
  - [ ] Caminhos de arquivos
  - [ ] Drag & drop
  
- [ ] **Rich Content**
  - [ ] HTML formatado
  - [ ] Markdown
  - [ ] Código com syntax highlight

### Plugin System
- [ ] API para plugins
- [ ] Plugin marketplace
- [ ] Plugins oficiais:
  - [ ] OCR para imagens
  - [ ] Tradutor
  - [ ] Formatadores (JSON, XML, etc)
  - [ ] QR Code generator

---

## 🔮 V3.0 - Futuro (Ideias)

**Previsão:** 2027+

- [ ] IA integrada
  - [ ] Sugestões inteligentes
  - [ ] Resumo de textos longos
  - [ ] Detecção de padrões
  
- [ ] Mobile companion
  - [ ] App Android/iOS
  - [ ] Sync com desktop
  - [ ] Compartilhamento cross-device
  
- [ ] Colaboração em equipe
  - [ ] Clipboards compartilhados
  - [ ] Permissões granulares
  - [ ] Auditoria de acessos
  
- [ ] Integração com serviços
  - [ ] Note-taking apps (Obsidian, Notion)
  - [ ] Password managers
  - [ ] Translation services
  - [ ] Code formatters

---

## 🎯 Como Contribuir

Quer ajudar a implementar alguma feature? Veja `CONTRIBUTING.md`!

**Áreas que precisam de ajuda:**
1. Interface Qt/QML (V1.1)
2. Dashboard de configuração (V1.2)
3. Suporte Wayland (V2.0)
4. Testes e documentação
5. Design de UI/UX

---

## 📊 Priorização

**Critérios:**
1. ⭐ Impacto no usuário
2. 🔧 Complexidade técnica
3. 🐛 Bugs reportados
4. 💬 Feedback da comunidade

**Processo:**
- Issues no GitHub com labels
- Discussões na comunidade
- Votação de features
- Roadmap atualizado mensalmente
