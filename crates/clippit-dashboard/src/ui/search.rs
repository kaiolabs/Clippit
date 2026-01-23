use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use clippit_core::Config;

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
    max_row.add_suffix(&max_spin);
    
    suggestions_group.add(&max_row);
    
    page.add(&suggestions_group);

    // Atalhos de Teclado Group
    let hotkey_group = adw::PreferencesGroup::new();
    hotkey_group.set_title("Atalhos de Pesquisa");
    hotkey_group.set_description(Some("Configure os atalhos para voltar ao campo de pesquisa"));
    hotkey_group.set_margin_top(12);

    // Focus search modifier
    let modifier_row = adw::ComboRow::new();
    modifier_row.set_title("Modificador");
    modifier_row.set_subtitle("Tecla modificadora para focar no campo de pesquisa");
    
    let icon3 = gtk::Image::from_icon_name("input-keyboard-symbolic");
    modifier_row.add_prefix(&icon3);
    
    let modifier_model = gtk::StringList::new(&["ctrl", "alt", "super", "shift"]);
    modifier_row.set_model(Some(&modifier_model));
    
    let current_modifier = config.search.focus_search_modifier.as_str();
    let modifier_index = match current_modifier {
        "ctrl" => 0,
        "alt" => 1,
        "super" => 2,
        "shift" => 3,
        _ => 0,
    };
    modifier_row.set_selected(modifier_index);
    
    hotkey_group.add(&modifier_row);

    // Focus search key
    let key_row = adw::ComboRow::new();
    key_row.set_title("Tecla");
    key_row.set_subtitle("Tecla para focar no campo de pesquisa");
    
    let icon4 = gtk::Image::from_icon_name("input-keyboard-symbolic");
    key_row.add_prefix(&icon4);
    
    let key_model = gtk::StringList::new(&["p", "f", "s", "/"]);
    key_row.set_model(Some(&key_model));
    
    let current_key = config.search.focus_search_key.as_str();
    let key_index = match current_key {
        "p" => 0,
        "f" => 1,
        "s" => 2,
        "/" => 3,
        _ => 0,
    };
    key_row.set_selected(key_index);
    
    hotkey_group.add(&key_row);
    
    page.add(&hotkey_group);

    // Save button
    let save_group = adw::PreferencesGroup::new();
    save_group.set_margin_top(24);
    
    let save_button = gtk::Button::with_label("Salvar Configurações");
    save_button.add_css_class("suggested-action");
    save_button.set_halign(gtk::Align::Center);
    
    // Clone for closure
    let save_button_clone = save_button.clone();
    
    // Connect save button
    save_button_clone.connect_clicked(move |btn| {
        if let Ok(mut cfg) = Config::load() {
            cfg.search.enable_suggestions = enable_switch.is_active();
            cfg.search.max_suggestions = max_spin.value() as usize;
            
            let modifier_idx = modifier_row.selected();
            cfg.search.focus_search_modifier = match modifier_idx {
                0 => "ctrl".to_string(),
                1 => "alt".to_string(),
                2 => "super".to_string(),
                3 => "shift".to_string(),
                _ => "ctrl".to_string(),
            };
            
            let key_idx = key_row.selected();
            cfg.search.focus_search_key = match key_idx {
                0 => "p".to_string(),
                1 => "f".to_string(),
                2 => "s".to_string(),
                3 => "/".to_string(),
                _ => "p".to_string(),
            };
            
            match cfg.save() {
                Ok(_) => {
                    eprintln!("✅ Configurações de pesquisa salvas com sucesso");
                    
                    // Show toast notification
                    if let Some(window) = btn.root().and_downcast::<adw::ApplicationWindow>() {
                        let toast = adw::Toast::new("Configurações salvas com sucesso!");
                        toast.set_timeout(3);
                        
                        // Find toast overlay in window hierarchy
                        if let Some(toast_overlay) = find_toast_overlay(&window) {
                            toast_overlay.add_toast(toast);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Erro ao salvar configurações: {}", e);
                }
            }
        }
    });
    
    save_group.add(&save_button);
    page.add(&save_group);

    scrolled.set_child(Some(&page));
    scrolled.upcast()
}

// Helper function to find toast overlay
fn find_toast_overlay(widget: &impl IsA<gtk::Widget>) -> Option<adw::ToastOverlay> {
    let mut current: Option<gtk::Widget> = Some(widget.clone().upcast());
    
    while let Some(w) = current {
        if let Ok(overlay) = w.clone().downcast::<adw::ToastOverlay>() {
            return Some(overlay);
        }
        current = w.parent();
    }
    
    None
}
