#!/bin/bash
# Script de Teste - OCR Feature do Clippit

set -e

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 Teste de OCR - Clippit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 1. Verificar instalação do Tesseract
echo "📦 Verificando Tesseract OCR..."
if ! command -v tesseract &> /dev/null; then
    echo "❌ Tesseract não está instalado!"
    echo ""
    echo "Instale com:"
    echo "  sudo apt-get install tesseract-ocr libtesseract-dev tesseract-ocr-por tesseract-ocr-eng"
    exit 1
fi

echo "✅ Tesseract instalado:"
tesseract --version | head -n 1
echo ""

# 2. Verificar idiomas disponíveis
echo "🌍 Idiomas disponíveis:"
tesseract --list-langs 2>&1 | grep -E "por|eng" || echo "⚠️ Idiomas por/eng não encontrados"
echo ""

# 3. Verificar ImageMagick (para criar imagens de teste)
if ! command -v convert &> /dev/null; then
    echo "⚠️ ImageMagick não está instalado (opcional para teste)"
    echo "   Instale com: sudo apt-get install imagemagick"
    echo ""
    USE_IMAGEMAGICK=false
else
    echo "✅ ImageMagick disponível"
    USE_IMAGEMAGICK=true
    echo ""
fi

# 4. Criar diretório temporário para testes
TEST_DIR="/tmp/clippit-ocr-test"
mkdir -p "$TEST_DIR"
echo "📁 Diretório de teste: $TEST_DIR"
echo ""

# 5. Teste 1: Texto simples em português
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Teste 1: Texto em Português"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$USE_IMAGEMAGICK" = true ]; then
    # Criar imagem com texto em português
    convert -size 800x200 xc:white \
        -font DejaVu-Sans -pointsize 48 -fill black \
        -gravity center -annotate +0+0 "Olá Mundo\nTeste de OCR" \
        "$TEST_DIR/test-pt.png"
    
    echo "📝 Texto esperado: 'Olá Mundo' e 'Teste de OCR'"
    
    # Processar OCR
    tesseract "$TEST_DIR/test-pt.png" "$TEST_DIR/result-pt" -l por 2>/dev/null
    
    echo "✅ Texto extraído:"
    cat "$TEST_DIR/result-pt.txt"
    echo ""
else
    echo "⚠️ ImageMagick não disponível, pulando teste 1"
    echo ""
fi

# 6. Teste 2: Texto em inglês
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Teste 2: Texto em Inglês"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$USE_IMAGEMAGICK" = true ]; then
    # Criar imagem com texto em inglês
    convert -size 800x200 xc:white \
        -font DejaVu-Sans -pointsize 48 -fill black \
        -gravity center -annotate +0+0 "Hello World\nOCR Test" \
        "$TEST_DIR/test-en.png"
    
    echo "📝 Texto esperado: 'Hello World' e 'OCR Test'"
    
    # Processar OCR
    tesseract "$TEST_DIR/test-en.png" "$TEST_DIR/result-en" -l eng 2>/dev/null
    
    echo "✅ Texto extraído:"
    cat "$TEST_DIR/result-en.txt"
    echo ""
else
    echo "⚠️ ImageMagick não disponível, pulando teste 2"
    echo ""
fi

# 7. Teste 3: Bilíngue (por+eng)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Teste 3: Texto Bilíngue (por+eng)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$USE_IMAGEMAGICK" = true ]; then
    # Criar imagem com texto misto
    convert -size 800x300 xc:white \
        -font DejaVu-Sans -pointsize 36 -fill black \
        -gravity north -annotate +0+30 "Reunião Meeting" \
        -gravity center -annotate +0+0 "Documento importante" \
        -gravity south -annotate +0+30 "Critical document" \
        "$TEST_DIR/test-mixed.png"
    
    echo "📝 Texto esperado: 'Reunião Meeting', 'Documento importante', 'Critical document'"
    
    # Processar OCR com ambos idiomas
    tesseract "$TEST_DIR/test-mixed.png" "$TEST_DIR/result-mixed" -l por+eng 2>/dev/null
    
    echo "✅ Texto extraído:"
    cat "$TEST_DIR/result-mixed.txt"
    echo ""
else
    echo "⚠️ ImageMagick não disponível, pulando teste 3"
    echo ""
fi

# 8. Teste 4: Código fonte
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🧪 Teste 4: Código Fonte"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$USE_IMAGEMAGICK" = true ]; then
    # Criar imagem com código
    convert -size 800x400 xc:white \
        -font DejaVu-Sans-Mono -pointsize 24 -fill black \
        -gravity northwest -annotate +20+20 \
"fn main() {
    println!(\"Hello, World!\");
    let x = 42;
    return x;
}" \
        "$TEST_DIR/test-code.png"
    
    echo "📝 Texto esperado: código Rust"
    
    # Processar OCR
    tesseract "$TEST_DIR/test-code.png" "$TEST_DIR/result-code" -l eng 2>/dev/null
    
    echo "✅ Texto extraído:"
    cat "$TEST_DIR/result-code.txt"
    echo ""
else
    echo "⚠️ ImageMagick não disponível, pulando teste 4"
    echo ""
fi

# 9. Verificar integração com Clippit
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔧 Verificando Integração com Clippit"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

DB="$HOME/.local/share/clippit/history.db"

if [ -f "$DB" ]; then
    echo "✅ Banco de dados encontrado: $DB"
    
    # Verificar se coluna ocr_text existe
    if sqlite3 "$DB" "PRAGMA table_info(clipboard_history);" | grep -q "ocr_text"; then
        echo "✅ Coluna ocr_text existe no schema"
    else
        echo "⚠️ Coluna ocr_text NÃO existe (migração pendente)"
    fi
    
    # Verificar FTS5 com ocr_text
    if sqlite3 "$DB" "SELECT sql FROM sqlite_master WHERE name='clipboard_history_fts';" | grep -q "ocr_text"; then
        echo "✅ FTS5 inclui campo ocr_text"
    else
        echo "⚠️ FTS5 não inclui ocr_text (precisa rebuild)"
    fi
    
    # Contar imagens com OCR
    OCR_COUNT=$(sqlite3 "$DB" "SELECT COUNT(*) FROM clipboard_history WHERE ocr_text IS NOT NULL;" 2>/dev/null || echo "0")
    echo "📊 Imagens com OCR processado: $OCR_COUNT"
    
    echo ""
else
    echo "⚠️ Banco de dados não encontrado"
    echo "   Execute o daemon primeiro: systemctl --user start clippit"
    echo ""
fi

# 10. Estatísticas
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Resumo dos Testes"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$USE_IMAGEMAGICK" = true ]; then
    echo "✅ Testes executados: 4/4"
    echo "✅ Imagens criadas: 4"
    echo "✅ Arquivos em: $TEST_DIR"
else
    echo "⚠️ Testes limitados (ImageMagick não disponível)"
fi

echo ""
echo "📁 Arquivos de teste:"
ls -lh "$TEST_DIR" 2>/dev/null || echo "  (nenhum)"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Testes Concluídos!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$USE_IMAGEMAGICK" = true ]; then
    echo "🧪 Como testar OCR integrado:"
    echo ""
    echo "1. Copiar uma das imagens de teste:"
    echo "   wl-copy < $TEST_DIR/test-pt.png"
    echo ""
    echo "2. Aguardar processamento (~2s)"
    echo ""
    echo "3. Ver logs do daemon:"
    echo "   journalctl --user -u clippit -f | grep OCR"
    echo ""
    echo "4. Buscar o texto extraído:"
    echo "   Super+V e buscar 'Olá' ou 'Hello'"
    echo ""
    echo "5. Verificar banco de dados:"
    echo "   sqlite3 $DB \"SELECT ocr_text FROM clipboard_history WHERE ocr_text IS NOT NULL LIMIT 5;\""
fi

echo ""
echo "🗑️  Para limpar arquivos de teste:"
echo "   rm -rf $TEST_DIR"
echo ""
