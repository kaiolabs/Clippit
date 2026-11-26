//! Página "Modelos de IA": onde roda, o modelo padrão, um modelo para cada
//! tarefa, visão e busca, e baixar modelos (catálogo ou qualquer .gguf do
//! Hugging Face). Quem sabe de tudo é o `clippit-correct --modelos` (JSON).

use super::conf;
use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const PICK: &str = "__escolher__";

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

/// Desliga o modelo agora: a próxima tarefa sobe com a escolha nova.
pub fn restart_ai() {
    let _ = std::process::Command::new(corrector())
        .arg("--stop")
        .status();
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
    next(input, Rc::new(on_line), Rc::new(on_end), false);
}

/// Saída inteira de um comando, sem travar a tela (ex.: listar um repositório).
fn run_async(args: &[&str], done: impl Fn(String) + 'static) {
    let exe = corrector();
    let mut argv: Vec<&std::ffi::OsStr> = vec![exe.as_os_str()];
    argv.extend(args.iter().map(|a| std::ffi::OsStr::new(*a)));
    match gtk::gio::Subprocess::newv(&argv, gtk::gio::SubprocessFlags::STDOUT_PIPE) {
        Ok(p) => p.communicate_utf8_async(None, gtk::gio::Cancellable::NONE, move |r| {
            done(
                r.ok()
                    .and_then(|(o, _)| o)
                    .map(|g| g.to_string())
                    .unwrap_or_default(),
            )
        }),
        Err(_) => done(String::new()),
    }
}

/// Uma lista de modelos ligada a uma chave do corretor.conf.
struct ModelChoice {
    row: adw::ComboRow,
    key: &'static str,
    kind: &'static str,                  // texto, visao ou embed
    default_label: Option<&'static str>, // opção sem arquivo (valor vazio)
    options: RefCell<Vec<(String, String)>>,
    filling: Cell<bool>,
}

impl ModelChoice {
    fn fill(&self, locals: &[Value], fallback: &str) {
        let mut opts: Vec<(String, String)> = Vec::new();
        if let Some(d) = self.default_label {
            opts.push((d.to_string(), String::new()));
        }
        for m in locals
            .iter()
            .filter(|m| m["tipo"].as_str() == Some(self.kind))
        {
            opts.push((
                format!(
                    "{} · {} MB",
                    m["arquivo"]
                        .as_str()
                        .unwrap_or("")
                        .trim_end_matches(".gguf"),
                    m["mb"]
                ),
                m["caminho"].as_str().unwrap_or("").to_string(),
            ));
        }
        let home = dirs::home_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let cur = conf::get(self.key).unwrap_or_else(|| fallback.to_string());
        let cur_full = if let Some(rest) = cur.strip_prefix('~') {
            format!("{home}{rest}")
        } else {
            cur.clone()
        };
        if !cur.is_empty() && !opts.iter().any(|o| o.1 == cur || o.1 == cur_full) {
            let name = std::path::Path::new(&cur_full)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            opts.push((format!("{name} (arquivo escolhido)"), cur.clone()));
        }
        opts.push(("Escolher arquivo…".into(), PICK.into()));
        self.filling.set(true);
        let names: Vec<&str> = opts.iter().map(|o| o.0.as_str()).collect();
        self.row.set_model(Some(&gtk::StringList::new(&names)));
        let sel = opts
            .iter()
            .position(|o| o.1 == cur || o.1 == cur_full)
            .unwrap_or(0);
        self.row.set_selected(sel as u32);
        *self.options.borrow_mut() = opts;
        self.filling.set(false);
    }
}

type Choices = Rc<RefCell<Vec<(Rc<ModelChoice>, String)>>>;

pub fn create_page() -> gtk::Widget {
    let page = adw::PreferencesPage::new();
    let data = Rc::new(RefCell::new(info()));
    let choices: Choices = Rc::new(RefCell::new(Vec::new()));

    // Recarrega a lista em todas as escolhas (depois de baixar ou escolher arquivo).
    let refresh: Rc<dyn Fn()> = {
        let (data, choices) = (data.clone(), choices.clone());
        Rc::new(move || {
            *data.borrow_mut() = info();
            let locals = data.borrow()["locais"]
                .as_array()
                .cloned()
                .unwrap_or_default();
            for (c, fallback) in choices.borrow().iter() {
                c.fill(&locals, fallback);
            }
        })
    };

    let add_choice = |group: &adw::PreferencesGroup,
                      title: &str,
                      subtitle: &str,
                      key: &'static str,
                      kind: &'static str,
                      default_label: Option<&'static str>,
                      fallback: String| {
        let row = adw::ComboRow::new();
        row.set_title(title);
        row.set_subtitle(subtitle);
        let c = Rc::new(ModelChoice {
            row: row.clone(),
            key,
            kind,
            default_label,
            options: RefCell::new(Vec::new()),
            filling: Cell::new(false),
        });
        c.fill(
            &data.borrow()["locais"]
                .as_array()
                .cloned()
                .unwrap_or_default(),
            &fallback,
        );
        let (c2, refresh2) = (c.clone(), refresh.clone());
        row.connect_selected_notify(move |r| {
            if c2.filling.get() {
                return;
            }
            let value = c2
                .options
                .borrow()
                .get(r.selected() as usize)
                .map(|o| o.1.clone())
                .unwrap_or_default();
            if value == PICK {
                pick_file(r, key, refresh2.clone());
                return;
            }
            if conf::get(key).unwrap_or_default() != value {
                conf::set(key, &value);
                restart_ai();
            }
        });
        group.add(&row);
        choices.borrow_mut().push((c, fallback));
    };

    let d = data.borrow().clone();

    // ---- Onde roda
    let run = adw::PreferencesGroup::new();
    run.set_title("Onde roda");
    run.set_description(Some(
        "Tudo roda no seu computador. O modelo sobe quando precisa e desliga sozinho depois de 1 minuto parado.",
    ));
    if d["llama"].is_null() {
        run.add(&download_row(
            "Motor da IA (llama.cpp)",
            "Ainda não instalado: sem ele os modelos não rodam",
            false,
            vec!["--instalar-llama".into()],
            refresh.clone(),
        ));
    }
    let device = adw::ComboRow::new();
    device.set_title("Placa ou processador");
    device.set_subtitle("Automático usa a placa NVIDIA só quando ela está livre");
    let mut options: Vec<(&'static str, &'static str)> = vec![("auto", "Automático")];
    for g in gpus(&d) {
        match g.as_str() {
            "nvidia" => options.push(("nvidia", "Placa de vídeo NVIDIA")),
            "amd" => options.push(("amd", "Placa de vídeo AMD")),
            "intel" => options.push(("intel", "Placa integrada Intel")),
            _ => {}
        }
    }
    options.push(("cpu", "Só processador"));
    device.set_model(Some(&gtk::StringList::new(
        &options.iter().map(|o| o.1).collect::<Vec<_>>(),
    )));
    let current = conf::get("device").unwrap_or_else(|| "auto".into());
    device.set_selected(options.iter().position(|o| o.0 == current).unwrap_or(0) as u32);
    device.connect_selected_notify(move |row| {
        conf::set("device", options[row.selected() as usize].0);
        restart_ai();
    });
    run.add(&device);
    page.add(&run);

    // ---- Modelo padrão
    let def = adw::PreferencesGroup::new();
    def.set_title("Modelo padrão");
    def.set_description(Some(
        "Usado pelas tarefas que estão em \"Padrão\" logo abaixo.",
    ));
    let esc = &d["escolhidos"];
    add_choice(
        &def,
        "Na placa de vídeo",
        "Maior = mais esperto, mas usa mais memória da placa",
        "model_gpu",
        "texto",
        None,
        esc["gpu"].as_str().unwrap_or("").into(),
    );
    add_choice(
        &def,
        "No processador",
        "Usado em modo jogo ou sem placa livre; pequeno é mais rápido",
        "model_cpu",
        "texto",
        None,
        esc["cpu"].as_str().unwrap_or("").into(),
    );
    page.add(&def);

    // ---- Um modelo por tarefa
    let tasks = adw::PreferencesGroup::new();
    tasks.set_title("Modelo de cada tarefa");
    tasks.set_description(Some(
        "Trocar de modelo entre uma tarefa e outra leva alguns segundos. Um modelo que não cabe na placa roda no processador.",
    ));
    for (key, title) in [
        ("modelo_corretor", "Corretor"),
        ("modelo_chat", "Chat com a IA"),
        ("modelo_assistente", "Assistente de resposta"),
        ("modelo_acoes", "Ações no texto"),
        ("modelo_doutor", "Doutor"),
    ] {
        add_choice(
            &tasks,
            title,
            "",
            key,
            "texto",
            Some("Padrão"),
            String::new(),
        );
    }
    page.add(&tasks);

    // ---- Visão e busca
    let extra = adw::PreferencesGroup::new();
    extra.set_title("Visão e busca");
    add_choice(
        &extra,
        "Visão (ler prints e imagens)",
        "Automático = o mmproj que combina com o modelo da placa",
        "model_visao",
        "visao",
        Some("Automático"),
        String::new(),
    );
    add_choice(
        &extra,
        "Busca por significado",
        "Acha respostas parecidas para o assistente (Alt+K)",
        "model_embed",
        "embed",
        Some("Padrão (Qwen3 Embedding)"),
        String::new(),
    );
    page.add(&extra);

    // ---- Baixar
    let dl = adw::PreferencesGroup::new();
    dl.set_title("Baixar modelos");
    dl.set_description(Some(
        "Ficam em ~/.local/share/clippit/models. Qualquer modelo GGUF funciona.",
    ));
    let more = adw::ExpanderRow::new();
    more.set_title("Recomendados");
    more.set_subtitle("Testados com o Clippit");
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
    for m in d["catalogo"].as_array().into_iter().flatten() {
        more.add_row(&download_row(
            m["nome"].as_str().unwrap_or(""),
            &format!("{} · {} MB", m["nota"].as_str().unwrap_or(""), m["mb"]),
            m["baixado"].as_bool() == Some(true),
            vec!["--baixar".into(), m["id"].as_str().unwrap_or("").into()],
            refresh.clone(),
        ));
    }
    dl.add(&more);

    // Qualquer modelo do Hugging Face: link ou "org/repositório".
    let search = adw::EntryRow::new();
    search.set_title(
        "Outro modelo: link ou repositório do Hugging Face (ex.: unsloth/Qwen3.5-9B-GGUF)",
    );
    let go = gtk::Button::with_label("Procurar");
    go.add_css_class("clip-btn");
    go.set_valign(gtk::Align::Center);
    search.add_suffix(&go);
    dl.add(&search);
    let results: Rc<RefCell<Vec<adw::ActionRow>>> = Rc::new(RefCell::new(Vec::new()));
    let find: Rc<dyn Fn()> = {
        let (search, dl, results, refresh, go) = (
            search.clone(),
            dl.clone(),
            results.clone(),
            refresh.clone(),
            go.clone(),
        );
        Rc::new(move || {
            let text = search.text().to_string();
            if text.trim().is_empty() {
                return;
            }
            for r in results.borrow_mut().drain(..) {
                dl.remove(&r);
            }
            go.set_sensitive(false);
            go.set_label("Procurando…");
            let (dl, results, refresh, go) =
                (dl.clone(), results.clone(), refresh.clone(), go.clone());
            run_async(&["--hf-arquivos", text.trim()], move |out| {
                go.set_sensitive(true);
                go.set_label("Procurar");
                let v: Value = serde_json::from_str(&out).unwrap_or(Value::Null);
                let err = if v.is_null() {
                    Some("não consegui falar com o Hugging Face")
                } else {
                    v["erro"].as_str()
                };
                if let Some(e) = err {
                    let row = adw::ActionRow::new();
                    row.set_title("Nada encontrado");
                    row.set_subtitle(e);
                    dl.add(&row);
                    results.borrow_mut().push(row);
                    return;
                }
                for f in v["arquivos"].as_array().into_iter().flatten() {
                    let size = f["mb"]
                        .as_u64()
                        .map(|mb| format!("{mb} MB"))
                        .unwrap_or_else(|| "tamanho desconhecido".into());
                    let name = f["arquivo"].as_str().unwrap_or("");
                    let low = name.to_lowercase();
                    let hint = if low.contains("mmproj") {
                        " · visão"
                    } else if low.contains("embed") {
                        " · busca"
                    } else {
                        ""
                    };
                    let row = download_row(
                        name,
                        &format!("{size}{hint}"),
                        false,
                        vec![
                            "--baixar-url".into(),
                            f["url"].as_str().unwrap_or("").into(),
                        ],
                        refresh.clone(),
                    );
                    dl.add(&row);
                    results.borrow_mut().push(row);
                }
            });
        })
    };
    {
        let find = find.clone();
        go.connect_clicked(move |_| find());
    }
    search.connect_entry_activated(move |_| find());
    page.add(&dl);

    page.upcast()
}

/// Linha com Baixar (mostra a porcentagem e, no fim, atualiza as listas de modelos).
fn download_row(
    title: &str,
    subtitle: &str,
    done: bool,
    args: Vec<String>,
    refresh: Rc<dyn Fn()>,
) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_subtitle(subtitle);
    let label = if args.first().map(|a| a.as_str()) == Some("--instalar-llama") {
        "Instalar"
    } else {
        "Baixar"
    };
    let btn = gtk::Button::with_label(if done { "Baixado" } else { label });
    btn.add_css_class("clip-btn");
    btn.set_valign(gtk::Align::Center);
    btn.set_sensitive(!done);
    btn.connect_clicked(move |b| {
        b.set_sensitive(false);
        let (b1, b2, refresh) = (b.clone(), b.clone(), refresh.clone());
        let argv: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        stream(
            &argv,
            move |l| {
                if let Some(p) = l.strip_prefix("PROGRESSO ") {
                    b1.set_label(&format!("{p}%"));
                } else if let Some(e) = l.strip_prefix("ERRO ") {
                    b1.set_tooltip_text(Some(e));
                }
            },
            move |ok| {
                b2.set_label(if ok {
                    "Pronto"
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
    row
}

/// "Escolher arquivo…": qualquer .gguf do computador.
fn pick_file(row: &adw::ComboRow, key: &'static str, refresh: Rc<dyn Fn()>) {
    let parent = row.root().and_downcast::<gtk::Window>();
    let dialog = gtk::FileChooserNative::new(
        Some("Escolher um modelo (.gguf)"),
        parent.as_ref(),
        gtk::FileChooserAction::Open,
        Some("Escolher"),
        Some("Cancelar"),
    );
    let filter = gtk::FileFilter::new();
    filter.set_name(Some("Modelos GGUF"));
    filter.add_pattern("*.gguf");
    dialog.add_filter(&filter);
    let keep = Rc::new(RefCell::new(Some(dialog.clone())));
    dialog.connect_response(move |d, resp| {
        if resp == gtk::ResponseType::Accept {
            if let Some(path) = d.file().and_then(|f| f.path()) {
                conf::set(key, &path.to_string_lossy());
                restart_ai();
            }
        }
        refresh(); // volta a mostrar a escolha atual (ou a nova)
        keep.borrow_mut().take();
    });
    dialog.show();
}
