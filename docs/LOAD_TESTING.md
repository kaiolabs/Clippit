# Teste de Carga - Clippit Database

Este documento explica como testar a performance do Clippit com grandes volumes de dados.

## 🎯 Objetivo

Verificar que o Clippit funciona bem com:
- 1000+ entradas de texto
- 50+ imagens com thumbnails
- Busca rápida com FTS5
- UI responsiva

## 🚀 Como Executar

### Opção 1: Script Bash (Recomendado)

```bash
# Executar teste de carga
./scripts/test-load.sh
```

**Requisitos:**
- `sqlite3` instalado
- `imagemagick` instalado (para gerar imagens)

**O que faz:**
- ✅ Insere 1000 textos variados
- ✅ Insere 50 imagens de diferentes tamanhos
- ✅ Cria thumbnails automaticamente
- ✅ Armazena dimensões no banco
- ✅ Timestamps espaçados no tempo
- ✅ Mostra estatísticas de performance

### Opção 2: Script Rust

```bash
# Executar via rust-script (requer rust-script)
cargo install rust-script
rust-script scripts/test-load.rs
```

## 📊 Dados Inseridos

### Textos (1000 entradas)
- Variados: Lorem ipsum, textos técnicos, multilinha
- Timestamps: Distribuídos nos últimos 1000 segundos
- Conteúdo: Inclui emojis, números, caracteres especiais

### Imagens (50 entradas)
- Tamanhos: 400x300, 800x600, 1024x768, 640x480, 512x512
- Formato: PNG
- Thumbnails: 128x128 pré-gerados
- Dimensões: Armazenadas no banco (image_width, image_height)
- Padrões: Gradientes coloridos com texto identificador

## 🧪 Como Testar Performance

### 1. Abertura do Popup

```bash
# Executar teste de carga
./scripts/test-load.sh

# Após inserção, testar abertura
# Pressione: Super+V
```

**Esperado:**
- ✅ Popup abre em < 1 segundo
- ✅ Primeiros 100 itens carregados instantaneamente
- ✅ Scroll fluido

### 2. Busca

```bash
# Abrir popup: Super+V
# Digitar na busca: "teste"
```

**Esperado:**
- ✅ Resultados instantâneos (< 50ms)
- ✅ Máximo 100 resultados exibidos
- ✅ Highlighting de busca funciona

### 3. Busca com Imagens

```bash
# Buscar: "test" (vai pegar imagens também)
```

**Esperado:**
- ✅ Imagens renderizadas com thumbnails
- ✅ Dimensões corretas exibidas
- ✅ Sem necessidade de carregar imagem completa

## 📈 Benchmarks Esperados

### Com FTS5 (Otimizado)

| Operação | 100 itens | 300 itens | 1000 itens |
|----------|-----------|-----------|------------|
| Abrir popup | 0.3s | 0.8s | 1.2s |
| Buscar texto | 5ms | 10ms | 20ms |
| Scroll | Fluido | Fluido | Fluido |

### Sem FTS5 (LIKE tradicional)

| Operação | 100 itens | 300 itens | 1000 itens |
|----------|-----------|-----------|------------|
| Abrir popup | 0.5s | 5s | 10s+ |
| Buscar texto | 50ms | 200ms | 1000ms |
| Scroll | Fluido | Lento | Travado |

## 🔍 Verificar Performance

### Ver Estatísticas do Banco

```bash
DB="$HOME/.local/share/clippit/history.db"

# Total de itens
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history;"

# Índice FTS5
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history_fts;"

# Por tipo
sqlite3 "$DB" "SELECT content_type, COUNT(*) FROM clipboard_history GROUP BY content_type;"

# Tamanho do banco
du -h "$DB"

# Tamanho das imagens
du -sh "$HOME/.local/share/clippit/images/"
```

### Ver Logs do Daemon

```bash
# Logs em tempo real
journalctl --user -u clippit -f

# Filtrar por busca
journalctl --user -u clippit | grep -i "search"

# Ver performance
journalctl --user -u clippit | grep -i "returned.*results"
```

## 🧹 Limpar Dados de Teste

### Limpar TUDO

```bash
# CUIDADO: Remove todo o histórico!
sqlite3 "$HOME/.local/share/clippit/history.db" "DELETE FROM clipboard_history;"

# Limpar imagens
rm -rf "$HOME/.local/share/clippit/images/"
mkdir -p "$HOME/.local/share/clippit/images/"

# Reiniciar daemon
systemctl --user restart clippit
```

### Limpar Apenas Itens de Teste

```bash
# Remover textos que contêm "teste" ou "Test"
sqlite3 "$HOME/.local/share/clippit/history.db" \
  "DELETE FROM clipboard_history WHERE content_text LIKE '%teste%' OR content_text LIKE '%Test%';"

# Remover imagens de teste (apenas se nomeadas com hash específico)
# (Mais seguro: limpar manualmente)
```

## 📊 Análise de Performance

### 1. Testar Busca FTS5

```bash
DB="$HOME/.local/share/clippit/history.db"

# Busca com FTS5 (rápida)
time sqlite3 "$DB" "
SELECT COUNT(*) FROM clipboard_history h
INNER JOIN clipboard_history_fts fts ON h.id = fts.rowid
WHERE fts.content_text MATCH 'teste';
"

# Busca com LIKE (lenta)
time sqlite3 "$DB" "
SELECT COUNT(*) FROM clipboard_history
WHERE content_text LIKE '%teste%';
"
```

### 2. Verificar Cache

```bash
# Ver queries lentas no log
journalctl --user -u clippit | grep -A 5 "slow\|timeout"

# Monitorar uso de memória
ps aux | grep clippit-daemon
```

### 3. Profile do Banco

```bash
# Analisar plano de query
sqlite3 "$HOME/.local/share/clippit/history.db" <<EOF
EXPLAIN QUERY PLAN
SELECT h.* FROM clipboard_history h
INNER JOIN clipboard_history_fts fts ON h.id = fts.rowid
WHERE fts.content_text MATCH 'teste';
EOF
```

## 🐛 Problemas Comuns

### Popup Lento Mesmo com FTS5

**Possível causa:** Índice FTS5 não foi populado

```bash
# Verificar se FTS5 está vazio
sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history_fts;"

# Se retornar 0, rebuild o índice
sqlite3 "$DB" "
INSERT INTO clipboard_history_fts(rowid, content_text)
SELECT id, content_text FROM clipboard_history
WHERE content_text IS NOT NULL;
"
```

### Imagens Não Aparecem

**Possível causa:** Arquivo de imagem foi deletado

```bash
# Verificar imagens órfãs
sqlite3 "$DB" "
SELECT id, image_path FROM clipboard_history
WHERE content_type = 'image'
  AND image_path IS NOT NULL
" | while read -r id path; do
    if [ ! -f "$path" ]; then
        echo "❌ Missing: $path (entry $id)"
    fi
done
```

### Banco de Dados Corrompido

```bash
# Verificar integridade
sqlite3 "$DB" "PRAGMA integrity_check;"

# Se corrompido, backup e recrear
cp "$DB" "$DB.backup"
sqlite3 "$DB" ".dump" | sqlite3 "$DB.new"
mv "$DB.new" "$DB"
```

## 📈 Métricas de Sucesso

### ✅ Performance Aceitável

- Popup abre em < 2s com 1000 itens
- Busca retorna em < 100ms
- Scroll fluido sem travamentos
- Uso de memória < 100MB

### ⚠️ Performance Ruim (Requer investigação)

- Popup demora > 5s
- Busca demora > 500ms
- UI trava ao scrollar
- Uso de memória > 500MB

## 🔧 Otimizações Adicionais

Se performance ainda estiver ruim:

1. **Limpar histórico antigo**
   ```bash
   # Manter apenas últimos 30 dias
   sqlite3 "$DB" "
   DELETE FROM clipboard_history
   WHERE timestamp < datetime('now', '-30 days');
   "
   ```

2. **VACUUM banco**
   ```bash
   # Compactar banco
   sqlite3 "$DB" "VACUUM;"
   ```

3. **Rebuild FTS5**
   ```bash
   # Recriar índice do zero
   sqlite3 "$DB" "
   DELETE FROM clipboard_history_fts;
   INSERT INTO clipboard_history_fts(rowid, content_text)
   SELECT id, content_text FROM clipboard_history
   WHERE content_text IS NOT NULL;
   "
   ```

## 📝 Notas

- Script de teste é seguro: não remove dados existentes
- Imagens geradas são padrões coloridos (não imagens reais)
- Thumbnails são gerados no momento da inserção
- Timestamps são retroativos para simular histórico real
- FTS5 deve sincronizar automaticamente via triggers

---

**Versão:** 1.9.6  
**Última atualização:** 2026-01-28
