# Autocomplete Global do Sistema - Clippit

## Visão Geral

O Clippit agora inclui um sistema de autocomplete global que funciona em **qualquer aplicação** do sistema, similar ao autocomplete de smartphones.

### Como Funciona

1. **IBus Engine**: Captura sua digitação em tempo real via Input Method
2. **Análise Inteligente**: Busca no histórico do clipboard por palavras/frases similares
3. **Popup Flutuante**: Mostra sugestões próximas ao cursor
4. **Completar**: Pressione Tab ou Enter para aceitar, ↑↓ para navegar

## Instalação

### 1. Compilar e Instalar o IBus Component

```bash
sudo bash scripts/install-ibus.sh
```

Este script:
- Compila o `clippit-ibus` engine
- Instala em `/usr/local/bin/`
- Registra no IBus
- Reinicia o IBus daemon

### 2. Ativar no Sistema

**GNOME Settings:**
1. Abra **Configurações** → **Teclado**
2. Vá em **Fontes de Entrada** (Input Sources)
3. Clique em **+** para adicionar
4. Procure por **"Clippit Autocomplete"**
5. Adicione à lista

**Linha de Comando:**
```bash
# Listar engines disponíveis
ibus list-engine

# Adicionar Clippit
gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us'), ('ibus', 'clippit')]"
```

### 3. Configurar no Dashboard

Abra o Dashboard do Clippit:
```bash
clippit-dashboard
```

Vá na aba **"Autocompletar Global"** e configure:
- ✅ Habilitar autocomplete
- 🔢 Número de sugestões (1-10)
- ⏱️ Delay antes de mostrar (ms)
- 🔤 Caracteres mínimos
- 📱 Apps ignorados

## Uso

### Digitação Normal

Digite normalmente em qualquer aplicação:

```
su[sugestões aparecem]
↓ navegar
Tab completar
```

### Teclas de Atalho

- **Tab**: Aceita sugestão selecionada
- **Enter**: Aceita sugestão OU insere quebra de linha
- **↑↓**: Navega entre sugestões
- **Esc**: Fecha popup de sugestões

### Apps Ignorados Padrão

Por segurança, o autocomplete é automaticamente desabilitado em:
- Gerenciadores de senha (KeePassXC, Bitwarden, 1Password)
- Terminais (gnome-terminal, tilix)
- Campos marcados como "password"

Você pode adicionar/remover apps nas configurações.

## Configuração Avançada

### Arquivo de Configuração

`~/.config/clippit/config.toml`:

```toml
[autocomplete]
enabled = true
max_suggestions = 3
min_chars = 2
delay_ms = 300
show_in_passwords = false
ignored_apps = ["gnome-terminal", "keepassxc", "bitwarden"]

[autocomplete.ai]  # Fase 2 - Futuro
enabled = false
provider = "local"
model = "gpt-4"
api_key = ""
```

### Performance

O autocomplete usa:
- **Cache em memória**: 1000 palavras mais frequentes
- **Busca assíncrona**: Não bloqueia digitação
- **Debounce**: 300ms de delay antes de buscar
- **Índice FTS5**: Busca rápida no SQLite (futuro)

## Troubleshooting

### Autocomplete não aparece

1. Verifique se IBus está rodando:
   ```bash
   ps aux | grep ibus
   ```

2. Verifique se Clippit está na lista de input sources:
   ```bash
   gsettings get org.gnome.desktop.input-sources sources
   ```

3. Verifique logs do engine:
   ```bash
   RUST_LOG=debug clippit-ibus
   ```

### Conflito com outros Input Methods

Se você usa outros IMEs (chinês, japonês, etc.), você pode ter múltiplos inputs ativos. Use a tecla `Super+Space` para alternar entre eles.

### Sugestões não relevantes

- Aumente `min_chars` para 3 ou 4 caracteres
- Ajuste a lista de apps ignorados
- O histórico é baseado no clipboard - quanto mais você usa, melhor fica

## Desinstalar

```bash
# Remover binário
sudo rm /usr/local/bin/clippit-ibus

# Remover component definition
sudo rm /usr/share/ibus/component/clippit.xml

# Remover do GNOME
gsettings set org.gnome.desktop.input-sources sources "[('xkb', 'us')]"

# Reiniciar IBus
ibus restart
```

## Fase 2 - IA (Futuro)

Em desenvolvimento:
- Sugestões contextuais via IA local ou API
- Aprendizado de padrões de digitação
- Snippets personalizados
- Sincronização entre dispositivos

## Contribuindo

Este é um projeto open-source. Contribuições são bem-vindas!

- Reportar bugs: GitHub Issues
- Sugerir features: GitHub Discussions
- Código: Pull Requests

## Licença

MIT License - Ver arquivo LICENSE no repositório.
