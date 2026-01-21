use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use clippit_core::{Config, HistoryManager};
use rust_i18n::t;

pub fn create_page() -> gtk::Widget {
    let config = Config::load().unwrap_or_default();

    // Create scrolled window for content
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scrolled.set_vexpand(true);

    // Create preference page
    let page = adw::PreferencesPage::new();
    
    // Create preference group
    let group = adw::PreferencesGroup::new();
    group.set_title(&t!("general.title"));
    group.set_description(Some(&t!("general.description")));

    // Max History Items
    let max_items_row = adw::ActionRow::new();
    max_items_row.set_title(&t!("general.max_items"));
    max_items_row.set_subtitle(&t!("general.max_items_desc"));
    
    let icon1 = gtk::Image::from_icon_name("folder-documents-symbolic");
    max_items_row.add_prefix(&icon1);
    
    let max_items_spin = gtk::SpinButton::with_range(10.0, 10000.0, 1.0);
    max_items_spin.set_value(config.general.max_history_items as f64);
    max_items_spin.set_valign(gtk::Align::Center);
    max_items_row.add_suffix(&max_items_spin);
    
    group.add(&max_items_row);

    // Poll Interval
    let poll_interval_row = adw::ActionRow::new();
    poll_interval_row.set_title(&t!("general.poll_interval"));
    poll_interval_row.set_subtitle(&t!("general.poll_interval_desc"));
    
    let icon2 = gtk::Image::from_icon_name("media-playlist-repeat-symbolic");
    poll_interval_row.add_prefix(&icon2);
    
    let poll_interval_spin = gtk::SpinButton::with_range(50.0, 5000.0, 10.0);
    poll_interval_spin.set_value(config.general.poll_interval_ms as f64);
    poll_interval_spin.set_valign(gtk::Align::Center);
    poll_interval_row.add_suffix(&poll_interval_spin);
    
    group.add(&poll_interval_row);

    page.add(&group);
    
    // Actions group - Limpar Histórico
    let actions_group = adw::PreferencesGroup::new();
    actions_group.set_title(&t!("privacy.actions_title"));
    actions_group.set_description(Some(&t!("privacy.actions_desc")));
    actions_group.set_margin_top(12);
    
    // Clear history row
    let clear_row = adw::ActionRow::new();
    clear_row.set_title(&t!("privacy.clear_all"));
    clear_row.set_subtitle(&t!("privacy.clear_all_desc"));
    
    let clear_icon = gtk::Image::from_icon_name("edit-clear-all-symbolic");
    clear_row.add_prefix(&clear_icon);
    
    // Clear button
    let clear_button = gtk::Button::with_label(&t!("privacy.clear_button"));
    clear_button.add_css_class("destructive-action");
    clear_button.set_valign(gtk::Align::Center);
    
    clear_button.connect_clicked(move |_| {
        eprintln!("🗑️ Limpando histórico completo...");
        
        // Get database path
        let mut db_path = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        db_path.push("clippit");
        std::fs::create_dir_all(&db_path).ok();
        db_path.push("history.db");
        
        // Clear history
        if let Ok(history) = HistoryManager::new(db_path, 100) {
            if let Err(e) = history.clear() {
                eprintln!("❌ Erro ao limpar histórico: {}", e);
            } else {
                eprintln!("✅ Histórico limpo com sucesso!");
            }
        }
    });
    
    clear_row.add_suffix(&clear_button);
    actions_group.add(&clear_row);
    page.add(&actions_group);
    
    // Image Settings Group
    let image_group = adw::PreferencesGroup::new();
    image_group.set_title(&t!("privacy.image_capture_title"));
    image_group.set_description(Some(&t!("privacy.image_capture_desc_group")));
    image_group.set_margin_top(12);
    
    // Enable image capture switch
    let enable_image_row = adw::ActionRow::new();
    enable_image_row.set_title(&t!("privacy.enable_image_capture"));
    enable_image_row.set_subtitle(&t!("privacy.enable_image_capture_desc"));
    
    let image_icon = gtk::Image::from_icon_name("image-x-generic-symbolic");
    enable_image_row.add_prefix(&image_icon);
    
    let enable_image_switch = gtk::Switch::new();
    enable_image_switch.set_active(config.privacy.enable_image_capture);
    enable_image_switch.set_valign(gtk::Align::Center);
    enable_image_row.set_activatable_widget(Some(&enable_image_switch));
    enable_image_row.add_suffix(&enable_image_switch);
    
    image_group.add(&enable_image_row);
    
    // Max image size slider
    let image_size_row = adw::ActionRow::new();
    image_size_row.set_title(&t!("privacy.max_image_size"));
    image_size_row.set_subtitle(&t!("privacy.max_image_size_desc"));
    
    let slider_icon = gtk::Image::from_icon_name("drive-harddisk-symbolic");
    image_size_row.add_prefix(&slider_icon);
    
    // Create horizontal box for slider and label
    let slider_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    
    let size_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 1.0, 20.0, 1.0);
    size_scale.set_value(config.privacy.max_image_size_mb as f64);
    size_scale.set_hexpand(true);
    size_scale.set_width_request(200);
    size_scale.set_draw_value(false);
    
    let size_label = gtk::Label::new(Some(&format!("{} MB", config.privacy.max_image_size_mb)));
    size_label.set_width_chars(6);
    
    let size_label_clone = size_label.clone();
    size_scale.connect_value_changed(move |scale| {
        size_label_clone.set_text(&format!("{} MB", scale.value() as u32));
    });
    
    slider_box.append(&size_scale);
    slider_box.append(&size_label);
    
    image_size_row.add_suffix(&slider_box);
    image_group.add(&image_size_row);
    
    page.add(&image_group);
    
    // Save button group at the end
    let button_group = adw::PreferencesGroup::new();
    button_group.set_margin_top(24);
    
    let save_button = gtk::Button::with_label(&t!("general.save"));
    save_button.add_css_class("suggested-action");
    save_button.add_css_class("pill");
    save_button.set_halign(gtk::Align::Center);
    save_button.set_size_request(300, -1);
    
    let max_items_clone = max_items_spin.clone();
    let poll_interval_clone = poll_interval_spin.clone();
    let size_scale_clone = size_scale.clone();
    let enable_image_clone = enable_image_switch.clone();
    
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
        let max_items_for_save = max_items_clone.clone();
        let poll_interval_for_save = poll_interval_clone.clone();
        let size_scale_for_save = size_scale_clone.clone();
        let enable_image_for_save = enable_image_clone.clone();
        
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(100), move || {
            let mut config = Config::load().unwrap_or_default();
            config.general.max_history_items = max_items_for_save.value() as usize;
            config.general.poll_interval_ms = poll_interval_for_save.value() as u64;
            config.privacy.max_image_size_mb = size_scale_for_save.value() as u32;
            config.privacy.enable_image_capture = enable_image_for_save.is_active();
            
            if config.save().is_ok() {
                eprintln!("✅ Configurações salvas!");
                eprintln!("🔄 Reiniciando daemon para aplicar mudanças...");
                
                let _restart_result = std::process::Command::new("systemctl")
                    .args(&["--user", "restart", "clippit"])
                    .output();
                
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
    
    // Create a box for the button
    let button_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    button_box.set_margin_start(12);
    button_box.set_margin_end(12);
    button_box.set_margin_top(12);
    button_box.set_margin_bottom(12);
    button_box.append(&save_button);
    
    let button_row = gtk::ListBoxRow::new();
    button_row.set_activatable(false);
    button_row.set_selectable(false);
    button_row.set_child(Some(&button_box));
    
    // We need to add the button after the page in a container
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
