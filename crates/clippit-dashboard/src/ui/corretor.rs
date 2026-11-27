//! Página "Corretor": o corretor de texto que aparece ao lado do campo
//! (extensão do GNOME + modelo local). As opções vão para o corretor.conf.

use super::conf;
use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;

fn corrector_cmd() -> std::path::PathBuf {
    clippit_core::tools::tool("clippit-correct")
}

pub fn switch_row(
    title: &str,
    subtitle: &str,
    _icon: &str,
    key: &str,
    default: bool,
) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_subtitle(subtitle);
    let sw = gtk::Switch::new();
    sw.set_active(conf::get_bool(key, default));
    sw.set_valign(gtk::Align::Center);
    let key = key.to_string();
    sw.connect_state_set(move |_, on| {
        conf::set(&key, if on { "true" } else { "false" });
        gtk::glib::Propagation::Proceed
    });
    row.set_activatable_widget(Some(&sw));
    row.add_suffix(&sw);
    row
}

pub fn key_row(keys: &[&str], title: &str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_subtitle(subtitle);
    let keys_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    keys_box.set_valign(gtk::Align::Center);
    for (i, k) in keys.iter().enumerate() {
        if i > 0 {
            let plus = gtk::Label::new(Some("ou"));
            plus.add_css_class("dim-label");
            keys_box.append(&plus);
        }
        let l = gtk::Label::new(Some(k));
        l.add_css_class("clip-key");
        keys_box.append(&l);
    }
    row.add_suffix(&keys_box);
    row
}

fn model_status() -> (bool, String) {
    let out = std::process::Command::new(corrector_cmd())
        .arg("--status")
        .output();
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let on = s.starts_with("ligado");
            let detail = s
                .split("rodando em ")
                .nth(1)
                .map(|d| {
                    d.trim_end_matches(')')
                        .replace("Vulkan1", "placa de vídeo NVIDIA")
                        .replace("cpu", "processador")
                })
                .unwrap_or_default();
            (on, detail)
        }
        Err(_) => (
            false,
            "clippit-correct não encontrado (~/.local/bin ou PATH)".into(),
        ),
    }
}

pub fn create_page() -> gtk::Widget {
    // PreferencesPage já rola sozinha; outra ScrolledWindow em volta fazia o
    // conteúdo encolher até a largura mínima.
    let page = adw::PreferencesPage::new();
    page.set_hexpand(true);
    page.set_vexpand(true);

    // ---- Correção
    let group = adw::PreferencesGroup::new();
    group.set_title("Corretor de texto");
    group.set_description(Some(
        "Verifica o que você digita em qualquer app e mostra as sugestões num selo ao lado do campo.",
    ));
    group.add(&switch_row(
        "Corretor ativo",
        "Mostra o selo de sugestões nos campos de texto",
        "tools-check-spelling-symbolic",
        "ativo",
        true,
    ));
    group.add(&switch_row(
        "Verificar enquanto digito",
        "Desligado: só verifica quando você clica no selo",
        "document-edit-symbolic",
        "verificar_sozinho",
        true,
    ));
    page.add(&group);

    // ---- Modelo
    let model = adw::PreferencesGroup::new();
    model.set_title("Modelo local");
    // Qual modelo usar e onde ele roda: página Modelos de IA.
    model.set_description(Some(
        "Tudo roda no seu computador. O modelo sobe quando precisa e desliga sozinho depois de 1 minuto parado.",
    ));

    // Modo jogo: o mesmo que Ctrl+Shift+E.
    let game = adw::ActionRow::new();
    game.set_title("Modo jogo");
    game.set_subtitle(&format!(
        "Tira o modelo da placa de vídeo para jogos e renders; o corretor continua no processador. Atalho: {}",
        super::atalhos::pretty(&super::atalhos::current("modo_jogo"))
    ));
    let game_sw = gtk::Switch::new();
    game_sw.set_valign(gtk::Align::Center);
    let flag = std::path::PathBuf::from(
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into()),
    )
    .join("clippit-correct/modo-jogo");
    game_sw.set_active(flag.exists());
    {
        let flag = flag.clone();
        game_sw.connect_state_set(move |_, on| {
            // Só age se mudou de verdade (o timer abaixo também mexe na chave).
            if on != flag.exists() {
                let _ = std::process::Command::new(corrector_cmd())
                    .args(["--modo-jogo", if on { "on" } else { "off" }])
                    .spawn();
            }
            gtk::glib::Propagation::Proceed
        });
    }
    // Acompanha o Ctrl+Shift+E feito fora do painel.
    {
        let game_sw = game_sw.clone();
        gtk::glib::timeout_add_seconds_local(2, move || {
            if game_sw.root().is_none() {
                return gtk::glib::ControlFlow::Break;
            }
            let on = flag.exists();
            if game_sw.is_active() != on {
                game_sw.set_state(on);
                game_sw.set_active(on);
            }
            gtk::glib::ControlFlow::Continue
        });
    }
    game.add_suffix(&game_sw);
    game.set_activatable_widget(Some(&game_sw));
    model.add(&game);

    let status = adw::ActionRow::new();
    status.set_title("Situação agora");
    let pill = gtk::Label::new(None);
    pill.add_css_class("clip-pill");
    pill.set_valign(gtk::Align::Center);
    let stop = gtk::Button::with_label("Desligar agora");
    stop.add_css_class("clip-btn");
    stop.set_valign(gtk::Align::Center);
    status.add_suffix(&pill);
    status.add_suffix(&stop);

    let refresh = {
        let pill = pill.clone();
        let status = status.clone();
        let stop = stop.clone();
        move || {
            let (on, detail) = model_status();
            pill.set_text(if on { "Ligado" } else { "Desligado" });
            pill.remove_css_class(if on { "off" } else { "on" });
            pill.add_css_class(if on { "on" } else { "off" });
            status.set_subtitle(if on {
                detail.as_str()
            } else {
                "Sobe sozinho na próxima correção"
            });
            stop.set_sensitive(on);
        }
    };
    refresh();
    {
        let refresh = refresh.clone();
        stop.connect_clicked(move |_| {
            let _ = std::process::Command::new(corrector_cmd())
                .arg("--stop")
                .status();
            refresh();
        });
    }
    // Atualiza a situação a cada 3 s enquanto o painel está aberto.
    {
        let pill = pill.clone();
        gtk::glib::timeout_add_seconds_local(3, move || {
            if pill.root().is_none() {
                return gtk::glib::ControlFlow::Break;
            }
            refresh();
            gtk::glib::ControlFlow::Continue
        });
    }
    model.add(&status);
    page.add(&model);

    // ---- Qualidade (prova)
    let quality = adw::PreferencesGroup::new();
    quality.set_title("Qualidade");
    quality.set_description(Some(
        "Uma prova com frases de exemplo e as correções que você aplicou. A nota mostra se as mudanças melhoram o corretor.",
    ));
    let score = adw::ActionRow::new();
    score.set_title("Nota da última prova");
    let run = gtk::Button::with_label("Rodar a prova");
    run.add_css_class("clip-btn");
    run.set_valign(gtk::Align::Center);
    score.add_suffix(&run);
    let show_last = {
        let score = score.clone();
        move || {
            let hist = dirs::data_dir()
                .unwrap_or_default()
                .join("clippit/aprendizado/prova-historico.jsonl");
            let last = std::fs::read_to_string(hist)
                .ok()
                .and_then(|t| t.lines().last().map(str::to_string));
            let field = |l: &str, k: &str| -> String {
                l.split(&format!("\"{k}\": "))
                    .nth(1)
                    .map(|r| {
                        r.trim_start_matches('"')
                            .split([',', '"', '}'])
                            .next()
                            .unwrap_or("")
                            .to_string()
                    })
                    .unwrap_or_default()
            };
            match last {
                Some(l) => score.set_subtitle(&format!(
                    "{} de 100 · {} de {} correções certas · {} trocas erradas · em {}",
                    field(&l, "nota"),
                    field(&l, "acertos"),
                    field(&l, "esperadas"),
                    field(&l, "erradas"),
                    field(&l, "quando")
                )),
                None => score.set_subtitle("Ainda não rodou"),
            }
        }
    };
    show_last();
    {
        let show_last = show_last.clone();
        let score = score.clone();
        run.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            score.set_subtitle("Rodando… (uns 30 a 60 segundos)");
            let (tx, rx) = std::sync::mpsc::channel::<()>();
            std::thread::spawn(move || {
                let _ = std::process::Command::new(corrector_cmd())
                    .arg("--prova")
                    .output();
                let _ = tx.send(());
            });
            let btn = btn.clone();
            let show_last = show_last.clone();
            gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                if rx.try_recv().is_ok() {
                    btn.set_sensitive(true);
                    show_last();
                    return gtk::glib::ControlFlow::Break;
                }
                gtk::glib::ControlFlow::Continue
            });
        });
    }
    quality.add(&score);
    page.add(&quality);

    // ---- Atalhos
    let keys = adw::PreferencesGroup::new();
    keys.set_title("Atalhos");
    keys.set_description(Some(
        "Só valem enquanto o selo ou o painel de sugestões está na tela.",
    ));
    let open = super::atalhos::pretty(&super::atalhos::current("sugestoes"));
    let all = super::atalhos::pretty(&super::atalhos::current("aplicar"));
    keys.add(&key_row(
        &[open.as_str()],
        "Abrir ou fechar as sugestões",
        "O mesmo que clicar no selo",
    ));
    keys.add(&key_row(
        &[all.as_str(), "Ctrl+S"],
        "Corrigir tudo",
        "Ctrl+S não vale em editores, onde é salvar",
    ));
    keys.add(&key_row(&["Esc"], "Fechar o painel", ""));
    keys.add(&key_row(
        &["Ctrl+Z"],
        "Desfazer uma correção",
        "No próprio app",
    ));
    let game = super::atalhos::pretty(&super::atalhos::current("modo_jogo"));
    keys.add(&key_row(
        &[game.as_str()],
        "Modo jogo",
        "Libera a placa de vídeo; aperte de novo para voltar (vale em qualquer lugar)",
    ));
    page.add(&keys);

    page.upcast()
}
