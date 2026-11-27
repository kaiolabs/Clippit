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
    (
        "emoji",
        "<Alt>e",
        "Abrir os emojis",
        "Direto na grade de emoji e símbolos (é o mesmo que digitar :)",
    ),
    (
        "snippets",
        "<Alt>semicolon",
        "Abrir os snippets",
        "Direto na lista de textos prontos (é o mesmo que digitar ;)",
    ),
];

/// Atalhos de dentro do lançador (só valem com ele aberto): mesma troca, mesmo arquivo.
/// `copiar` só usa os modificadores (vale com Enter: Ctrl+Enter copia em vez de colar).
pub const INNER: &[(&str, &str, &str, &str)] = &[
    (
        "fechar",
        "Escape",
        "Fechar / parar a IA",
        "O primeiro aperto para a resposta da IA; o segundo fecha",
    ),
    (
        "lado",
        "<Alt>c",
        "Área de transferência ao lado",
        "Abre e fecha a lista do que você copiou; no histórico, fecha",
    ),
    (
        "copiar",
        "<Control>Return",
        "Só copiar (sem colar)",
        "Enter com esta combinação copia em vez de colar; no chat, abre a fonte",
    ),
    (
        "voltar",
        "BackSpace",
        "Voltar para o início",
        "Sem modificador, só com o campo vazio",
    ),
    ("ditado", "<Alt>v", "Falar (ditado)", "No chat da IA"),
    (
        "nova_conversa",
        "<Control>n",
        "Nova conversa",
        "No chat da IA",
    ),
    (
        "conversas",
        "<Control>h",
        "Conversas anteriores",
        "No chat da IA",
    ),
    (
        "transformar",
        "Tab",
        "Transformar o item",
        "No histórico: MAIÚSCULAS, JSON, traduzir…; de novo volta",
    ),
    (
        "fixar",
        "<Control>p",
        "Fixar / desafixar",
        "No histórico e nos emojis",
    ),
    (
        "categoria",
        "Tab",
        "Próxima categoria de emoji",
        "Com Shift volta para a anterior",
    ),
    (
        "apagar",
        "<Control>Delete",
        "Apagar",
        "No histórico e nas conversas",
    ),
    (
        "ampliar",
        "<Control>space",
        "Ampliar a prévia",
        "No histórico",
    ),
];

/// Todos os atalhos que dá para trocar: os globais e os de dentro do lançador.
pub fn all(
) -> impl Iterator<Item = &'static (&'static str, &'static str, &'static str, &'static str)> {
    SHORTCUTS.iter().chain(INNER.iter())
}

fn is_inner(id: &str) -> bool {
    INNER.iter().any(|s| s.0 == id)
}

pub fn default_of(id: &str) -> &'static str {
    all().find(|s| s.0 == id).map(|s| s.1).unwrap_or("")
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
            "Escape" => "Esc".into(),
            "BackSpace" => "Backspace".into(),
            "semicolon" => ";".into(),
            "colon" => ":".into(),
            "period" => ".".into(),
            "comma" => ",".into(),
            "Delete" => "Del".into(),
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
    let inner = is_inner(id);
    let hint = gtk::Label::new(Some(if inner {
        if id == "copiar" {
            "Aperte Enter junto com Ctrl, Alt, Shift ou Super. Para cancelar, use o botão."
        } else {
            "Aperte a nova tecla ou combinação. Para cancelar, use o botão."
        }
    } else {
        "Aperte a nova combinação (com Ctrl, Alt ou Super). Esc cancela."
    }));
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
    let cancel = gtk::Button::with_label("Cancelar");
    cancel.add_css_class("clip-btn");
    let off = gtk::Button::with_label("Desligar");
    off.add_css_class("clip-btn");
    let reset = gtk::Button::with_label("Padrão");
    reset.add_css_class("clip-btn");
    let save = gtk::Button::with_label("Salvar");
    save.add_css_class("clip-primary");
    save.set_sensitive(false);
    buttons.append(&cancel);
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
            // Nos de dentro do lançador o Esc pode ser o atalho; o cancelar é o botão.
            if key == Key::Escape && state.is_empty() && !inner {
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
            // Dentro do lançador, teclas que não escrevem nada (Tab, Esc, Del, setas) valem sozinhas.
            let silent = key.to_unicode().is_none_or(|c| c.is_control());
            if mods.is_empty() && !fkey && !(inner && silent) {
                warn.set_text(if inner {
                    "Use pelo menos Ctrl, Alt ou Super, ou uma tecla que não escreve nada (Tab, Esc, Del, F1…)."
                } else {
                    "Use pelo menos Ctrl, Alt ou Super (sozinha, a tecla atrapalharia a digitação)."
                });
                return gtk::glib::Propagation::Stop;
            }
            if id == "copiar" && !matches!(key, Key::Return | Key::KP_Enter | Key::ISO_Enter) {
                warn.set_text("Aperte Enter junto com Ctrl, Alt, Shift ou Super (ex.: Ctrl+Enter).");
                return gtk::glib::Propagation::Stop;
            }
            if id == "copiar" && mods.is_empty() {
                warn.set_text("Só o Enter já cola; junte Ctrl, Alt, Shift ou Super.");
                return gtk::glib::Propagation::Stop;
            }
            // Shift+Tab chega como outra tecla; guardamos como Tab.
            let key = if key == Key::ISO_Left_Tab { Key::Tab } else { key };
            let accel = gtk::accelerator_name(key.to_lower(), mods).to_string();
            shown.set_text(&pretty(&accel));
            let clash = all().find(|s| s.0 != id && current(s.0) == accel);
            warn.set_text(&match clash {
                Some(c) => format!("Já é o atalho de \"{}\". Se salvar, os dois ficam iguais.", c.2),
                None if accel == "<Alt>c" && id != "lado" => "Alt+C é o atalho do histórico.".into(),
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
    {
        let win = win.clone();
        cancel.connect_clicked(move |_| win.close());
    }
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
