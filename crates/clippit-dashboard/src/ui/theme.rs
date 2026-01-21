use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use clippit_core::{Config, set_language};
use rust_i18n::t;

pub fn create_page() -> gtk::Widget {
    let config = Config::load().unwrap_or_default();

    // Create scrolled window
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);

    // Create preference page
    let page = adw::PreferencesPage::new();
    
    // Create preference group
    let group = adw::PreferencesGroup::new();
    group.set_title(&t!("theme.title"));
    group.set_description(Some(&t!("theme.description")));

    // Theme selection
    let theme_row = adw::ComboRow::new();
    theme_row.set_title(&t!("theme.theme_label"));
    theme_row.set_subtitle(&t!("theme.theme_desc"));
    
    let icon1 = gtk::Image::from_icon_name("weather-clear-night-symbolic");
    theme_row.add_prefix(&icon1);
    
    let themes = gtk::StringList::new(&["Sistema", "Dark", "Light"]);
    theme_row.set_model(Some(&themes));
    
    match config.ui.theme.as_str() {
        "system" => theme_row.set_selected(0),
        "dark" => theme_row.set_selected(1),
        "light" => theme_row.set_selected(2),
        _ => theme_row.set_selected(0),
    }
    
    group.add(&theme_row);

    // Language selection
    let lang_row = adw::ComboRow::new();
    lang_row.set_title(&t!("theme.language"));
    lang_row.set_subtitle(&t!("theme.language_desc"));
    
    let icon_lang = gtk::Image::from_icon_name("preferences-desktop-locale-symbolic");
    lang_row.add_prefix(&icon_lang);
    
    let languages = gtk::StringList::new(&["English", "Português"]);
    lang_row.set_model(Some(&languages));
    
    match config.ui.language.as_str() {
        "en" => lang_row.set_selected(0),
        "pt" => lang_row.set_selected(1),
        _ => lang_row.set_selected(0),
    }
    
    group.add(&lang_row);

    // Font family - SELETOR DE FONTES DO SISTEMA
    let font_row = adw::ActionRow::new();
    font_row.set_title(&t!("theme.font"));
    
    // Mostrar fonte atual ou mensagem amigável
    let font_subtitle = if config.ui.font_family.is_empty() {
        "Nenhuma fonte selecionada".to_string()
    } else {
        config.ui.font_family.clone()
    };
    font_row.set_subtitle(&font_subtitle);
    
    let icon2 = gtk::Image::from_icon_name("font-x-generic-symbolic");
    font_row.add_prefix(&icon2);
    
    // Botão de seleção de fonte (compatível com GTK4 4.6)
    let font_button = gtk::FontButton::new();
    font_button.set_use_font(false); // ✅ NÃO aplicar fonte ao botão (evita ícones estranhos)
    font_button.set_use_size(false); // Não mostra tamanho (temos SpinButton separado)
    
    // Configurar fonte inicial (se houver)
    if !config.ui.font_family.is_empty() {
        let initial_font_desc = pango::FontDescription::from_string(&config.ui.font_family);
        font_button.set_font_desc(&initial_font_desc);
    }
    
    font_row.add_suffix(&font_button);
    font_row.set_activatable_widget(Some(&font_button));
    
    group.add(&font_row);

    // Font size
    let font_size_row = adw::ActionRow::new();
    font_size_row.set_title(&t!("theme.font_size"));
    font_size_row.set_subtitle(&t!("theme.font_size_desc"));
    
    let icon3 = gtk::Image::from_icon_name("format-text-larger-symbolic");
    font_size_row.add_prefix(&icon3);
    
    let font_size_spin = gtk::SpinButton::with_range(8.0, 32.0, 1.0);
    font_size_spin.set_value(config.ui.font_size as f64);
    font_size_spin.set_valign(gtk::Align::Center);
    font_size_row.add_suffix(&font_size_spin);
    
    group.add(&font_size_row);

    page.add(&group);

    // Save button at the end
    let save_button = gtk::Button::with_label(&t!("general.save"));
    save_button.add_css_class("suggested-action");
    save_button.add_css_class("pill");
    save_button.set_halign(gtk::Align::Center);
    save_button.set_size_request(300, -1);
    
    let theme_row_clone = theme_row.clone();
    let lang_row_clone = lang_row.clone();
    let font_button_clone = font_button.clone();
    let font_size_clone = font_size_spin.clone();
    
    save_button.connect_clicked(move |btn| {
        // Desabilitar botão e mostrar loading
        btn.set_sensitive(false);
        let original_label = btn.label().unwrap_or_default();
        
        // Criar spinner
        let spinner = gtk::Spinner::new();
        spinner.start();
        spinner.set_size_request(16, 16);
        
        let loading_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        loading_box.set_halign(gtk::Align::Center);
        loading_box.append(&spinner);
        loading_box.append(&gtk::Label::new(Some("Salvando...")));
        
        btn.set_child(Some(&loading_box));
        
        // Processar em background
        let btn_clone = btn.clone();
        let theme_row_for_save = theme_row_clone.clone();
        let lang_row_for_save = lang_row_clone.clone();
        let font_button_for_save = font_button_clone.clone();
        let font_size_for_save = font_size_clone.clone();
        
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            let mut config = Config::load().unwrap_or_default();
            
            config.ui.theme = match theme_row_for_save.selected() {
                0 => "system".to_string(),
                1 => "dark".to_string(),
                2 => "light".to_string(),
                _ => "system".to_string(),
            };
            
            let new_language = match lang_row_for_save.selected() {
                0 => "en".to_string(),
                1 => "pt".to_string(),
                _ => "en".to_string(),
            };
            
            config.ui.language = new_language.clone();
            
            // Obter família de fonte selecionada (compatível com FontButton)
            let font_family = font_button_for_save.font_desc()
                .and_then(|desc| desc.family().map(|f| f.to_string()))
                .unwrap_or_else(|| "Nunito".to_string());
            config.ui.font_family = font_family;
            
            config.ui.font_size = font_size_for_save.value() as u32;
            
            // Verificar se idioma OU fonte mudaram
            let old_config = Config::load().unwrap_or_default();
            let language_changed = old_config.ui.language != new_language;
            let font_changed = old_config.ui.font_family != config.ui.font_family;
            let needs_reload = language_changed || font_changed;
            
            if config.save().is_ok() {
                // Aplicar tema imediatamente
                let style_manager = adw::StyleManager::default();
                match config.ui.theme.as_str() {
                    "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
                    "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
                    "system" | _ => style_manager.set_color_scheme(adw::ColorScheme::Default),
                }
                
                // Aplicar idioma imediatamente
                set_language(&new_language);
                
                eprintln!("✅ {}", t!("messages.saved"));
                
                // Reiniciar daemon
                let _restart_result = std::process::Command::new("systemctl")
                    .args(&["--user", "restart", "clippit"])
                    .output();
                
                // Mostrar sucesso
                btn_clone.set_child(Some(&gtk::Label::new(Some("✓ Salvo!"))));
                btn_clone.add_css_class("success");
                
                // Se idioma OU fonte mudaram, recarregar o dashboard
                if needs_reload {
                    if language_changed {
                        eprintln!("🔄 Idioma alterado, recarregando dashboard...");
                    }
                    if font_changed {
                        eprintln!("🔄 Fonte alterada, recarregando dashboard...");
                    }
                    
                    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(800), move || {
                        // Iniciar nova instância do dashboard
                        std::process::Command::new("clippit-dashboard")
                            .spawn()
                            .ok();
                        
                        // Fechar janela atual após pequeno delay
                        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(200), move || {
                            if let Some(window) = btn_clone.root().and_downcast::<gtk::Window>() {
                                window.close();
                            }
                        });
                    });
                } else {
                    // Apenas tema mudou, restaurar botão
                    let btn_final = btn_clone.clone();
                    let label_final = original_label.clone();
                    gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                        btn_final.set_child(None::<&gtk::Widget>);
                        btn_final.set_label(&label_final);
                        btn_final.remove_css_class("success");
                        btn_final.set_sensitive(true);
                    });
                }
            } else {
                // Erro ao salvar
                btn_clone.set_label("✗ Erro!");
                btn_clone.set_sensitive(true);
            }
        });
    });
    
    let button_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);
    button_box.set_margin_top(24);
    button_box.set_margin_bottom(12);
    button_box.append(&save_button);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.append(&page);
    container.append(&button_box);
    container.set_margin_start(12);
    container.set_margin_end(12);
    container.set_margin_top(12);
    container.set_margin_bottom(12);
    
    scrolled.set_child(Some(&container));

    scrolled.upcast()
}
