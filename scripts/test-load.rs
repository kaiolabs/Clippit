#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! rusqlite = "0.32"
//! chrono = "0.4"
//! image = "0.25"
//! rand = "0.8"
//! sha2 = "0.10"
//! ```

use chrono::{DateTime, Duration, Utc};
use image::{ImageBuffer, Rgba, RgbaImage};
use rand::Rng;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::io::Cursor;
use std::path::PathBuf;

fn main() {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧪 Teste de Carga - Clippit Database");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Abrir banco de dados
    let db_path = get_db_path();
    println!("📂 Banco de dados: {}", db_path.display());
    
    let conn = Connection::open(&db_path).expect("Erro ao abrir banco");
    
    // Contar itens existentes
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    
    println!("📊 Itens existentes: {}", count);
    println!();
    
    // Perguntar se deseja continuar
    println!("⚠️  Este script irá adicionar:");
    println!("   • 1000 entradas de texto");
    println!("   • 50 imagens com thumbnails");
    println!();
    print!("Continuar? (s/N): ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    
    if !input.trim().eq_ignore_ascii_case("s") {
        println!("❌ Cancelado pelo usuário");
        return;
    }
    
    println!();
    println!("🚀 Iniciando inserção...");
    println!();

    // Criar diretório de imagens se não existir
    let images_dir = get_images_dir();
    std::fs::create_dir_all(&images_dir).expect("Erro ao criar diretório de imagens");

    // Inserir 1000 textos
    println!("📝 Inserindo 1000 textos...");
    let start = std::time::Instant::now();
    
    for i in 0..1000 {
        let text = generate_random_text(i);
        let timestamp = Utc::now() - Duration::seconds(i as i64);
        
        insert_text_entry(&conn, &text, timestamp);
        
        if (i + 1) % 100 == 0 {
            println!("   ✅ {} textos inseridos", i + 1);
        }
    }
    
    let elapsed = start.elapsed();
    println!("   ⏱️  Tempo: {:.2}s ({:.0} itens/s)", 
        elapsed.as_secs_f64(),
        1000.0 / elapsed.as_secs_f64()
    );
    println!();

    // Inserir 50 imagens
    println!("🖼️  Inserindo 50 imagens...");
    let start = std::time::Instant::now();
    
    for i in 0..50 {
        let (image_data, width, height) = generate_random_image(i);
        let thumbnail = create_thumbnail(&image_data, 128);
        let timestamp = Utc::now() - Duration::seconds(1000 + i as i64);
        
        // Calcular hash
        let mut hasher = Sha256::new();
        hasher.update(&image_data);
        let hash = format!("{:x}", hasher.finalize());
        
        // Salvar imagem
        let image_path = save_image(&images_dir, &hash, &image_data);
        
        insert_image_entry(&conn, &image_path, &thumbnail, width, height, timestamp);
        
        if (i + 1) % 10 == 0 {
            println!("   ✅ {} imagens inseridas", i + 1);
        }
    }
    
    let elapsed = start.elapsed();
    println!("   ⏱️  Tempo: {:.2}s ({:.1} imagens/s)", 
        elapsed.as_secs_f64(),
        50.0 / elapsed.as_secs_f64()
    );
    println!();

    // Contar total final
    let final_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ Teste de Carga Completo!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("📊 Estatísticas:");
    println!("   • Itens antes: {}", count);
    println!("   • Itens inseridos: {}", final_count - count);
    println!("   • Total agora: {}", final_count);
    println!();
    println!("🧪 Teste agora:");
    println!("   1. Abra o popup: Super+V");
    println!("   2. Busque algo (ex: 'teste', 'lorem')");
    println!("   3. Verifique a performance!");
    println!();
    println!("📊 Ver estatísticas do banco:");
    println!("   sqlite3 {} \"SELECT COUNT(*) FROM clipboard_history;\"", db_path.display());
    println!("   sqlite3 {} \"SELECT COUNT(*) FROM clipboard_history_fts;\"", db_path.display());
}

fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("clippit");
    std::fs::create_dir_all(&path).ok();
    path.push("history.db");
    path
}

fn get_images_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("clippit");
    path.push("images");
    path
}

fn generate_random_text(index: usize) -> String {
    let mut rng = rand::thread_rng();
    
    let templates = vec![
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit",
        "Teste de performance do Clippit com múltiplas entradas",
        "Rust é uma linguagem de programação incrível para sistemas",
        "O histórico de clipboard nunca foi tão eficiente",
        "SQLite FTS5 proporciona busca ultrarrápida",
        "Wayland + GTK4 = Interface moderna e fluida",
        "Código limpo e bem documentado é essencial",
        "Performance importa em aplicações do dia a dia",
        "Testes de carga revelam gargalos antes dos usuários",
        "Otimização prematura é a raiz de todos os males",
    ];
    
    let template = templates[rng.gen_range(0..templates.len())];
    
    // Adicionar variação
    match rng.gen_range(0..5) {
        0 => format!("{} #{}", template, index),
        1 => format!("[{}] {}", index, template),
        2 => format!("{}\nLinha 2: Informação adicional\nLinha 3: Mais dados", template),
        3 => format!("🚀 {} - Item {}", template, index),
        _ => format!("{} (entrada {})", template, index),
    }
}

fn generate_random_image(index: usize) -> (Vec<u8>, u32, u32) {
    let mut rng = rand::thread_rng();
    
    // Tamanhos variados
    let sizes = vec![
        (400, 300),
        (800, 600),
        (1024, 768),
        (640, 480),
        (512, 512),
    ];
    
    let (width, height) = sizes[index % sizes.len()];
    
    // Criar imagem com padrão aleatório
    let mut img: RgbaImage = ImageBuffer::new(width, height);
    
    let base_color = (
        rng.gen_range(0..256) as u8,
        rng.gen_range(0..256) as u8,
        rng.gen_range(0..256) as u8,
    );
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = base_color.0.wrapping_add((x / 10) as u8);
        let g = base_color.1.wrapping_add((y / 10) as u8);
        let b = base_color.2.wrapping_add(((x + y) / 20) as u8);
        *pixel = Rgba([r, g, b, 255]);
    }
    
    // Adicionar texto na imagem
    // (simplificado - apenas um retângulo como marcador)
    for x in 10..100 {
        for y in 10..30 {
            if x < width && y < height {
                img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
    
    // Converter para PNG
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Erro ao codificar PNG");
    
    (buf, width, height)
}

fn create_thumbnail(data: &[u8], size: u32) -> Vec<u8> {
    let img = image::load_from_memory(data).expect("Erro ao carregar imagem");
    let thumbnail = img.resize(size, size, image::imageops::FilterType::Lanczos3);
    
    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    thumbnail
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("Erro ao criar thumbnail");
    
    buf
}

fn save_image(dir: &PathBuf, hash: &str, data: &[u8]) -> String {
    let filename = format!("{}.png", hash);
    let mut path = dir.clone();
    path.push(&filename);
    
    std::fs::write(&path, data).expect("Erro ao salvar imagem");
    
    path.to_string_lossy().to_string()
}

fn insert_text_entry(conn: &Connection, text: &str, timestamp: DateTime<Utc>) {
    let timestamp_str = timestamp.to_rfc3339();
    
    conn.execute(
        "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, timestamp)
         VALUES ('text', ?1, NULL, NULL, NULL, NULL, NULL, ?2)",
        params![text, timestamp_str],
    )
    .expect("Erro ao inserir texto");
}

fn insert_image_entry(
    conn: &Connection,
    image_path: &str,
    thumbnail: &[u8],
    width: u32,
    height: u32,
    timestamp: DateTime<Utc>,
) {
    let timestamp_str = timestamp.to_rfc3339();
    
    conn.execute(
        "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, timestamp)
         VALUES ('image', NULL, NULL, ?1, ?2, ?3, ?4, ?5)",
        params![image_path, thumbnail, width as i64, height as i64, timestamp_str],
    )
    .expect("Erro ao inserir imagem");
}
