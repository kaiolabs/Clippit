use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use clippit_core::Config;
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
    group.set_title(&t!("privacy.title"));
    group.set_description(Some(&t!("privacy.description")));

    // Ignore sensitive apps switch
    let ignore_row = adw::ActionRow::new();
    ignore_row.set_title(&t!("privacy.ignore_sensitive"));
    ignore_row.set_subtitle(&t!("privacy.ignore_sensitive_desc"));
    
    let icon1 = gtk::Image::from_icon_name("security-high-symbolic");
    ignore_row.add_prefix(&icon1);
    
    let ignore_switch = gtk::Switch::new();
    ignore_switch.set_active(config.privacy.ignore_sensitive_apps);
    ignore_switch.set_valign(gtk::Align::Center);
    ignore_row.set_activatable_widget(Some(&ignore_switch));
    ignore_row.add_suffix(&ignore_switch);
    
    group.add(&ignore_row);
    page.add(&group);

    // Ignored apps info
    let ignored_group = adw::PreferencesGroup::new();
    ignored_group.set_title(&t!("privacy.ignored_apps"));
    ignored_group.set_margin_top(12);

    if config.privacy.ignored_apps.is_empty() {
        let empty_row = adw::ActionRow::new();
        empty_row.set_title(&t!("privacy.no_apps"));
        empty_row.set_subtitle(&t!("privacy.no_apps_desc"));
        
        let icon = gtk::Image::from_icon_name("dialog-information-symbolic");
        empty_row.add_prefix(&icon);
        
        ignored_group.add(&empty_row);
    } else {
        for app in &config.privacy.ignored_apps {
            let app_row = adw::ActionRow::new();
            app_row.set_title(app);
            
            let icon = gtk::Image::from_icon_name("applications-system-symbolic");
            app_row.add_prefix(&icon);
            
            ignored_group.add(&app_row);
        }
    }
    
    page.add(&ignored_group);

    // Buttons at the end
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    button_box.set_halign(gtk::Align::Center);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);
    button_box.set_margin_top(24);
    button_box.set_margin_bottom(12);
    
    // Edit config button
    let edit_button = gtk::Button::with_label(&t!("privacy.edit_list"));
    
    edit_button.connect_clicked(|_| {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("clippit")
            .join("config.toml");
        
        let _ = std::process::Command::new("xdg-open")
            .arg(config_path)
            .spawn();
    });
    
    // Save button
    let save_button = gtk::Button::with_label(&t!("privacy.save"));
    save_button.add_css_class("suggested-action");
    
    let ignore_row_clone = ignore_switch.clone();
    
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
        let ignore_row_for_save = ignore_row_clone.clone();
        
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            let mut config = Config::load().unwrap_or_default();
            config.privacy.ignore_sensitive_apps = ignore_row_for_save.is_active();
            
            if config.save().is_ok() {
                eprintln!("✅ Configurações de privacidade salvas!");
                
                // Reiniciar daemon para aplicar alterações de captura de imagem
                std::process::Command::new("systemctl")
                    .args(&["--user", "restart", "clippit"])
                    .spawn()
                    .ok();
                eprintln!("🔄 Daemon reiniciado para aplicar mudanças de captura de imagem");
                
                // Mostrar sucesso
                btn_clone.set_child(Some(&gtk::Label::new(Some("✓ Salvo!"))));
                btn_clone.add_css_class("success");
                
                // Restaurar botão após 2 segundos
                let btn_final = btn_clone.clone();
                let label_final = original_label.clone();
                gtk::glib::timeout_add_local_once(std::time::Duration::from_secs(2), move || {
                    btn_final.set_child(None::<&gtk::Widget>);
                    btn_final.set_label(&label_final);
                    btn_final.remove_css_class("success");
                    btn_final.set_sensitive(true);
                });
            } else {
                // Erro ao salvar
                btn_clone.set_label("✗ Erro!");
                btn_clone.set_sensitive(true);
            }
        });
    });
    
    button_box.append(&edit_button);
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
