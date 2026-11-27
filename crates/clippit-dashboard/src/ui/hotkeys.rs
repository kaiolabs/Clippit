use adw::prelude::*;
use clippit_core::Config;
use gtk::gdk;
use gtk::prelude::*;
use libadwaita as adw;
use rust_i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

/// Converte formato Clippit (ctrl+kp_1) para formato GNOME (<Primary>KP_1)
fn convert_to_gnome_format(modifier: &str, key: &str) -> String {
    let mut parts = Vec::new();

    // Converter modificadores
    for mod_part in modifier.split('+') {
        let mod_trimmed = mod_part.trim().to_lowercase();
        match mod_trimmed.as_str() {
            "ctrl" | "control" => parts.push("Primary"),
            "super" | "meta" | "win" => parts.push("Super"),
            "alt" => parts.push("Alt"),
            "shift" => parts.push("Shift"),
            "none" | "" => {}
            _ => {}
        }
    }

    // Converter tecla
    let key_upper = key.to_uppercase();
    let gnome_key = if let Some(num) = key.strip_prefix("kp_") {
        // Numpad keys: kp_1 -> KP_1
        format!("KP_{num}")
    } else if key.len() == 1 {
        // Single letter: v -> v
        key_upper.clone()
    } else {
        // Function keys, etc: f1 -> F1
        key_upper.clone()
    };

    // Montar formato GNOME: <Primary><Super>KP_1
    if parts.is_empty() {
        gnome_key
    } else {
        format!("<{}>{}", parts.join("><"), gnome_key)
    }
}

pub fn create_page() -> gtk::Widget {
    let config = Config::load().unwrap_or_default();
    let page = adw::PreferencesPage::new();

    // Único atalho que dá para trocar aqui (vale no sistema todo).
    let group = adw::PreferencesGroup::new();
    group.set_title("Histórico");
    let hotkey_row = adw::ActionRow::new();
    hotkey_row.set_title("Abrir o histórico do que você copiou");
    hotkey_row.set_subtitle("Funciona em qualquer lugar do sistema");
    let hotkey_label = gtk::Label::new(Some(&format!(
        "{}+{}",
        pretty(&config.hotkeys.show_history_modifier),
        config.hotkeys.show_history_key.to_uppercase()
    )));
    hotkey_label.add_css_class("clip-key");
    hotkey_label.set_valign(gtk::Align::Center);
    let edit_button = gtk::Button::with_label("Trocar");
    edit_button.add_css_class("clip-btn");
    edit_button.set_valign(gtk::Align::Center);
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    button_box.append(&hotkey_label);
    button_box.append(&edit_button);
    hotkey_row.add_suffix(&button_box);
    group.add(&hotkey_row);
    page.add(&group);

    // Atalhos da extensão: cada um dá para trocar (vale na hora, sem reiniciar).
    let ext = adw::PreferencesGroup::new();
    ext.set_title("Lançador, IA e corretor");
    for (id, _def, title, sub) in super::atalhos::SHORTCUTS {
        ext.add(&super::atalhos::editable_row(id, title, sub));
    }
    page.add(&ext);

    // Atalhos de dentro do lançador: também dá para trocar (valem na hora, sem reiniciar).
    for (title, ids) in [
        (
            "Dentro do lançador",
            &["fechar", "voltar", "lado", "copiar"][..],
        ),
        (
            "No chat da IA",
            &["ditado", "nova_conversa", "conversas"][..],
        ),
        (
            "No histórico",
            &["transformar", "fixar", "apagar", "ampliar"][..],
        ),
        ("Nos emojis", &["categoria"][..]),
    ] {
        let group = adw::PreferencesGroup::new();
        group.set_title(title);
        for id in ids {
            if let Some((id, _def, title, sub)) = super::atalhos::INNER.iter().find(|s| s.0 == *id)
            {
                group.add(&super::atalhos::editable_row(id, title, sub));
            }
        }
        page.add(&group);
    }

    let hotkey_label_clone = hotkey_label.clone();
    edit_button.connect_clicked(move |btn| {
        if let Some(window) = btn.root().and_downcast::<gtk::Window>() {
            show_hotkey_dialog(&window, hotkey_label_clone.clone());
        }
    });

    page.upcast()
}

fn pretty(modifier: &str) -> String {
    match modifier.to_lowercase().as_str() {
        "alt" => "Alt".into(),
        "ctrl" | "control" => "Ctrl".into(),
        "super" | "meta" => "Super".into(),
        "shift" => "Shift".into(),
        other => other.to_string(),
    }
}

fn show_hotkey_dialog(parent: &gtk::Window, label: gtk::Label) {
    // Create dialog using gtk::Window SEM DECORAÇÃO (modal limpo)
    let dialog = gtk::Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(480)
        .default_height(340)
        .resizable(false)
        .decorated(false) // ✅ SEM HEADER DO SISTEMA!
        .build();

    dialog.add_css_class("background");

    // Aplicar CSS para cantos arredondados e animação CSS nativa (API moderna GTK4 4.10+)
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(
        "window {
            border-radius: 16px;
            background-color: @window_bg_color;
            box-shadow: 0 8px 24px rgba(0,0,0,0.3);
            animation: fadeIn 150ms ease-out;
        }
        @keyframes fadeIn {
            from { opacity: 0; transform: scale(0.95); }
            to { opacity: 1; transform: scale(1); }
        }",
    );

    // Usar API moderna para GTK4 4.10+
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // Container principal com padding
    let main_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_container.set_margin_start(32);
    main_container.set_margin_end(32);
    main_container.set_margin_top(24);
    main_container.set_margin_bottom(24);

    // Botão CANCELAR no topo (traduzido)
    let cancel_button = gtk::Button::new();
    cancel_button.set_label(&t!("hotkeys.cancel"));
    cancel_button.set_icon_name("window-close-symbolic");
    cancel_button.add_css_class("flat");
    cancel_button.set_halign(gtk::Align::End);
    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });
    main_container.append(&cancel_button);

    // Content area - MINIMALISTA
    let content = gtk::Box::new(gtk::Orientation::Vertical, 20);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    content.set_vexpand(true);

    // Icon
    let icon = gtk::Image::from_icon_name("input-keyboard-symbolic");
    icon.set_pixel_size(72);
    icon.add_css_class("accent");
    content.append(&icon);

    // Instruction label - SIMPLES E DIRETO
    let instruction = gtk::Label::new(Some(&t!("hotkeys.press_combination")));
    instruction.add_css_class("title-1");
    content.append(&instruction);

    // Current keys display - GRANDE E VISÍVEL
    let keys_display = gtk::Label::new(Some("⌨️"));
    keys_display.add_css_class("title-1");
    keys_display.set_margin_top(12);
    content.append(&keys_display);

    // Captured keys storage
    let captured_modifier = Rc::new(RefCell::new(String::new()));
    let captured_key = Rc::new(RefCell::new(String::new()));

    // Key event controller
    let key_controller = gtk::EventControllerKey::new();
    let keys_display_clone = keys_display.clone();
    let captured_modifier_clone = captured_modifier.clone();
    let captured_key_clone = captured_key.clone();
    let dialog_clone = dialog.clone();
    let label_clone = label.clone();

    key_controller.connect_key_pressed(move |_, keyval, _keycode, modifier| {
        let mut modifier_str = String::new();

        // Detect modifiers
        if modifier.contains(gdk::ModifierType::CONTROL_MASK) {
            modifier_str.push_str("ctrl");
        }
        if modifier.contains(gdk::ModifierType::ALT_MASK) {
            if !modifier_str.is_empty() {
                modifier_str.push('+');
            }
            modifier_str.push_str("alt");
        }
        if modifier.contains(gdk::ModifierType::SHIFT_MASK) {
            if !modifier_str.is_empty() {
                modifier_str.push('+');
            }
            modifier_str.push_str("shift");
        }
        if modifier.contains(gdk::ModifierType::SUPER_MASK) {
            if !modifier_str.is_empty() {
                modifier_str.push('+');
            }
            modifier_str.push_str("super");
        }

        // Get key name
        let key_name = keyval.name();
        if let Some(key) = key_name {
            let key_str = key.to_lowercase();

            // Ignore modifier-only presses
            if key_str == "control_l"
                || key_str == "control_r"
                || key_str == "alt_l"
                || key_str == "alt_r"
                || key_str == "shift_l"
                || key_str == "shift_r"
                || key_str == "super_l"
                || key_str == "super_r"
                || key_str == "meta_l"
                || key_str == "meta_r"
            {
                return gtk::glib::Propagation::Proceed;
            }

            // Clean up key name
            let clean_key = key_str.replace("_l", "").replace("_r", "").to_lowercase();

            // Update display
            let display_text = if !modifier_str.is_empty() {
                format!(
                    "{}+{}",
                    modifier_str.to_uppercase(),
                    clean_key.to_uppercase()
                )
            } else {
                clean_key.to_uppercase()
            };

            keys_display_clone.set_text(&display_text);

            // Store captured keys
            *captured_modifier_clone.borrow_mut() = if modifier_str.is_empty() {
                "none".to_string()
            } else {
                modifier_str
            };
            *captured_key_clone.borrow_mut() = clean_key;

            // Save after a short delay
            let dialog_weak = dialog_clone.downgrade();
            let label_weak = label_clone.downgrade();
            let modifier_final = captured_modifier_clone.borrow().clone();
            let key_final = captured_key_clone.borrow().clone();

            gtk::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                if let (Some(dialog), Some(label)) = (dialog_weak.upgrade(), label_weak.upgrade()) {
                    // Save to config
                    let mut config = Config::load().unwrap_or_default();
                    config.hotkeys.show_history_modifier = modifier_final.clone();
                    config.hotkeys.show_history_key = key_final.clone();

                    if config.save().is_ok() {
                        // Update label
                        label.set_text(&format!(
                            "{}+{}",
                            pretty(&modifier_final),
                            key_final.to_uppercase()
                        ));

                        // Mostrar mensagem de sucesso
                        eprintln!(
                            "✅ Atalho salvo: {} + {}",
                            modifier_final.to_uppercase(),
                            key_final.to_uppercase()
                        );

                        // 🔄 SYNC: Atualizar gsettings do GNOME para refletir globalmente!
                        eprintln!("🔄 Sincronizando com GNOME gsettings...");
                        let gnome_binding = convert_to_gnome_format(&modifier_final, &key_final);
                        // Fora do GNOME o clippit-shell lê o atalho do corretor.conf,
                        // no mesmo formato dos outros atalhos (<Control>, não <Primary>).
                        super::conf::set(
                            "atalho_historico",
                            &gnome_binding.replace("<Primary>", "<Control>"),
                        );
                        let sync_result = std::process::Command::new("gsettings")
                            .args([
                                "set",
                                "org.gnome.settings-daemon.plugins.media-keys.custom-keybinding:/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/clippit/",
                                "binding",
                                &gnome_binding,
                            ])
                            .output();

                        if sync_result.is_ok() {
                            eprintln!("✅ Atalho sincronizado com GNOME: {}", gnome_binding);
                        } else {
                            eprintln!("⚠️  Não foi possível sincronizar com GNOME (não afeta funcionamento)");
                        }

                        // Alterar conteúdo do dialog para mostrar sucesso (centralizado)
                        let success_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
                        success_box.set_valign(gtk::Align::Center);
                        success_box.set_halign(gtk::Align::Center);
                        success_box.set_vexpand(true);
                        success_box.set_hexpand(true);

                        let success_icon = gtk::Label::new(Some("✓"));
                        success_icon.add_css_class("title-1");
                        success_icon.add_css_class("success");
                        success_box.append(&success_icon);

                        let success_text = gtk::Label::new(Some(&t!("hotkeys.shortcut_saved")));
                        success_text.add_css_class("title-2");
                        success_box.append(&success_text);

                        let hotkey_text = gtk::Label::new(Some(&format!(
                            "{}+{}",
                            pretty(&modifier_final),
                            key_final.to_uppercase()
                        )));
                        hotkey_text.add_css_class("title-3");
                        hotkey_text.add_css_class("accent");
                        success_box.append(&hotkey_text);

                        let status_text = gtk::Label::new(Some(&t!("hotkeys.daemon_restarting")));
                        status_text.add_css_class("dim-label");
                        success_box.append(&status_text);

                        // Atualizar dialog
                        if let Some(main_box) = dialog.child().and_downcast::<gtk::Box>() {
                            while let Some(child) = main_box.first_child() {
                                main_box.remove(&child);
                            }
                            main_box.append(&success_box);
                        }

                        // Restart daemon automatically to apply new hotkey
                        eprintln!("🔄 Reiniciando daemon...");

                        let dialog_for_close = dialog.clone();
                        let status_text_clone = status_text.clone();

                        // Reiniciar daemon em background
                        gtk::glib::timeout_add_local_once(
                            std::time::Duration::from_millis(100),
                            move || {
                                let restart_result = std::process::Command::new("systemctl")
                                    .args(["--user", "restart", "clippit"])
                                    .output();

                                if restart_result.is_ok() {
                                    eprintln!("✅ Daemon reiniciado! Atalho ativo!");
                                    status_text_clone.set_text(&t!("hotkeys.daemon_restarted"));
                                } else {
                                    eprintln!("⚠️  Execute: systemctl --user restart clippit");
                                    status_text_clone.set_text(&t!("hotkeys.daemon_error"));
                                }

                                // Fechar imediatamente com animação suave
                                let dialog_for_fade = dialog_for_close.clone();
                                gtk::glib::timeout_add_local(
                                    std::time::Duration::from_millis(10),
                                    move || {
                                        let target_opacity = dialog_for_fade.opacity() - 0.15;
                                        if target_opacity > 0.0 {
                                            dialog_for_fade.set_opacity(target_opacity);
                                            gtk::glib::ControlFlow::Continue
                                        } else {
                                            dialog_for_fade.destroy();
                                            gtk::glib::ControlFlow::Break
                                        }
                                    },
                                );
                            },
                        );
                    }
                }
                gtk::glib::ControlFlow::Break
            });
        }

        gtk::glib::Propagation::Stop
    });

    dialog.add_controller(key_controller);

    // Adicionar content ao container
    main_container.append(&content);

    dialog.set_child(Some(&main_container));

    // ✅ Apresentar modal INSTANTANEAMENTE (animação via CSS)
    dialog.present();
}
