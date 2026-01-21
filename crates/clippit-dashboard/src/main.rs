mod ui;

use gtk::prelude::*;
use gtk::Application;
use libadwaita as adw;
use clippit_core::{Config, set_language};

// Initialize i18n pointing to the same locales as clippit-core
rust_i18n::i18n!("../clippit-core/locales", fallback = "en");
use rust_i18n::t;

const APP_ID: &str = "com.clippit.Dashboard";

fn main() -> anyhow::Result<()> {
    // Initialize GTK
    adw::init().expect("Failed to initialize libadwaita");

    // Create application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();

    Ok(())
}

fn apply_theme(config: &Config) {
    let style_manager = adw::StyleManager::default();
    
    match config.ui.theme.as_str() {
        "dark" => {
            style_manager.set_color_scheme(adw::ColorScheme::ForceDark);
            eprintln!("✅ Tema forçado: Dark");
        },
        "light" => {
            style_manager.set_color_scheme(adw::ColorScheme::ForceLight);
            eprintln!("✅ Tema forçado: Light");
        },
        "system" | _ => {
            style_manager.set_color_scheme(adw::ColorScheme::Default);
            eprintln!("✅ Tema: Sistema (seguindo SO)");
        },
    }
}

fn build_ui(app: &Application) {
    // Carregar configuração e aplicar idioma e tema ANTES de criar UI
    let config = Config::load().unwrap_or_default();
    set_language(&config.ui.language);
    apply_theme(&config);
    
    // Create main content with sidebar
    let content = ui::create_content();
    
    // Create main window with libadwaita
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(&t!("dashboard.title").to_string())
        .default_width(750)
        .default_height(580)
        .content(&content)
        .build();

    // Ensure window can be closed properly
    window.set_deletable(true);
    
    window.present();
}
