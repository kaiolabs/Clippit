//! Leitura e escrita do ~/.config/clippit/corretor.conf (chave = valor),
//! o mesmo arquivo que o `clippit-correct` e a extensão do GNOME leem.

use std::path::PathBuf;

pub fn path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("clippit")
        .join("corretor.conf")
}

pub fn contexts_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("clippit")
        .join("contextos")
}

pub fn get(key: &str) -> Option<String> {
    let text = std::fs::read_to_string(path()).ok()?;
    text.lines().find_map(|line| {
        let line = line.split('#').next()?.trim();
        let (k, v) = line.split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_string())
    })
}

pub fn get_bool(key: &str, default: bool) -> bool {
    match get(key).as_deref() {
        Some("true") | Some("sim") | Some("1") => true,
        Some("false") | Some("nao") | Some("não") | Some("0") => false,
        _ => default,
    }
}

/// Troca (ou acrescenta) uma chave mantendo o resto do arquivo como está.
pub fn set(key: &str, value: &str) {
    let p = path();
    let text = std::fs::read_to_string(&p).unwrap_or_default();
    let mut found = false;
    let mut lines: Vec<String> = text
        .lines()
        .map(|line| {
            let bare = line.split('#').next().unwrap_or("").trim();
            if let Some((k, _)) = bare.split_once('=') {
                if k.trim() == key {
                    found = true;
                    return format!("{key} = {value}");
                }
            }
            line.to_string()
        })
        .collect();
    if !found {
        lines.push(format!("{key} = {value}"));
    }
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Err(e) = std::fs::write(&p, lines.join("\n") + "\n") {
        eprintln!("❌ Erro ao salvar {}: {}", p.display(), e);
    }
}

/// Abre um arquivo ou pasta no app padrão do sistema.
pub fn open(path: &std::path::Path) {
    use gtk::gio::prelude::*;
    let uri = gtk::gio::File::for_path(path).uri();
    if let Err(e) =
        gtk::gio::AppInfo::launch_default_for_uri(&uri, None::<&gtk::gio::AppLaunchContext>)
    {
        eprintln!("❌ Não consegui abrir {}: {}", path.display(), e);
    }
}
