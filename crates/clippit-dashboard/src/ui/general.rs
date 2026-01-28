use adw::prelude::*;
use clippit_core::{Config, HistoryManager};
use gtk::prelude::*;
use libadwaita as adw;
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

    // Auto-save on value change
    max_items_spin.connect_value_changed(|spin| {
        if let Ok(mut cfg) = Config::load() {
            cfg.general.max_history_items = spin.value() as usize;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Max items atualizado: {}", spin.value());
            }
        }
    });

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

    // Auto-save on value change
    poll_interval_spin.connect_value_changed(|spin| {
        if let Ok(mut cfg) = Config::load() {
            cfg.general.poll_interval_ms = spin.value() as u64;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Poll interval atualizado: {}", spin.value());
            }
        }
    });

    poll_interval_row.add_suffix(&poll_interval_spin);
    group.add(&poll_interval_row);

    // Show Notifications
    let notifications_row = adw::ActionRow::new();
    notifications_row.set_title("Mostrar Notificações");
    notifications_row.set_subtitle("Exibir notificações ao copiar itens");

    let icon3 = gtk::Image::from_icon_name("preferences-system-notifications-symbolic");
    notifications_row.add_prefix(&icon3);

    let notifications_switch = gtk::Switch::new();
    notifications_switch.set_active(config.ui.show_notifications);
    notifications_switch.set_valign(gtk::Align::Center);

    // Auto-save on toggle
    notifications_switch.connect_active_notify(|switch| {
        if let Ok(mut cfg) = Config::load() {
            cfg.ui.show_notifications = switch.is_active();
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!(
                    "✅ Notificações {}",
                    if switch.is_active() {
                        "habilitadas"
                    } else {
                        "desabilitadas"
                    }
                );
            }
        }
    });

    notifications_row.add_suffix(&notifications_switch);
    notifications_row.set_activatable_widget(Some(&notifications_switch));
    group.add(&notifications_row);

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
        let mut db_path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
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

    // Auto-save on toggle
    enable_image_switch.connect_state_set(|_, state| {
        if let Ok(mut cfg) = Config::load() {
            cfg.privacy.enable_image_capture = state;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Image capture atualizado: {}", state);
            }
        }
        gtk::glib::Propagation::Proceed
    });

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
        let value = scale.value() as u32;
        size_label_clone.set_text(&format!("{} MB", value));

        // Auto-save on slider change
        if let Ok(mut cfg) = Config::load() {
            cfg.privacy.max_image_size_mb = value;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Max image size atualizado: {} MB", value);
            }
        }
    });

    slider_box.append(&size_scale);
    slider_box.append(&size_label);

    image_size_row.add_suffix(&slider_box);
    image_group.add(&image_size_row);

    page.add(&image_group);

    // OCR Settings Group
    let ocr_group = adw::PreferencesGroup::new();
    ocr_group.set_title("OCR (Reconhecimento de Texto)");
    ocr_group.set_description(Some("Extrai automaticamente texto de imagens para busca"));
    ocr_group.set_margin_top(12);

    // Enable OCR switch
    let enable_ocr_row = adw::ActionRow::new();
    enable_ocr_row.set_title("Ativar OCR");
    enable_ocr_row.set_subtitle("Extrair texto de screenshots e imagens automaticamente");

    let ocr_icon = gtk::Image::from_icon_name("document-properties-symbolic");
    enable_ocr_row.add_prefix(&ocr_icon);

    let enable_ocr_switch = gtk::Switch::new();
    enable_ocr_switch.set_active(config.features.enable_ocr);
    enable_ocr_switch.set_valign(gtk::Align::Center);

    enable_ocr_switch.connect_active_notify(|switch| {
        if let Ok(mut cfg) = Config::load() {
            cfg.features.enable_ocr = switch.is_active();
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!(
                    "✅ OCR {}",
                    if switch.is_active() {
                        "habilitado"
                    } else {
                        "desabilitado"
                    }
                );
            }
        }
    });

    enable_ocr_row.add_suffix(&enable_ocr_switch);
    enable_ocr_row.set_activatable_widget(Some(&enable_ocr_switch));
    ocr_group.add(&enable_ocr_row);

    // OCR Languages row
    let ocr_lang_row = adw::ActionRow::new();
    ocr_lang_row.set_title("Idiomas OCR");
    ocr_lang_row.set_subtitle("Idiomas para reconhecimento de texto");

    let lang_icon = gtk::Image::from_icon_name("preferences-desktop-locale-symbolic");
    ocr_lang_row.add_prefix(&lang_icon);

    // Create dropdown for language selection
    let lang_dropdown = gtk::DropDown::from_strings(&[
        "por+eng (Português + Inglês)",
        "por (Apenas Português)",
        "eng (Apenas Inglês)",
    ]);

    // Set current selection based on config
    let current_lang_index = match config.ocr.languages.as_str() {
        "por+eng" => 0,
        "por" => 1,
        "eng" => 2,
        _ => 0,
    };
    lang_dropdown.set_selected(current_lang_index);
    lang_dropdown.set_valign(gtk::Align::Center);

    lang_dropdown.connect_selected_notify(|dropdown| {
        let languages = match dropdown.selected() {
            0 => "por+eng",
            1 => "por",
            2 => "eng",
            _ => "por+eng",
        };

        if let Ok(mut cfg) = Config::load() {
            cfg.ocr.languages = languages.to_string();
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Idiomas OCR atualizados: {}", languages);
            }
        }
    });

    ocr_lang_row.add_suffix(&lang_dropdown);
    ocr_group.add(&ocr_lang_row);

    page.add(&ocr_group);

    // No need for save button - auto-save enabled
    page.set_margin_start(12);
    page.set_margin_end(12);
    page.set_margin_top(12);
    page.set_margin_bottom(12);

    scrolled.set_child(Some(&page));

    scrolled.upcast()
}
