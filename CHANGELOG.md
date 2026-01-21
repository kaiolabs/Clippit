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
