use adw::prelude::*;
use clippit_core::Config;
use gtk::prelude::*;
use libadwaita as adw;
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

    // Auto-save on toggle
    ignore_switch.connect_state_set(|_, state| {
        if let Ok(mut cfg) = Config::load() {
            cfg.privacy.ignore_sensitive_apps = state;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Ignorar apps sensíveis atualizado: {}", state);
            }
        }
        gtk::glib::Propagation::Proceed
    });

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

    // Edit config button group
    let button_group = adw::PreferencesGroup::new();
    button_group.set_margin_top(12);

    let edit_row = adw::ActionRow::new();
    edit_row.set_title(&t!("privacy.edit_list"));
    edit_row.set_subtitle("Editar manualmente o arquivo de configuração");

    let icon = gtk::Image::from_icon_name("document-edit-symbolic");
    edit_row.add_prefix(&icon);

    let edit_button = gtk::Button::from_icon_name("go-next-symbolic");
    edit_button.set_valign(gtk::Align::Center);
    edit_button.add_css_class("flat");

    edit_button.connect_clicked(|_| {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("clippit")
            .join("config.toml");

        let _ = std::process::Command::new("xdg-open")
            .arg(config_path)
            .spawn();
    });

    edit_row.add_suffix(&edit_button);
    edit_row.set_activatable(true);
    edit_row.connect_activated(move |_| {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("clippit")
            .join("config.toml");

        let _ = std::process::Command::new("xdg-open")
            .arg(config_path)
            .spawn();
    });

    button_group.add(&edit_row);
    page.add(&button_group);

    // No need for save button - auto-save enabled
    page.set_margin_start(12);
    page.set_margin_end(12);
    page.set_margin_top(12);
    page.set_margin_bottom(12);

    scrolled.set_child(Some(&page));

    scrolled.upcast()
}
