use adw::prelude::*;
use clippit_core::{set_language, Config};
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

    // Auto-save on theme change
    theme_row.connect_selected_notify(|row| {
        if let Ok(mut cfg) = Config::load() {
            cfg.ui.theme = match row.selected() {
                0 => "system".to_string(),
                1 => "dark".to_string(),
                2 => "light".to_string(),
                _ => "system".to_string(),
            };

            if cfg.save().is_ok() {
                // Apply theme immediately
                let style_manager = adw::StyleManager::default();
                match cfg.ui.theme.as_str() {
                    "dark" => style_manager.set_color_scheme(adw::ColorScheme::ForceDark),
                    "light" => style_manager.set_color_scheme(adw::ColorScheme::ForceLight),
                    "system" | _ => style_manager.set_color_scheme(adw::ColorScheme::Default),
                }
                eprintln!("✅ Tema atualizado: {}", cfg.ui.theme);
            }
        }
    });

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

    // Auto-save on language change (will reload dashboard)
    lang_row.connect_selected_notify(|row| {
        if let Ok(mut cfg) = Config::load() {
            let new_language = match row.selected() {
                0 => "en".to_string(),
                1 => "pt".to_string(),
                _ => "en".to_string(),
            };

            let old_language = cfg.ui.language.clone();
            cfg.ui.language = new_language.clone();

            if cfg.save().is_ok() && old_language != new_language {
                set_language(&new_language);
                eprintln!(
                    "✅ Idioma atualizado: {} (recarregando dashboard...)",
                    new_language
                );

                // Restart dashboard to apply language
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(500),
                    move || {
                        std::process::Command::new("clippit-dashboard").spawn().ok();
                        gtk::glib::timeout_add_local_once(
                            std::time::Duration::from_millis(200),
                            move || {
                                std::process::exit(0);
                            },
                        );
                    },
                );
            }
        }
    });

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

    // Auto-save on font change (will reload dashboard)
    font_button.connect_font_set(|btn| {
        if let Ok(mut cfg) = Config::load() {
            let font_family = btn
                .font_desc()
                .and_then(|desc| desc.family().map(|f| f.to_string()))
                .unwrap_or_else(|| "Nunito".to_string());

            let old_font = cfg.ui.font_family.clone();
            cfg.ui.font_family = font_family.clone();

            if cfg.save().is_ok() && old_font != font_family {
                eprintln!(
                    "✅ Fonte atualizada: {} (recarregando dashboard...)",
                    font_family
                );

                // Restart dashboard to apply font
                gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(500),
                    move || {
                        std::process::Command::new("clippit-dashboard").spawn().ok();
                        gtk::glib::timeout_add_local_once(
                            std::time::Duration::from_millis(200),
                            move || {
                                std::process::exit(0);
                            },
                        );
                    },
                );
            }
        }
    });

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

    // Auto-save on font size change
    font_size_spin.connect_value_changed(|spin| {
        if let Ok(mut cfg) = Config::load() {
            cfg.ui.font_size = spin.value() as u32;
            if let Err(e) = cfg.save() {
                eprintln!("❌ Erro ao salvar: {}", e);
            } else {
                eprintln!("✅ Tamanho de fonte atualizado: {}", spin.value());
            }
        }
    });

    font_size_row.add_suffix(&font_size_spin);
    group.add(&font_size_row);

    page.add(&group);

    // No need for save button - auto-save enabled
    // Note: Language and Font changes will automatically reload the dashboard
    page.set_margin_start(12);
    page.set_margin_end(12);
    page.set_margin_top(12);
    page.set_margin_bottom(12);

    scrolled.set_child(Some(&page));

    scrolled.upcast()
}
