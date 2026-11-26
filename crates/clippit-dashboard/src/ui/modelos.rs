//! Modelos da IA: qual roda na placa de vídeo e qual no processador, baixar
//! os do catálogo (com progresso) e instalar o motor (llama.cpp) se faltar.
//! Quem sabe de tudo é o `clippit-correct --modelos` (JSON).

use super::conf;
use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;

fn corrector() -> std::path::PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".local/bin/clippit-correct")
}

pub fn info() -> Value {
    std::process::Command::new(corrector())
        .arg("--modelos")
        .output()
        .ok()
        .and_then(|o| serde_json::from_slice(&o.stdout).ok())
        .unwrap_or(Value::Null)
}

/// Placas que o llama.cpp enxerga nesta máquina (nvidia, amd, intel).
pub fn gpus(info: &Value) -> Vec<String> {
    info["placas"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// (nome para mostrar, caminho) dos modelos de texto já baixados.
fn local_models(info: &Value) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for m in info["catalogo"].as_array().into_iter().flatten() {
        let papel = m["papel"].as_str().unwrap_or("");
        if m["baixado"].as_bool() == Some(true) && (papel.contains("gpu") || papel.contains("cpu"))
        {
            out.push((
                format!("{} · {} MB", m["nome"].as_str().unwrap_or(""), m["mb"]),
                m["caminho"].as_str().unwrap_or("").into(),
            ));
        }
    }
    for m in info["outros"].as_array().into_iter().flatten() {
        out.push((
            format!("{} · {} MB", m["arquivo"].as_str().unwrap_or(""), m["mb"]),
            m["caminho"].as_str().unwrap_or("").into(),
        ));
    }
    out
}

/// Roda o clippit-correct e entrega cada linha da saída (para o progresso).
fn stream(args: &[&str], on_line: impl Fn(&str) + 'static, on_end: impl Fn(bool) + 'static) {
    let exe = corrector();
    let mut argv: Vec<&std::ffi::OsStr> = vec![exe.as_os_str()];
    argv.extend(args.iter().map(|a| std::ffi::OsStr::new(*a)));
    let proc = match gtk::gio::Subprocess::newv(&argv, gtk::gio::SubprocessFlags::STDOUT_PIPE) {
        Ok(p) => p,
        Err(_) => return on_end(false),
    };
    let input = gtk::gio::DataInputStream::new(&proc.stdout_pipe().unwrap());
    let on_line = Rc::new(on_line);
    let on_end = Rc::new(on_end);
    fn next(
        input: gtk::gio::DataInputStream,
        on_line: Rc<dyn Fn(&str)>,
        on_end: Rc<dyn Fn(bool)>,
        ok: bool,
    ) {
        let i2 = input.clone();
        input.read_line_utf8_async(
            gtk::glib::Priority::DEFAULT,
            gtk::gio::Cancellable::NONE,
            move |res| match res {
                Ok(Some(line)) => {
                    let ok = ok || line.starts_with("OK ");
                    on_line(&line);
                    next(i2, on_line, on_end, ok);
                }
                _ => on_end(ok),
            },
        );
    }
    next(input, on_line, on_end, false);
}

fn fill(combo: &adw::ComboRow, models: &[(String, String)], key: &str, chosen: &str) {
    let names: Vec<&str> = models.iter().map(|m| m.0.as_str()).collect();
    combo.set_model(Some(&gtk::StringList::new(&names)));
    let cur = conf::get(key).unwrap_or_else(|| chosen.to_string());
    if let Some(i) = models.iter().position(|m| m.1 == cur) {
        combo.set_selected(i as u32);
    }
}

pub fn add_rows(group: &adw::PreferencesGroup, info: &Value) {
    let models = Rc::new(RefCell::new(local_models(info)));
    let gpu = adw::ComboRow::new();
    gpu.set_title("Modelo na placa de vídeo");
    gpu.set_subtitle("Maior = mais esperto, mas usa mais memória da placa");
    let cpu = adw::ComboRow::new();
    cpu.set_title("Modelo no processador");
    cpu.set_subtitle("Usado em modo jogo ou sem placa livre; pequeno é mais rápido");
    let chosen_gpu = info["escolhidos"]["gpu"].as_str().unwrap_or("").to_string();
    let chosen_cpu = info["escolhidos"]["cpu"].as_str().unwrap_or("").to_string();
    let refresh = {
        let (gpu, cpu, models) = (gpu.clone(), cpu.clone(), models.clone());
        Rc::new(move || {
            *models.borrow_mut() = local_models(&self::info());
            fill(&gpu, &models.borrow(), "model_gpu", &chosen_gpu);
            fill(&cpu, &models.borrow(), "model_cpu", &chosen_cpu);
        })
    };
    refresh();
    for (combo, key) in [(&gpu, "model_gpu"), (&cpu, "model_cpu")] {
        let models = models.clone();
        combo.connect_selected_notify(move |c| {
            if let Some(m) = models.borrow().get(c.selected() as usize) {
                if conf::get(key).as_deref() != Some(m.1.as_str()) {
                    conf::set(key, &m.1);
                    // Para valer já: o modelo novo sobe no próximo uso.
                    let _ = std::process::Command::new(corrector())
                        .arg("--stop")
                        .status();
                }
            }
        });
    }
    group.add(&gpu);
    group.add(&cpu);

    // Motor da IA.
    if info["llama"].is_null() {
        let row = adw::ActionRow::new();
        row.set_title("Motor da IA (llama.cpp)");
        row.set_subtitle("Ainda não instalado: sem ele a IA e o corretor não funcionam");
        let btn = gtk::Button::with_label("Instalar");
        btn.add_css_class("clip-primary");
        btn.set_valign(gtk::Align::Center);
        btn.connect_clicked(move |b| {
            b.set_sensitive(false);
            let (b1, b2) = (b.clone(), b.clone());
            stream(
                &["--instalar-llama"],
                move |l| {
                    if let Some(p) = l.strip_prefix("PROGRESSO ") {
                        b1.set_label(&format!("{p}%"));
                    }
                },
                move |ok| {
                    b2.set_label(if ok {
                        "Instalado"
                    } else {
                        "Falhou — tente de novo"
                    });
                    b2.set_sensitive(!ok);
                },
            );
        });
        row.add_suffix(&btn);
        group.add(&row);
    }

    // Baixar outros modelos do catálogo.
    let more = adw::ExpanderRow::new();
    more.set_title("Baixar modelos");
    more.set_subtitle("Modelos abertos (Hugging Face); ficam em ~/.local/share/clippit/models");
    let arrow = gtk::Image::from_icon_name("pan-down-symbolic");
    arrow.add_css_class("clip-arrow");
    more.add_action(&arrow);
    more.connect_expanded_notify(move |r| {
        arrow.set_icon_name(Some(if r.is_expanded() {
            "pan-up-symbolic"
        } else {
            "pan-down-symbolic"
        }));
    });
    for m in info["catalogo"].as_array().into_iter().flatten() {
        let row = adw::ActionRow::new();
        row.set_title(m["nome"].as_str().unwrap_or(""));
        row.set_subtitle(&format!(
            "{} · {} MB",
            m["nota"].as_str().unwrap_or(""),
            m["mb"]
        ));
        let id = m["id"].as_str().unwrap_or("").to_string();
        let btn = gtk::Button::with_label(if m["baixado"].as_bool() == Some(true) {
            "Baixado"
        } else {
            "Baixar"
        });
        btn.add_css_class("clip-btn");
        btn.set_valign(gtk::Align::Center);
        btn.set_sensitive(m["baixado"].as_bool() != Some(true));
        let refresh = refresh.clone();
        btn.connect_clicked(move |b| {
            b.set_sensitive(false);
            let (b1, b2, refresh) = (b.clone(), b.clone(), refresh.clone());
            stream(
                &["--baixar", &id],
                move |l| {
                    if let Some(p) = l.strip_prefix("PROGRESSO ") {
                        b1.set_label(&format!("{p}%"));
                    } else if let Some(e) = l.strip_prefix("ERRO ") {
                        b1.set_tooltip_text(Some(e));
                    }
                },
                move |ok| {
                    b2.set_label(if ok {
                        "Baixado"
                    } else {
                        "Falhou — tente de novo"
                    });
                    b2.set_sensitive(!ok);
                    if ok {
                        refresh();
                    }
                },
            );
        });
        row.add_suffix(&btn);
        more.add_row(&row);
    }
    group.add(&more);
}
