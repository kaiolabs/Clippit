use arboard::{Clipboard, ImageData};
use clippit_core::Config;
use std::cell::RefCell;
use std::collections::HashMap;

/// Caminho/dimensões das imagens já listadas no popup (vem da consulta de metadados).
/// Permite copiar uma imagem sem pedir ao daemon a imagem inteira via IPC
/// (que chega como JSON de vários MB e era descartada em seguida).
struct ImageInfo {
    path: String,
    dims: Option<(u32, u32)>,
}

thread_local! {
    static IMAGE_CACHE: RefCell<HashMap<i64, ImageInfo>> = RefCell::new(HashMap::new());
    /// Instância única do arboard: o popup é residente, então mantê-la viva evita
    /// recriar a conexão a cada cópia e mantém a posse do clipboard
    static ARBOARD: RefCell<Option<Clipboard>> = const { RefCell::new(None) };
}

fn with_arboard<T>(f: impl FnOnce(&mut Clipboard) -> Result<T, String>) -> Result<T, String> {
    ARBOARD.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot =
                Some(Clipboard::new().map_err(|e| format!("Erro ao acessar clipboard: {}", e))?);
        }
        f(slot.as_mut().unwrap())
    })
}

/// Caminho e dimensões de uma imagem já listada (para o modo ampliado)
pub fn image_info(entry_id: i64) -> Option<(String, Option<(u32, u32)>)> {
    IMAGE_CACHE.with(|c| c.borrow().get(&entry_id).map(|i| (i.path.clone(), i.dims)))
}

/// Registra o caminho de uma entrada de imagem exibida na lista
pub fn remember_entry(entry: &clippit_ipc::HistoryEntry) {
    if let (clippit_ipc::ContentType::Image, Some(path)) = (&entry.content_type, &entry.image_path)
    {
        let dims = entry.image_width.zip(entry.image_height);
        IMAGE_CACHE.with(|c| {
            c.borrow_mut().insert(
                entry.id,
                ImageInfo {
                    path: path.clone(),
                    dims,
                },
            );
        });
    }
}

/// Copia uma entrada para o clipboard e mostra notificação do sistema.
///
/// - texto: arboard (instância mantida viva no processo residente);
/// - imagens PNG: `wl-copy` com os bytes do arquivo, sem decodificar/re-encodar
///   (fallback: arboard). Obs.: o clipboard do GTK foi testado, mas o GNOME ignora a
///   cópia quando não vem de um evento de input recente, então não é confiável aqui.
pub fn copy_to_clipboard(entry_id: i64) -> bool {
    let start = std::time::Instant::now();
    let show_notifications = Config::load().unwrap_or_default().ui.show_notifications;

    // ⚡ Imagem já conhecida pela lista: pula o IPC
    let cached = IMAGE_CACHE.with(|c| c.borrow().get(&entry_id).map(|i| (i.path.clone(), i.dims)));
    let result = if let Some((path, dims)) = cached {
        copy_image_file(&path, dims)
    } else {
        match clippit_ipc::IpcClient::get_entry_data(entry_id) {
            Ok(entry) => match entry.content_type {
                clippit_ipc::ContentType::Text => match &entry.content_text {
                    Some(text) => with_arboard(|cb| {
                        cb.set_text(text.as_str())
                            .map_err(|e| format!("Erro ao copiar texto: {}", e))?;
                        let preview = if text.chars().count() > 80 {
                            format!("{}...", text.chars().take(80).collect::<String>())
                        } else {
                            text.clone()
                        };
                        Ok(format!("Copiado: {}", preview))
                    }),
                    None => Err("Entrada sem conteúdo".to_string()),
                },
                clippit_ipc::ContentType::Image => match &entry.image_path {
                    Some(path) => copy_image_file(path, entry.image_width.zip(entry.image_height)),
                    None => Err("Imagem sem caminho".to_string()),
                },
            },
            Err(e) => Err(format!("Erro: {}", e)),
        }
    };

    match result {
        Ok(msg) => {
            eprintln!("✅ Entrada {} copiada em {:?}", entry_id, start.elapsed());
            show_notification("Clippit", &msg, show_notifications);
            true
        }
        Err(msg) => {
            eprintln!("❌ Falha ao copiar entrada {}: {}", entry_id, msg);
            show_notification("Erro", &msg, show_notifications);
            false
        }
    }
}

fn copy_image_file(path: &str, dims: Option<(u32, u32)>) -> Result<String, String> {
    let dims_str = dims
        .map(|(w, h)| format!(" ({}x{})", w, h))
        .unwrap_or_default();

    if is_png(path) && copy_png_with_wl_copy(path) {
        return Ok(format!("Imagem copiada{}", dims_str));
    }

    // Fallback: decodifica e copia via arboard (mais lento em imagens grandes)
    let img = image::open(path).map_err(|e| format!("Erro ao carregar imagem: {}", e))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());
    with_arboard(|cb| {
        cb.set_image(ImageData {
            width: w as usize,
            height: h as usize,
            bytes: rgba.into_raw().into(),
        })
        .map_err(|e| format!("Erro ao copiar imagem: {}", e))
    })?;
    Ok(format!("Imagem copiada ({}x{})", w, h))
}

fn is_png(path: &str) -> bool {
    use std::io::Read;
    let mut magic = [0u8; 8];
    std::fs::File::open(path)
        .and_then(|mut f| f.read_exact(&mut magic))
        .map(|_| &magic == b"\x89PNG\r\n\x1a\n")
        .unwrap_or(false)
}

/// Copia o PNG cru via `wl-copy` (funciona no GNOME, que não tem data-control).
/// O wl-copy fica em background servindo o clipboard; o processo pai sai logo.
fn copy_png_with_wl_copy(path: &str) -> bool {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return false;
    }
    let Ok(file) = std::fs::File::open(path) else {
        return false;
    };
    match std::process::Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(file)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
    {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("⚠️  wl-copy saiu com {:?}, usando fallback", status);
            false
        }
        Err(e) => {
            eprintln!("ℹ️  wl-copy indisponível ({}), usando fallback", e);
            false
        }
    }
}

/// Notificação do sistema via notify-send, sem bloquear a UI
fn show_notification(summary: &str, body: &str, enabled: bool) {
    if !enabled {
        return;
    }
    match std::process::Command::new("notify-send")
        .args([
            "-i",
            "edit-copy",
            "-t",
            "3000",
            "-a",
            "Clippit",
            "-u",
            "normal",
        ])
        .arg(summary)
        .arg(body)
        .spawn()
    {
        // Espera numa thread para não deixar processo zumbi (popup é residente)
        Ok(mut child) => {
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => eprintln!("❌ Falha ao executar notify-send: {}", e),
    }
}
