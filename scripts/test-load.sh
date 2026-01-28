#!/bin/bash
# Script de Teste de Carga - Clippit Database
# Insere 1000 textos e 50 imagens para testar performance

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Teste de Carga - Clippit Database"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Verificar se sqlite3 está instalado
if ! command -v sqlite3 &> /dev/null; then
    echo "❌ sqlite3 não está instalado!"
    echo "   Instale com: sudo apt install sqlite3"
    exit 1
fi

# Verificar se ImageMagick está instalado (para criar imagens de teste)
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick não está instalado!"
    echo "   Instale com: sudo apt install imagemagick"
    exit 1
fi

# Caminho do banco
DB_PATH="$HOME/.local/share/clippit/history.db"
IMAGES_DIR="$HOME/.local/share/clippit/images"

echo "📂 Banco de dados: $DB_PATH"

# Verificar se banco existe
if [ ! -f "$DB_PATH" ]; then
    echo "❌ Banco de dados não encontrado!"
    echo "   Execute o daemon primeiro: systemctl --user start clippit"
    exit 1
fi

# Contar itens existentes
COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM clipboard_history;")
echo "📊 Itens existentes: $COUNT"
echo ""

# Perguntar se deseja continuar
echo "⚠️  Este script irá adicionar:"
echo "   • 1000 entradas de texto"
echo "   • 50 imagens com thumbnails"
echo ""
read -p "Continuar? (s/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Ss]$ ]]; then
    echo "❌ Cancelado pelo usuário"
    exit 0
fi

echo ""
echo "🚀 Iniciando inserção..."
echo ""

# Criar diretório de imagens
mkdir -p "$IMAGES_DIR"

# Inserir 1000 textos
echo "📝 Inserindo 1000 textos..."
START=$(date +%s)

for i in $(seq 1 1000); do
    # Gerar texto aleatório
    case $((i % 10)) in
        0) TEXT="Lorem ipsum dolor sit amet #$i" ;;
        1) TEXT="Teste de performance do Clippit - entrada $i" ;;
        2) TEXT="Rust é uma linguagem incrível 🦀 #$i" ;;
        3) TEXT="SQLite FTS5 = busca ultrarrápida ⚡ ($i)" ;;
        4) TEXT="Wayland + GTK4 = Interface moderna #$i" ;;
        5) TEXT="$(printf 'Linha 1: Texto multiplas linhas\nLinha 2: Item %d\nLinha 3: Mais dados' $i)" ;;
        6) TEXT="Performance importa! Item $i" ;;
        7) TEXT="🚀 Otimização é essencial - Test $i" ;;
        8) TEXT="Código limpo = manutenção fácil ($i)" ;;
        *) TEXT="Entrada de teste número $i" ;;
    esac
    
    # Timestamp (espaçados no tempo)
    TIMESTAMP=$(date -u -d "-$i seconds" +"%Y-%m-%dT%H:%M:%S.000000000Z")
    
    # Inserir no banco
    sqlite3 "$DB_PATH" "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, timestamp) VALUES ('text', '$TEXT', NULL, NULL, NULL, NULL, NULL, '$TIMESTAMP');"
    
    # Progresso
    if [ $((i % 100)) -eq 0 ]; then
        echo "   ✅ $i textos inseridos"
    fi
done

END=$(date +%s)
ELAPSED=$((END - START))
RATE=$((1000 / ELAPSED))
echo "   ⏱️  Tempo: ${ELAPSED}s (${RATE} itens/s)"
echo ""

# Inserir 50 imagens
echo "🖼️  Inserindo 50 imagens..."
START=$(date +%s)

for i in $(seq 1 50); do
    # Tamanhos variados
    case $((i % 5)) in
        0) WIDTH=400; HEIGHT=300 ;;
        1) WIDTH=800; HEIGHT=600 ;;
        2) WIDTH=1024; HEIGHT=768 ;;
        3) WIDTH=640; HEIGHT=480 ;;
        *) WIDTH=512; HEIGHT=512 ;;
    esac
    
    # Gerar hash único
    HASH=$(echo "test_image_$i_$(date +%s%N)" | sha256sum | cut -d' ' -f1)
    
    # Criar imagem com padrão
    IMAGE_PATH="$IMAGES_DIR/${HASH}.png"
    THUMB_PATH="/tmp/clippit_thumb_${HASH}.png"
    
    # Gerar cores aleatórias
    COLOR1=$((RANDOM % 256))
    COLOR2=$((RANDOM % 256))
    COLOR3=$((RANDOM % 256))
    
    # Criar imagem colorida com gradiente
    convert -size ${WIDTH}x${HEIGHT} \
        gradient:"rgb($COLOR1,$COLOR2,$COLOR3)"-"rgb($COLOR3,$COLOR1,$COLOR2)" \
        -pointsize 40 -fill white -gravity center \
        -annotate +0+0 "Test #$i\n${WIDTH}x${HEIGHT}" \
        "$IMAGE_PATH" 2>/dev/null
    
    # Criar thumbnail 128x128
    convert "$IMAGE_PATH" -resize 128x128 "$THUMB_PATH" 2>/dev/null
    
    # Timestamp
    TIMESTAMP=$(date -u -d "-$((1000 + i)) seconds" +"%Y-%m-%dT%H:%M:%S.000000000Z")
    
    # Ler thumbnail como BLOB (hex)
    THUMBNAIL_HEX=$(xxd -p "$THUMB_PATH" | tr -d '\n')
    
    # Inserir no banco
    sqlite3 "$DB_PATH" "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, timestamp) VALUES ('image', NULL, NULL, '$IMAGE_PATH', X'$THUMBNAIL_HEX', $WIDTH, $HEIGHT, '$TIMESTAMP');"
    
    # Limpar thumbnail temporário
    rm -f "$THUMB_PATH"
    
    # Progresso
    if [ $((i % 10)) -eq 0 ]; then
        echo "   ✅ $i imagens inseridas"
    fi
done

END=$(date +%s)
ELAPSED=$((END - START))
RATE=$(echo "scale=1; 50 / $ELAPSED" | bc)
echo "   ⏱️  Tempo: ${ELAPSED}s (${RATE} imagens/s)"
echo ""

# Contar total final
FINAL_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM clipboard_history;")
INSERTED=$((FINAL_COUNT - COUNT))

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Teste de Carga Completo!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 Estatísticas:"
echo "   • Itens antes: $COUNT"
echo "   • Itens inseridos: $INSERTED"
echo "   • Total agora: $FINAL_COUNT"
echo ""

# Verificar FTS5
FTS_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM clipboard_history_fts;" 2>/dev/null || echo "0")
echo "📇 Índice FTS5: $FTS_COUNT entradas"
echo ""

echo "🧪 Teste agora:"
echo "   1. Abra o popup: Super+V"
echo "   2. Busque algo (ex: 'teste', 'lorem', 'rust')"
echo "   3. Verifique a performance!"
echo ""
echo "📊 Comandos úteis:"
echo "   • Ver total: sqlite3 $DB_PATH 'SELECT COUNT(*) FROM clipboard_history;'"
echo "   • Ver FTS5: sqlite3 $DB_PATH 'SELECT COUNT(*) FROM clipboard_history_fts;'"
echo "   • Limpar tudo: sqlite3 $DB_PATH 'DELETE FROM clipboard_history;'"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
