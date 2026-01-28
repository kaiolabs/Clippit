use clippit_core::Config;
use gtk::gdk;
use gtk::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_page() -> gtk::Widget {
    let page = adw::PreferencesPage::new();
    page.set_title("Autocompletar Global");
    page.set_icon_name(Some("input-keyboard-symbolic"));

    // ========== SEÇÃO: CONFIGURAÇÕES GERAIS ==========
    let group_general = adw::PreferencesGroup::new();
    group_general.set_title("Configurações Gerais");
    group_general.set_description(Some("Configure o autocomplete global do sistema"));

    // Habilitar/Desabilitar
    let row_enable = adw::ActionRow::new();
    row_enable.set_title("Habilitar Autocomplete Global");
    row_enable.set_subtitle("Sugestões de palavras em qualquer aplicação");

    let config = Config::load().unwrap_or_default();

    let switch_enable = gtk::Switch::new();
    switch_enable.set_active(config.autocomplete.enabled);
    switch_enable.set_valign(gtk::Align::Center);

    switch_enable.connect_active_notify(|switch| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.enabled = switch.is_active();
        let _ = cfg.save();
        eprintln!("✅ Autocomplete enabled: {}", switch.is_active());

        // Reiniciar daemon automaticamente
        eprintln!("🔄 Reiniciando daemon para aplicar configuração...");
        let restart_result = std::process::Command::new("systemctl")
            .args(&["--user", "restart", "clippit"])
            .output();

        if restart_result.is_ok() {
            eprintln!("✅ Daemon reiniciado com sucesso!");
        } else {
            eprintln!("⚠️  Execute manualmente: systemctl --user restart clippit");
        }
    });

    row_enable.add_suffix(&switch_enable);
    row_enable.set_activatable_widget(Some(&switch_enable));

    group_general.add(&row_enable);

    // Máximo de Sugestões
    let row_max = adw::ActionRow::new();
    row_max.set_title("Máximo de Sugestões");
    row_max.set_subtitle("Quantidade de sugestões a mostrar (1-10)");

    let spin_max = gtk::SpinButton::with_range(1.0, 10.0, 1.0);
    spin_max.set_value(config.autocomplete.max_suggestions as f64);
    spin_max.set_valign(gtk::Align::Center);

    spin_max.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.max_suggestions = spin.value() as usize;
        let _ = cfg.save();
        eprintln!("✅ Max suggestions: {}", spin.value());
    });

    row_max.add_suffix(&spin_max);
    group_general.add(&row_max);

    // Caracteres Mínimos
    let row_min_chars = adw::ActionRow::new();
    row_min_chars.set_title("Caracteres Mínimos");
    row_min_chars.set_subtitle("Mínimo de letras para mostrar sugestões");

    let spin_min = gtk::SpinButton::with_range(1.0, 5.0, 1.0);
    spin_min.set_value(config.autocomplete.min_chars as f64);
    spin_min.set_valign(gtk::Align::Center);

    spin_min.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.min_chars = spin.value() as usize;
        let _ = cfg.save();
        eprintln!("✅ Min chars: {}", spin.value());
    });

    row_min_chars.add_suffix(&spin_min);
    group_general.add(&row_min_chars);

    // Delay
    let row_delay = adw::ActionRow::new();
    row_delay.set_title("Atraso (ms)");
    row_delay.set_subtitle("Tempo antes de mostrar sugestões (100-1000ms)");

    let spin_delay = gtk::SpinButton::with_range(100.0, 1000.0, 50.0);
    spin_delay.set_value(config.autocomplete.delay_ms as f64);
    spin_delay.set_valign(gtk::Align::Center);

    spin_delay.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.delay_ms = spin.value() as u64;
        let _ = cfg.save();
        eprintln!("✅ Delay: {}ms", spin.value());
    });

    row_delay.add_suffix(&spin_delay);
    group_general.add(&row_delay);

    page.add(&group_general);

    // ========== SEÇÃO: ATALHO DE TECLADO ==========
    let group_hotkey = adw::PreferencesGroup::new();
    group_hotkey.set_title("Atalho de Teclado");
    group_hotkey.set_description(Some(
        "Configure o atalho para ativar/desativar autocomplete temporariamente",
    ));

    let hotkey_row = adw::ActionRow::new();
    hotkey_row.set_title("Alternar Autocomplete");
    hotkey_row.set_subtitle("Atalho para ativar/desativar temporariamente");

    let icon_keyboard = gtk::Image::from_icon_name("input-keyboard-symbolic");
    hotkey_row.add_prefix(&icon_keyboard);

    // Label mostrando hotkey atual
    let hotkey_label = gtk::Label::new(Some(&format!(
        "{} + {}",
        config.autocomplete.toggle_modifier, config.autocomplete.toggle_key
    )));
    hotkey_label.add_css_class("dim-label");
    hotkey_label.add_css_class("caption");
    hotkey_label.add_css_class("monospace");

    // Botão de editar
    let edit_button = gtk::Button::new();
    edit_button.set_icon_name("document-edit-symbolic");
    edit_button.set_valign(gtk::Align::Center);
    edit_button.add_css_class("flat");
    edit_button.set_tooltip_text(Some("Editar atalho"));

    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    button_box.append(&hotkey_label);
    button_box.append(&edit_button);

    hotkey_row.add_suffix(&button_box);
    group_hotkey.add(&hotkey_row);

    // Setup edit dialog
    let hotkey_label_clone = hotkey_label.clone();
    edit_button.connect_clicked(move |btn| {
        if let Some(window) = btn.root().and_downcast::<gtk::Window>() {
            show_autocomplete_hotkey_dialog(&window, hotkey_label_clone.clone());
        }
    });

    page.add(&group_hotkey);

    // ========== SEÇÃO: PRIVACIDADE E SEGURANÇA ==========
    let group_privacy = adw::PreferencesGroup::new();
    group_privacy.set_title("Privacidade e Segurança");

    // Mostrar em Senhas
    let row_passwords = adw::ActionRow::new();
    row_passwords.set_title("Mostrar em Campos de Senha");
    row_passwords.set_subtitle("⚠️ Não recomendado por segurança");

    let switch_passwords = gtk::Switch::new();
    switch_passwords.set_active(config.autocomplete.show_in_passwords);
    switch_passwords.set_valign(gtk::Align::Center);

    switch_passwords.connect_active_notify(|switch| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.show_in_passwords = switch.is_active();
        let _ = cfg.save();
        eprintln!("✅ Show in passwords: {}", switch.is_active());
    });

    row_passwords.add_suffix(&switch_passwords);
    row_passwords.set_activatable_widget(Some(&switch_passwords));

    group_privacy.add(&row_passwords);

    // Apps Ignorados
    let row_ignored = adw::ExpanderRow::new();
    row_ignored.set_title("Aplicativos Ignorados");
    row_ignored.set_subtitle("Apps onde autocomplete não funciona");
    row_ignored.set_expanded(false);

    // Lista de apps ignorados
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_min_content_height(150);
    scrolled.set_margin_start(12);
    scrolled.set_margin_end(12);
    scrolled.set_margin_top(6);
    scrolled.set_margin_bottom(6);

    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_margin_start(12);
    text_view.set_margin_end(12);
    text_view.set_margin_top(12);
    text_view.set_margin_bottom(12);

    let buffer = text_view.buffer();
    buffer.set_text(&config.autocomplete.ignored_apps.join("\n"));

    buffer.connect_changed(|buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let apps: Vec<String> = text
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.ignored_apps = apps;
        let _ = cfg.save();
        eprintln!("✅ Ignored apps updated");
    });

    scrolled.set_child(Some(&text_view));

    let ignored_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    ignored_box.append(&scrolled);

    let help_label = gtk::Label::new(Some("Um app por linha (ex: gnome-terminal, keepassxc)"));
    help_label.add_css_class("dim-label");
    help_label.set_margin_start(12);
    help_label.set_margin_end(12);
    help_label.set_margin_bottom(6);
    ignored_box.append(&help_label);

    row_ignored.add_row(&ignored_box);
    group_privacy.add(&row_ignored);

    page.add(&group_privacy);

    // ========== SEÇÃO: INSTALAÇÃO ==========
    let group_setup = adw::PreferencesGroup::new();
    group_setup.set_title("Instalação");
    group_setup.set_description(Some("Configure o IBus Input Method"));

    let row_install = adw::ActionRow::new();
    row_install.set_title("Como Ativar");
    row_install.set_subtitle("Configurações → Teclado → Fontes de Entrada → Adicionar \"Clippit\"");

    let icon_info = gtk::Image::from_icon_name("dialog-information-symbolic");
    row_install.add_prefix(&icon_info);

    group_setup.add(&row_install);

    // Botão de verificação
    let row_check = adw::ActionRow::new();
    row_check.set_title("Verificar Instalação");
    row_check.set_subtitle("Verifica se o IBus component está instalado");

    let btn_check = gtk::Button::with_label("Verificar");
    btn_check.add_css_class("suggested-action");
    btn_check.set_valign(gtk::Align::Center);

    let row_check_clone = row_check.clone();
    btn_check.connect_clicked(move |btn| {
        // Desabilitar botão durante verificação
        btn.set_sensitive(false);
        btn.set_label("Verificando...");
        
        // Verificar se IBus está instalado
        use std::process::Command;
        
        let result = Command::new("ibus")
            .arg("list-engine")
            .output();
        
        let message = match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("clippit") {
                    "✅ Clippit IBus engine está instalado e pronto para uso!"
                } else {
                    "⚠️ Clippit IBus engine NÃO encontrado.\n\nExecute no terminal:\nsudo bash scripts/install-ibus.sh"
                }
            }
            Err(_) => {
                "❌ IBus não está instalado no sistema.\n\nInstale com:\nsudo apt install ibus"
            }
        };
        
        // Criar diálogo de resultado
        let parent_window = btn.root().and_downcast::<gtk::Window>();
        let dialog = adw::MessageDialog::new(
            parent_window.as_ref(),
            Some("Verificação do IBus"),
            Some(message),
        );
        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));
        dialog.set_close_response("ok");
        
        let btn_clone = btn.clone();
        dialog.connect_response(None, move |dialog, _| {
            dialog.close();
            btn_clone.set_sensitive(true);
            btn_clone.set_label("Verificar");
        });
        
        dialog.present();
        
        // Também atualizar o subtitle da row
        row_check_clone.set_subtitle(message);
    });

    row_check.add_suffix(&btn_check);
    group_setup.add(&row_check);

    page.add(&group_setup);

    // ========== SEÇÃO: IA (FASE 2 - FUTURO) ==========
    let group_ai = adw::PreferencesGroup::new();
    group_ai.set_title("Inteligência Artificial (Fase 2)");
    group_ai.set_description(Some("🚧 Em desenvolvimento - Sugestões via IA"));

    let row_ai_enable = adw::ActionRow::new();
    row_ai_enable.set_title("Habilitar Sugestões IA");
    row_ai_enable.set_subtitle("Sugestões contextuais via modelo de IA");

    let switch_ai = gtk::Switch::new();
    switch_ai.set_active(config.autocomplete.ai.enabled);
    switch_ai.set_valign(gtk::Align::Center);
    switch_ai.set_sensitive(false); // Desabilitado por enquanto

    row_ai_enable.add_suffix(&switch_ai);
    row_ai_enable.set_activatable_widget(Some(&switch_ai));

    group_ai.add(&row_ai_enable);

    page.add(&group_ai);

    page.upcast()
}

fn show_autocomplete_hotkey_dialog(parent: &gtk::Window, label: gtk::Label) {
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

    // Botão CANCELAR no topo
    let cancel_button = gtk::Button::new();
    cancel_button.set_label("Cancelar");
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
    let instruction = gtk::Label::new(Some("Pressione a combinação de teclas desejada..."));
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
                modifier_str.push_str("+");
            }
            modifier_str.push_str("alt");
        }
        if modifier.contains(gdk::ModifierType::SHIFT_MASK) {
            if !modifier_str.is_empty() {
                modifier_str.push_str("+");
            }
            modifier_str.push_str("shift");
        }
        if modifier.contains(gdk::ModifierType::SUPER_MASK) {
            if !modifier_str.is_empty() {
                modifier_str.push_str("+");
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
                    "{} + {}",
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
                    // Save to config (autocomplete)
                    let mut config = Config::load().unwrap_or_default();
                    config.autocomplete.toggle_modifier = modifier_final.clone();
                    config.autocomplete.toggle_key = key_final.clone();

                    if config.save().is_ok() {
                        // Update label
                        label.set_text(&format!(
                            "{} + {}",
                            modifier_final.to_uppercase(),
                            key_final.to_uppercase()
                        ));

                        // Mostrar mensagem de sucesso
                        eprintln!(
                            "✅ Atalho de autocomplete salvo: {} + {}",
                            modifier_final.to_uppercase(),
                            key_final.to_uppercase()
                        );

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

                        let success_text = gtk::Label::new(Some("Atalho Salvo"));
                        success_text.add_css_class("title-2");
                        success_box.append(&success_text);

                        let hotkey_text = gtk::Label::new(Some(&format!(
                            "{} + {}",
                            modifier_final.to_uppercase(),
                            key_final.to_uppercase()
                        )));
                        hotkey_text.add_css_class("title-3");
                        hotkey_text.add_css_class("accent");
                        success_box.append(&hotkey_text);

                        let status_text = gtk::Label::new(Some("Reiniciando daemon..."));
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
                                    .args(&["--user", "restart", "clippit"])
                                    .output();

                                if restart_result.is_ok() {
                                    eprintln!("✅ Daemon reiniciado! Atalho ativo!");
                                    status_text_clone.set_text("Daemon reiniciado com sucesso!");
                                } else {
                                    eprintln!("⚠️  Execute: systemctl --user restart clippit");
                                    status_text_clone
                                        .set_text("Erro ao reiniciar. Execute manualmente.");
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
