//! Atalhos da extensão do GNOME: padrões, leitura/escrita no corretor.conf
//! (atalho_<id> = <Alt>space) e a janelinha que captura a combinação nova.
//! A extensão confere o arquivo a cada 2 s e troca o atalho na hora.

use super::conf;
use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;

/// (id, padrão no formato do GTK, título, descrição)
pub const SHORTCUTS: &[(&str, &str, &str, &str)] = &[
    (
        "lancador",
        "<Alt>space",
        "Abrir o lançador",
        "Apps, arquivos, comandos, emoji e mais",
    ),
    ("ia", "<Alt>i", "Conversar com a IA", "Abre direto no chat"),
    (
        "acoes",
        "<Alt>a",
        "Ações no texto selecionado",
        "Resumir, traduzir, reescrever…",
    ),
    (
        "assistente",
        "<Alt>k",
        "Sugerir resposta",
        "Lê a conversa da janela em foco",
    ),
    (
        "modo_jogo",
        "<Control><Shift>e",
        "Modo jogo",
        "Tira a IA da placa de vídeo",
    ),
    (
        "sugestoes",
        "<Alt>o",
        "Abrir as sugestões do corretor",
        "O mesmo que clicar no selo",
    ),
    (
        "aplicar",
        "<Alt>s",
        "Corrigir tudo",
        "Aplica todas as sugestões do corretor",
    ),
    (
        "ajuste",
        "<Alt>f",
        "Escrever um ajuste no assistente",
        "Ex.: \"mais curta\"",
    ),
];

pub fn default_of(id: &str) -> &'static str {
    SHORTCUTS
        .iter()
        .find(|s| s.0 == id)
        .map(|s| s.1)
        .unwrap_or("")
}

/// Atalho atual (o do .conf ou o padrão).
pub fn current(id: &str) -> String {
    conf::get(&format!("atalho_{id}"))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default_of(id).to_string())
}

/// "<Control><Shift>e" → "Ctrl+Shift+E"; "desligado" → "Desligado".
pub fn pretty(accel: &str) -> String {
    if accel == "desligado" {
        return "Desligado".into();
    }
    if let Some((key, mods)) = gtk::accelerator_parse(accel) {
        let mut parts = Vec::new();
        if mods.contains(gtk::gdk::ModifierType::CONTROL_MASK) {
            parts.push("Ctrl".to_string());
        }
        if mods.contains(gtk::gdk::ModifierType::SHIFT_MASK) {
            parts.push("Shift".into());
        }
        if mods.contains(gtk::gdk::ModifierType::ALT_MASK) {
            parts.push("Alt".into());
        }
        if mods.contains(gtk::gdk::ModifierType::SUPER_MASK) {
            parts.push("Super".into());
        }
        let name = key.name().map(|n| n.to_string()).unwrap_or_default();
        let shown = match name.as_str() {
            "space" => "Espaço".to_string(),
            "Return" => "Enter".into(),
            n if n.chars().count() == 1 => n.to_uppercase(),
            n => n.to_string(),
        };
        parts.push(shown);
        return parts.join("+");
    }
    accel.to_string()
}

/// Janela que espera você apertar a combinação. on_done recebe o accel novo
/// (já salvo) para atualizar a tela.
pub fn capture(
    parent: Option<&gtk::Window>,
    id: &'static str,
    title: &str,
    on_done: impl Fn(String) + 'static,
) {
    let win = adw::Window::builder()
        .modal(true)
        .default_width(420)
        .resizable(false)
        .title(title)
        .build();
    if let Some(p) = parent {
        win.set_transient_for(Some(p));
    }
    win.add_css_class("clippit-dashboard");
    let body = gtk::Box::new(gtk::Orientation::Vertical, 14);
    body.set_margin_top(22);
    body.set_margin_bottom(18);
    body.set_margin_start(22);
    body.set_margin_end(22);
    let head = gtk::Label::new(Some(title));
    head.add_css_class("heading");
    head.set_xalign(0.0);
    let hint = gtk::Label::new(Some(
        "Aperte a nova combinação (com Ctrl, Alt ou Super). Esc cancela.",
    ));
    hint.add_css_class("dim-label");
    hint.set_xalign(0.0);
    hint.set_wrap(true);
    let shown = gtk::Label::new(Some(&pretty(&current(id))));
    shown.add_css_class("clip-key");
    shown.add_css_class("clip-key-big");
    let warn = gtk::Label::new(None);
    warn.add_css_class("clip-warn");
    warn.set_xalign(0.0);
    warn.set_wrap(true);
    let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    buttons.set_halign(gtk::Align::End);
    let off = gtk::Button::with_label("Desligar");
    off.add_css_class("clip-btn");
    let reset = gtk::Button::with_label("Padrão");
    reset.add_css_class("clip-btn");
    let save = gtk::Button::with_label("Salvar");
    save.add_css_class("clip-primary");
    save.set_sensitive(false);
    buttons.append(&off);
    buttons.append(&reset);
    buttons.append(&save);
    body.append(&head);
    body.append(&hint);
    body.append(&shown);
    body.append(&warn);
    body.append(&buttons);
    win.set_content(Some(&body));

    let chosen = std::rc::Rc::new(std::cell::RefCell::new(String::new()));
    let on_done = std::rc::Rc::new(on_done);
    let finish = {
        let win = win.clone();
        let on_done = on_done.clone();
        move |accel: String| {
            conf::set(&format!("atalho_{id}"), &accel);
            on_done(accel);
            win.close();
        }
    };

    let keys = gtk::EventControllerKey::new();
    keys.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let (shown, warn, save, chosen, win) = (
            shown.clone(),
            warn.clone(),
            save.clone(),
            chosen.clone(),
            win.clone(),
        );
        keys.connect_key_pressed(move |_, key, _code, state| {
            use gtk::gdk::Key;
            if key == Key::Escape && state.is_empty() {
                win.close();
                return gtk::glib::Propagation::Stop;
            }
            let mods = state
                & (gtk::gdk::ModifierType::CONTROL_MASK
                    | gtk::gdk::ModifierType::SHIFT_MASK
                    | gtk::gdk::ModifierType::ALT_MASK
                    | gtk::gdk::ModifierType::SUPER_MASK);
            let only_mod = matches!(
                key,
                Key::Control_L | Key::Control_R | Key::Shift_L | Key::Shift_R | Key::Alt_L | Key::Alt_R
                    | Key::Super_L | Key::Super_R | Key::Meta_L | Key::Meta_R | Key::ISO_Level3_Shift
            );
            if only_mod {
                return gtk::glib::Propagation::Stop;
            }
            let fkey = key.name().is_some_and(|n| n.len() <= 3 && n.starts_with('F') && n[1..].parse::<u8>().is_ok());
            if mods.is_empty() && !fkey {
                warn.set_text("Use pelo menos Ctrl, Alt ou Super (sozinha, a tecla atrapalharia a digitação).");
                return gtk::glib::Propagation::Stop;
            }
            let accel = gtk::accelerator_name(key.to_lower(), mods).to_string();
            shown.set_text(&pretty(&accel));
            let clash = SHORTCUTS.iter().find(|s| s.0 != id && current(s.0) == accel);
            warn.set_text(&match clash {
                Some(c) => format!("Já é o atalho de \"{}\". Se salvar, os dois ficam iguais.", c.2),
                None if accel == "<Alt>c" => "Alt+C é o atalho do histórico.".into(),
                None => String::new(),
            });
            *chosen.borrow_mut() = accel;
            save.set_sensitive(true);
            gtk::glib::Propagation::Stop
        });
    }
    win.add_controller(keys);
    {
        let finish = finish.clone();
        save.connect_clicked(move |_| {
            let a = chosen.borrow().clone();
            if !a.is_empty() {
                finish(a);
            }
        });
    }
    {
        let finish = finish.clone();
        reset.connect_clicked(move |_| finish(default_of(id).to_string()));
    }
    off.connect_clicked(move |_| finish("desligado".to_string()));
    win.present();
}

/// Linha de atalho com o botão Trocar (para a tela Atalhos).
pub fn editable_row(id: &'static str, title: &'static str, subtitle: &str) -> adw::ActionRow {
    let row = adw::ActionRow::new();
    row.set_title(title);
    row.set_subtitle(subtitle);
    let key = gtk::Label::new(Some(&pretty(&current(id))));
    key.add_css_class("clip-key");
    key.set_valign(gtk::Align::Center);
    let btn = gtk::Button::with_label("Trocar");
    btn.add_css_class("clip-btn");
    btn.set_valign(gtk::Align::Center);
    let k = key.clone();
    btn.connect_clicked(move |b| {
        let k = k.clone();
        capture(
            b.root().and_downcast_ref::<gtk::Window>(),
            id,
            title,
            move |accel| k.set_text(&pretty(&accel)),
        );
    });
    let bx = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    bx.append(&key);
    bx.append(&btn);
    row.add_suffix(&bx);
    row
}
