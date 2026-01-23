use gtk::prelude::*;
use gtk::gdk;
use libadwaita as adw;
use adw::prelude::*;
use clippit_core::Config;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_page() -> gtk::Widget {
    let config = Config::load().unwrap_or_default();

    // Create scrolled window
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);

    // Create preference page
    let page = adw::PreferencesPage::new();
    
    // Sugestões Group
    let suggestions_group = adw::PreferencesGroup::new();
    suggestions_group.set_title("Sugestões de Autocompletar");
    suggestions_group.set_description(Some("Configure o comportamento das sugestões durante a pesquisa"));

    // Enable suggestions switch
    let enable_row = adw::ActionRow::new();
    enable_row.set_title("Habilitar Sugestões");
    enable_row.set_subtitle("Mostra sugestões de palavras enquanto você digita");
    
    let icon1 = gtk::Image::from_icon_name("dialog-information-symbolic");
    enable_row.add_prefix(&icon1);
    
    let enable_switch = gtk::Switch::new();
    enable_switch.set_active(config.search.enable_suggestions);
    enable_switch.set_valign(gtk::Align::Center);
    
    // Auto-save on toggle
    enable_switch.connect_state_set(|_, state| {
        if let Ok(mut cfg) = Config::load() {
            cfg.search.enable_suggestions = state;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Sugestões atualizado: {}", state);
            }
        }
        gtk::glib::Propagation::Proceed
    });
    
    enable_row.set_activatable_widget(Some(&enable_switch));
    enable_row.add_suffix(&enable_switch);
    
    suggestions_group.add(&enable_row);

    // Max suggestions spin button
    let max_row = adw::ActionRow::new();
    max_row.set_title("Máximo de Sugestões");
    max_row.set_subtitle("Número máximo de sugestões exibidas (1-10)");
    
    let icon2 = gtk::Image::from_icon_name("view-list-symbolic");
    max_row.add_prefix(&icon2);
    
    let max_spin = gtk::SpinButton::with_range(1.0, 10.0, 1.0);
    max_spin.set_value(config.search.max_suggestions as f64);
    max_spin.set_valign(gtk::Align::Center);
    
    // Auto-save on value change
    max_spin.connect_value_changed(|spin| {
        if let Ok(mut cfg) = Config::load() {
            cfg.search.max_suggestions = spin.value() as usize;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Max sugestões atualizado: {}", spin.value());
            }
        }
    });
    
    max_row.add_suffix(&max_spin);
    suggestions_group.add(&max_row);
    
    page.add(&suggestions_group);

    // Atalhos de Teclado Group
    let hotkey_group = adw::PreferencesGroup::new();
    hotkey_group.set_title("Atalhos de Pesquisa");
    hotkey_group.set_description(Some("Configure os atalhos para voltar ao campo de pesquisa"));
    hotkey_group.set_margin_top(12);

    // Hotkey row with edit button (same pattern as Shortcuts page)
    let focus_hotkey_row = adw::ActionRow::new();
    focus_hotkey_row.set_title("Focar no campo de pesquisa");
    focus_hotkey_row.set_subtitle("Atalho para voltar o foco ao campo de pesquisa");
    
    let icon3 = gtk::Image::from_icon_name("input-keyboard-symbolic");
    focus_hotkey_row.add_prefix(&icon3);
    
    // Current hotkey label
    let focus_hotkey_label = gtk::Label::new(Some(&format!(
        "{} + {}",
        config.search.focus_search_modifier.to_uppercase(),
        config.search.focus_search_key.to_uppercase()
    )));
    focus_hotkey_label.add_css_class("dim-label");
    focus_hotkey_label.add_css_class("caption");
    focus_hotkey_label.add_css_class("monospace");
    
    // Edit button
    let focus_edit_button = gtk::Button::new();
    focus_edit_button.set_icon_name("document-edit-symbolic");
    focus_edit_button.set_valign(gtk::Align::Center);
    focus_edit_button.add_css_class("flat");
    focus_edit_button.set_tooltip_text(Some("Editar atalho"));
    
    let focus_button_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    focus_button_box.append(&focus_hotkey_label);
    focus_button_box.append(&focus_edit_button);
    
    focus_hotkey_row.add_suffix(&focus_button_box);
    
    hotkey_group.add(&focus_hotkey_row);
    page.add(&hotkey_group);

    // Setup edit dialog for focus hotkey
    let focus_hotkey_label_clone = focus_hotkey_label.clone();
    focus_edit_button.connect_clicked(move |btn| {
        if let Some(window) = btn.root().and_downcast::<gtk::Window>() {
            show_focus_hotkey_dialog(&window, focus_hotkey_label_clone.clone());
        }
    });

    // No need for save button - auto-save enabled
    page.set_margin_start(12);
    page.set_margin_end(12);
    page.set_margin_top(12);
    page.set_margin_bottom(12);

    scrolled.set_child(Some(&page));
    scrolled.upcast()
}

fn show_focus_hotkey_dialog(parent: &gtk::Window, label: gtk::Label) {
    // Create dialog using gtk::Window SEM DECORAÇÃO (modal limpo)
    let dialog = gtk::Window::builder()
        .modal(true)
        .transient_for(parent)
        .default_width(480)
        .default_height(340)
        .resizable(false)
        .decorated(false)  // ✅ SEM HEADER DO SISTEMA!
        .build();
    
    dialog.add_css_class("background");
    
    // Aplicar CSS para cantos arredondados
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
        }"
    );
    
    // Aplicar CSS
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

    // Instruction label
    let instruction = gtk::Label::new(Some("Pressione a combinação de teclas"));
    instruction.add_css_class("title-1");
    content.append(&instruction);

    // Current keys display
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
            if key_str == "control_l" || key_str == "control_r" ||
               key_str == "alt_l" || key_str == "alt_r" ||
               key_str == "shift_l" || key_str == "shift_r" ||
               key_str == "super_l" || key_str == "super_r" ||
               key_str == "meta_l" || key_str == "meta_r" {
                return gtk::glib::Propagation::Proceed;
            }
            
            // Clean up key name
            let clean_key = key_str
                .replace("_l", "")
                .replace("_r", "")
                .to_lowercase();
            
            // Update display
            let display_text = if !modifier_str.is_empty() {
                format!("{} + {}", modifier_str.to_uppercase(), clean_key.to_uppercase())
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
            
            // Update label, save config, and close dialog automatically after 500ms
            let label_clone = label_clone.clone();
            let dialog_clone = dialog_clone.clone();
            let captured_mod = captured_modifier.clone();
            let captured_k = captured_key.clone();
            
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                label_clone.set_text(&display_text);
                
                // Auto-save hotkey configuration
                if let Ok(mut cfg) = Config::load() {
                    cfg.search.focus_search_modifier = captured_mod.borrow().clone();
                    cfg.search.focus_search_key = captured_k.borrow().clone();
                    
                    if let Err(e) = cfg.save() {
                        eprintln!("❌ Erro ao salvar: {}", e);
                    } else {
                        eprintln!("✅ Atalho de foco atualizado: {} + {}", 
                            captured_mod.borrow(), captured_k.borrow());
                    }
                }
                
                dialog_clone.close();
            });
        }
        
        gtk::glib::Propagation::Stop
    });
    
    dialog.add_controller(key_controller);
    
    main_container.append(&content);
    dialog.set_child(Some(&main_container));
    dialog.present();
}
