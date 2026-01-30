# OCR Feature - Clippit

## 🎯 O que é

OCR (Optical Character Recognition) extrai automaticamente texto de imagens capturadas, permitindo buscar conteúdo dentro de screenshots e fotos.

## 🚀 Como funciona

```
1. Você copia uma imagem (print screen, screenshot, foto)
   ↓
2. Clippit salva a imagem normalmente
   ↓
3. Em background, OCR processa a imagem
   ↓
4. Texto extraído é indexado no banco (FTS5)
   ↓
5. Você pode buscar pelo texto e encontrar a imagem!
```

**Exemplo:**
- Você tira print de uma conversa que contém "reunião amanhã às 14h"
- Clippit extrai automaticamente esse texto via OCR
- Depois, você busca "reunião" no popup (`Super+V`)
- O print aparece nos resultados! 🎉

## 📋 Requisitos

### Sistema

```bash
# Ubuntu/Debian
sudo apt-get install tesseract-ocr libtesseract-dev tesseract-ocr-por tesseract-ocr-eng

# Fedora/RHEL
sudo dnf install tesseract tesseract-devel tesseract-langpack-por tesseract-langpack-eng

# Arch Linux
sudo pacman -S tesseract tesseract-data-por tesseract-data-eng
```

### Espaço em Disco
- Tesseract: ~5MB
- Dados de treino (por+eng): ~8MB
- **Total: ~13MB**

### Idiomas Suportados
- ✅ **Português** (por)
- ✅ **Inglês** (eng)
- 🔧 **Configurável**: Adicione mais idiomas via config

## 🛠️ Instalação

### Método 1: Automática (via install.sh)

O script de instalação já verifica e instala Tesseract automaticamente:

```bash
./scripts/install.sh
```

### Método 2: Manual

```bash
# 1. Instalar Tesseract
sudo apt-get update
sudo apt-get install -y tesseract-ocr libtesseract-dev tesseract-ocr-por tesseract-ocr-eng

# 2. Verificar instalação
tesseract --version
# Deve mostrar: tesseract 4.x.x

# 3. Verificar idiomas
tesseract --list-langs
# Deve listar: eng, por

# 4. Recompilar Clippit
cargo build --release

# 5. Reinstalar
./scripts/install.sh
```

## ⚙️ Configuração

### Via Dashboard

1. Abrir Dashboard: `clippit-dashboard`
2. Ir para aba **"General"**
3. Seção **"OCR (Reconhecimento de Texto)"**:
   - **Ativar OCR**: Liga/desliga extração de texto
   - **Idiomas OCR**: Escolher por+eng, por, ou eng

### Via config.toml

Arquivo: `~/.config/clippit/config.toml`

```toml
[features]
enable_ocr = true  # Liga/desliga OCR

[ocr]
languages = "por+eng"  # Idiomas (português + inglês)
timeout_seconds = 5    # Timeout máximo (5s padrão)
```

**Opções de idiomas:**
- `"por+eng"` - Português e Inglês (recomendado)
- `"por"` - Apenas Português
- `"eng"` - Apenas Inglês
- `"por+eng+fra"` - Múltiplos idiomas (requer instalação)

## 🎨 Como Usar

### 1. Capturar Screenshot com Texto

```bash
# Tirar print screen (qualquer método)
Print Screen / Flameshot / Spectacle / etc.

# Ou copiar imagem de qualquer lugar
Ctrl+C em qualquer imagem
```

### 2. OCR Processa Automaticamente

```
[Imagem salva] 
  → [OCR inicia em background]
  → [Tesseract extrai texto]
  → [Texto indexado no FTS5]
  → [Busca disponível!]
```

**Tempo**: ~1-2 segundos por imagem (não bloqueia UI)

### 3. Buscar Texto na Imagem

```bash
# Abrir popup
Super+V

# Digitar texto que estava na imagem
"reunião"
"documento"
"código python"

# Resultados incluem TANTO texto normal QUANTO imagens com OCR!
```

## 📊 Performance

### Benchmarks

| Tamanho Imagem | Tempo OCR | Memória |
|----------------|-----------|---------|
| 800x600 | ~0.8s | ~50MB |
| 1920x1080 | ~1.5s | ~80MB |
| 4K (3840x2160) | ~3.0s | ~150MB |

**Notas:**
- Processa em background (não bloqueia captura)
- Uma imagem por vez (fila automática)
- Timeout de 5s (configurável)

### Limitações

- Funciona melhor com texto **claro e grande**
- Resolução ideal: **300+ DPI**
- Não funciona bem com:
  - Fontes muito estilizadas ou manuscritas
  - Texto muito pequeno (< 10px)
  - Imagens com muito ruído/blur
  - Texto em ângulos extremos

## 🔍 Verificar Status

### Ver Logs de OCR

```bash
# Logs em tempo real
journalctl --user -u clippit -f | grep OCR

# Ver apenas OCR processing
journalctl --user -u clippit | grep "OCR"

# Exemplo de saída:
# 🔍 Starting OCR for: /home/user/.local/share/clippit/images/abc123.png
# ✅ OCR extracted 245 characters
# ✅ OCR text saved for entry 42
```

### Verificar Banco de Dados

```bash
DB="$HOME/.local/share/clippit/history.db"

# Contar imagens com OCR processado
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history WHERE ocr_text IS NOT NULL;"

# Ver exemplos de texto OCR
sqlite3 "$DB" "SELECT id, substr(ocr_text, 1, 50) FROM clipboard_history WHERE ocr_text IS NOT NULL LIMIT 5;"

# Buscar via OCR
sqlite3 "$DB" "SELECT id, image_path FROM clipboard_history_fts WHERE ocr_text MATCH 'reunião';"
```

## 🐛 Troubleshooting

### OCR não está funcionando

**Verificar instalação:**
```bash
# Tesseract instalado?
which tesseract
tesseract --version

# Idiomas instalados?
tesseract --list-langs
# Deve listar: eng, por
```

**Se não estiver instalado:**
```bash
sudo apt-get install tesseract-ocr libtesseract-dev tesseract-ocr-por tesseract-ocr-eng
```

### OCR muito lento

**Causas possíveis:**
- Imagens muito grandes (4K+)
- Sistema com pouco RAM
- CPU antiga

**Soluções:**
- Reduzir `max_image_size_mb` no config
- Imagens serão otimizadas para max 2048px
- Desabilitar OCR se não for necessário

### Texto não está sendo extraído

**Causas possíveis:**
- Texto muito pequeno ou borrado
- Fonte muito estilizada
- Baixo contraste

**Soluções:**
- Usar screenshots com texto claro e grande
- Aumentar resolução da captura
- Usar fontes padrão quando possível

### Idioma errado reconhecido

**Problema:** OCR confunde português com inglês

**Solução:**
```toml
# Priorizar português
[ocr]
languages = "por"  # Apenas português
```

## 🔒 Privacidade

### Dados Locais

**Tudo é processado localmente:**
- ✅ Tesseract roda 100% no seu PC
- ✅ Nenhum dado enviado para internet
- ✅ Texto OCR armazenado apenas no SQLite local
- ✅ Sem APIs externas ou cloud

### Desabilitar para Apps Sensíveis

O OCR **respeita** as configurações de privacidade:

```toml
[privacy]
ignored_apps = ["keepassxc", "bitwarden", "1password"]
```

Imagens de apps bloqueados **não** terão OCR processado.

## 📈 Casos de Uso

### 1. Buscar Screenshots Antigos
- Tirou print de documentos há semanas
- Busque palavras-chave e encontre rapidamente
- Exemplo: buscar "contrato" encontra todos prints de contratos

### 2. Código em Imagens
- Print de código fonte
- Busque funções, variáveis, comentários
- Exemplo: buscar "async fn" encontra prints de código Rust

### 3. Conversas e Mensagens
- Screenshots de WhatsApp, Telegram, Discord
- Busque por nome, mensagem, data
- Exemplo: buscar "João disse" encontra conversas

### 4. Documentos e PDFs
- Screenshot de PDFs ou docs
- Busque termos específicos
- Melhor que salvar arquivo inteiro

## 🔧 Configuração Avançada

### Múltiplos Idiomas

```bash
# Instalar idiomas adicionais
sudo apt-get install tesseract-ocr-spa  # Espanhol
sudo apt-get install tesseract-ocr-fra  # Francês
sudo apt-get install tesseract-ocr-deu  # Alemão
```

```toml
[ocr]
languages = "por+eng+spa"  # Português, Inglês, Espanhol
```

### Ajustar Timeout

```toml
[ocr]
timeout_seconds = 10  # Aumentar para imagens grandes/complexas
```

### Desabilitar Temporariamente

```toml
[features]
enable_ocr = false  # Desabilitar OCR (economizar CPU/RAM)
```

## 📊 Estatísticas

### Ver Estatísticas de OCR

```bash
DB="$HOME/.local/share/clippit/history.db"

# Total de imagens
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history WHERE content_type = 'image';"

# Imagens com OCR
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history WHERE ocr_text IS NOT NULL;"

# Média de caracteres extraídos
sqlite3 "$DB" "SELECT AVG(LENGTH(ocr_text)) FROM clipboard_history WHERE ocr_text IS NOT NULL;"

# Tamanho do índice FTS5
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history_fts WHERE ocr_text IS NOT NULL;"
```

## 🆘 Suporte

### Logs Detalhados

```bash
# Ver logs de OCR
journalctl --user -u clippit | grep -i ocr

# Logs em tempo real
journalctl --user -u clippit -f
```

### Reprocessar OCR

Se OCR falhou ou você mudou idiomas, pode reprocessar:

```bash
# TODO: Implementar comando de reprocessamento
# clippit-daemon --reprocess-ocr
```

## 🔮 Melhorias Futuras

- [ ] **Reprocessar OCR** sob demanda
- [ ] **Confiança do OCR**: Mostrar score de confiabilidade
- [ ] **Pré-processamento**: Melhorar imagem antes de OCR (contraste, nitidez)
- [ ] **Cache**: Evitar reprocessar mesma imagem
- [ ] **Mais idiomas**: Suporte a 100+ idiomas do Tesseract

---

**Versão:** 1.10.0  
**Última atualização:** 2026-01-28  
**Dependência:** Tesseract OCR 4.x+
