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
/// Ler, trocar e gravar acontece com o arquivo trancado (vale entre processos: duas janelas do painel
/// salvando ao mesmo tempo não apagam a troca uma da outra) e a gravação é atômica (a extensão e os
/// scripts nunca leem o arquivo pela metade).
pub fn set(key: &str, value: &str) {
    let p = path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // Se o arquivo for um link (ex.: gerenciado por outra ferramenta), troca o destino e não o link.
    let p = std::fs::canonicalize(&p).unwrap_or(p);
    // O cadeado fica num arquivo ao lado: o próprio corretor.conf é trocado por outro a cada gravação.
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(p.with_extension("conf.lock"))
        .ok();
    if let Some(l) = &lock {
        if let Err(e) = l.lock() {
            eprintln!("⚠️  Não consegui trancar {}: {}", p.display(), e);
        }
    }

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
    let tmp = p.with_extension(format!("conf.tmp{}", std::process::id()));
    let saved =
        std::fs::write(&tmp, lines.join("\n") + "\n").and_then(|_| std::fs::rename(&tmp, &p));
    if let Err(e) = saved {
        let _ = std::fs::remove_file(&tmp);
        eprintln!("❌ Erro ao salvar {}: {}", p.display(), e);
    }
    // O cadeado solta quando `lock` sai de escopo.
    drop(lock);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Várias gravações ao mesmo tempo (como duas janelas do painel) não podem perder nenhuma chave.
    #[test]
    fn concurrent_sets_keep_every_key() {
        let dir = std::env::temp_dir().join(format!("clippit-conf-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_CONFIG_HOME", &dir);

        let handles: Vec<_> = (0..8)
            .map(|t| {
                std::thread::spawn(move || {
                    for i in 0..25 {
                        set(&format!("atalho_t{t}_{i}"), "<Alt>x");
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }

        let text = std::fs::read_to_string(path()).unwrap();
        for t in 0..8 {
            for i in 0..25 {
                assert!(
                    text.contains(&format!("atalho_t{t}_{i} = <Alt>x")),
                    "faltou atalho_t{t}_{i}"
                );
            }
        }
        // Nenhum arquivo temporário sobrou.
        let leftovers = std::fs::read_dir(dir.join("clippit"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .count();
        assert_eq!(leftovers, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
