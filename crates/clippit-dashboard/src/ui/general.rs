//! Geral: histórico, imagens (com OCR), idioma e limpar os dados.
//! Só aparece aqui o que o Clippit realmente usa.

use adw::prelude::*;
use clippit_core::{set_language, Config, HistoryManager};
use gtk::prelude::*;
use libadwaita as adw;

/// Salva uma mudança no config.toml (relê antes para não apagar outras).
fn save(change: impl FnOnce(&mut Config)) {
    if let Ok(mut cfg) = Config::load() {
        change(&mut cfg);
        if let Err(e) = cfg.save() {
            eprintln!("❌ Erro ao salvar: {}", e);
        }
    }
}

fn switch_row(
    title: &str,
    subtitle: &str,
    on: bool,
    apply: impl Fn(&mut Config, bool) + 'static,
) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_subtitle(subtitle);
    let sw = gtk::Switch::new();
    sw.set_active(on);
    sw.set_valign(gtk::Align::Center);
    sw.connect_active_notify(move |s| {
        let v = s.is_active();
        save(|c| apply(c, v));
    });
    row.add_suffix(&sw);
    row.set_activatable_widget(Some(&sw));
    row
}

/// Chave geral da IA e uma por função. Desligada, nenhum modelo roda.
fn ai_group() -> adw::PreferencesGroup {
    use super::conf;
    let g = group("Inteligência artificial");
    g.set_description(Some("Tudo roda no seu computador. Escolha onde quer usar a IA; os modelos ficam em Modelos de IA."));
    let on = |k: &str| conf::get_bool(k, true);
    let master = adw::ActionRow::new();
    master.set_title("Usar IA");
    master.set_subtitle("Desligada: nenhum modelo roda e o corretor usa só o dicionário");
    let master_sw = gtk::Switch::new();
    master_sw.set_active(on("ia"));
    master_sw.set_valign(gtk::Align::Center);
    master.add_suffix(&master_sw);
    master.set_activatable_widget(Some(&master_sw));
    g.add(&master);
    let mut rows = Vec::new();
    for (key, title, sub) in [
        ("ia_chat", "Chat com a IA", "Alt+I e o \"?\" no lançador"),
        (
            "ia_acoes",
            "Ações no texto",
            "Resumir, traduzir, reescrever (Alt+A)",
        ),
        (
            "ia_assistente",
            "Assistente de resposta",
            "Sugere respostas para conversas (Alt+K)",
        ),
        (
            "ia_corretor",
            "Corretor com IA",
            "Desligado: o corretor usa só o dicionário",
        ),
        (
            "ia_doutor",
            "Doutor com IA",
            "Analisa erros novos do Clippit; os conhecidos ele corrige de qualquer jeito",
        ),
    ] {
        let row = adw::ActionRow::new();
        row.set_title(title);
        row.set_subtitle(sub);
        let sw = gtk::Switch::new();
        sw.set_active(on(key));
        sw.set_valign(gtk::Align::Center);
        sw.connect_active_notify(move |s| {
            conf::set(key, if s.is_active() { "true" } else { "false" });
        });
        row.add_suffix(&sw);
        row.set_activatable_widget(Some(&sw));
        row.set_sensitive(master_sw.is_active());
        g.add(&row);
        rows.push(row);
    }
    master_sw.connect_active_notify(move |s| {
        let v = s.is_active();
        conf::set("ia", if v { "true" } else { "false" });
        for r in &rows {
            r.set_sensitive(v);
        }
        if !v {
            super::modelos::restart_ai(); // desliga o modelo que estiver carregado
        }
    });
    g
}

fn group(title: &str) -> adw::PreferencesGroup {
    let g = adw::PreferencesGroup::new();
    g.set_title(title);
    g
}

pub fn create_page() -> gtk::Widget {
    let config = Config::load().unwrap_or_default();
    let page = adw::PreferencesPage::new();

    // ---- Histórico
    let hist = group("Histórico");
    let items = adw::ActionRow::new();
    items.set_title("Itens guardados");
    items.set_subtitle("Quando passa disso, os mais antigos saem");
    let spin = gtk::SpinButton::with_range(10.0, 10000.0, 50.0);
    spin.set_value(config.general.max_history_items as f64);
    spin.set_valign(gtk::Align::Center);
    spin.connect_value_changed(|s| {
        let v = s.value() as usize;
        save(|c| c.general.max_history_items = v);
    });
    items.add_suffix(&spin);
    hist.add(&items);
    hist.add(&switch_row(
        "Avisar ao copiar",
        "Mostra uma notificação a cada item copiado",
        config.ui.show_notifications,
        |c, v| c.ui.show_notifications = v,
    ));
    page.add(&hist);

    // ---- Inteligência artificial (fica no corretor.conf, lido pela extensão e pela IA)
    page.add(&ai_group());

    // ---- Imagens
    let img = group("Imagens");
    img.add(&switch_row(
        "Guardar imagens",
        "Prints e imagens copiadas entram no histórico",
        config.privacy.enable_image_capture,
        |c, v| c.privacy.enable_image_capture = v,
    ));
    let size = adw::ActionRow::new();
    size.set_title("Tamanho máximo");
    size.set_subtitle("Imagens maiores que isso não são guardadas");
    let size_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    size_box.set_valign(gtk::Align::Center);
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 1.0, 20.0, 1.0);
    scale.set_value(config.privacy.max_image_size_mb as f64);
    scale.set_width_request(140);
    scale.set_draw_value(false);
    let size_label = gtk::Label::new(Some(&format!("{} MB", config.privacy.max_image_size_mb)));
    size_label.set_width_chars(6);
    size_label.set_xalign(1.0);
    size_label.add_css_class("dim-label");
    let lbl = size_label.clone();
    scale.connect_value_changed(move |s| {
        let v = s.value() as u32;
        lbl.set_text(&format!("{} MB", v));
        save(|c| c.privacy.max_image_size_mb = v);
    });
    size_box.append(&scale);
    size_box.append(&size_label);
    size.add_suffix(&size_box);
    img.add(&size);
    img.add(&switch_row(
        "Ler o texto das imagens",
        "Assim dá para achar um print pelo que está escrito nele",
        config.features.enable_ocr,
        |c, v| c.features.enable_ocr = v,
    ));
    let ocr = adw::ComboRow::new();
    ocr.set_title("Idiomas da leitura");
    let langs = [
        ("por+eng", "Português e inglês"),
        ("por", "Português"),
        ("eng", "Inglês"),
    ];
    ocr.set_model(Some(&gtk::StringList::new(
        &langs.iter().map(|l| l.1).collect::<Vec<_>>(),
    )));
    ocr.set_selected(
        langs
            .iter()
            .position(|l| l.0 == config.ocr.languages)
            .unwrap_or(0) as u32,
    );
    ocr.connect_selected_notify(move |r| {
        let v = langs
            .get(r.selected() as usize)
            .map(|l| l.0)
            .unwrap_or("por+eng");
        save(|c| c.ocr.languages = v.to_string());
    });
    img.add(&ocr);
    page.add(&img);

    // ---- Idioma
    let lang_group = group("Idioma");
    let lang = adw::ComboRow::new();
    lang.set_title("Idioma do Clippit");
    lang.set_subtitle("O painel reabre para trocar");
    let options = [("pt", "Português"), ("en", "English")];
    lang.set_model(Some(&gtk::StringList::new(
        &options.iter().map(|l| l.1).collect::<Vec<_>>(),
    )));
    lang.set_selected(
        options
            .iter()
            .position(|l| l.0 == config.ui.language)
            .unwrap_or(0) as u32,
    );
    lang.connect_selected_notify(move |r| {
        let new = options
            .get(r.selected() as usize)
            .map(|l| l.0)
            .unwrap_or("pt")
            .to_string();
        let mut changed = false;
        save(|c| {
            changed = c.ui.language != new;
            c.ui.language = new.clone();
        });
        if changed {
            set_language(&new);
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(300), || {
                std::process::Command::new("clippit-dashboard").spawn().ok();
                gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(200), || {
                    std::process::exit(0)
                });
            });
        }
    });
    lang_group.add(&lang);
    page.add(&lang_group);

    // ---- Dados
    let data = group("Dados");
    let clear = adw::ActionRow::new();
    clear.set_title("Apagar o histórico");
    clear.set_subtitle("Remove para sempre tudo o que foi copiado");
    let btn = gtk::Button::with_label("Apagar…");
    btn.add_css_class("clip-danger");
    btn.set_valign(gtk::Align::Center);
    btn.connect_clicked(|b| {
        let dialog = adw::MessageDialog::new(
            b.root().and_downcast_ref::<gtk::Window>(),
            Some("Apagar todo o histórico?"),
            Some("Tudo o que foi copiado some e não dá para desfazer."),
        );
        dialog.add_response("cancelar", "Cancelar");
        dialog.add_response("apagar", "Apagar");
        dialog.set_response_appearance("apagar", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancelar"));
        dialog.connect_response(None, |_, resp| {
            if resp != "apagar" {
                return;
            }
            let mut db = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            db.push("clippit");
            db.push("history.db");
            match HistoryManager::new(db, 100).and_then(|h| h.clear()) {
                Ok(_) => eprintln!("✅ Histórico apagado"),
                Err(e) => eprintln!("❌ Erro ao apagar histórico: {}", e),
            }
        });
        dialog.present();
    });
    clear.add_suffix(&btn);
    data.add(&clear);
    page.add(&data);

    page.upcast()
}
