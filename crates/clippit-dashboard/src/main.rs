mod ui;

use clippit_core::{set_language, Config};
use gtk::prelude::*;
use gtk::Application;
use libadwaita as adw;

// Initialize i18n pointing to the same locales as clippit-core
rust_i18n::i18n!("../clippit-core/locales", fallback = "en");
use rust_i18n::t;

// ID próprio: o popup (que fica sempre aberto) usa com.clippit.Clippit. Com o
// mesmo ID, abrir o painel só reativava o popup e o painel fechava na hora.
const APP_ID: &str = "com.clippit.Dashboard";

fn main() -> anyhow::Result<()> {
    // Initialize GTK
    adw::init().expect("Failed to initialize libadwaita");

    // Create application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);
    app.run();

    Ok(())
}

fn build_ui(app: &Application) {
    // Carregar configuração e aplicar idioma e tema ANTES de criar UI
    let config = Config::load().unwrap_or_default();
    set_language(&config.ui.language);
    // Sempre o visual escuro do Clippit (o mesmo do lançador).
    adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
    ui::style::install();

    // Create main content with sidebar
    let content = ui::create_content();

    // Create main window with libadwaita
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(t!("dashboard.title").to_string())
        .default_width(1120)
        .default_height(800)
        .content(&content)
        .build();

    // Ensure window can be closed properly
    window.set_deletable(true);
    window.add_css_class("clippit-dashboard");

    // 🔑 FECHAR COMPLETAMENTE ao invés de minimizar
    let app_clone = app.clone();
    window.connect_close_request(move |_| {
        eprintln!("🚪 Dashboard fechando completamente...");
        app_clone.quit();
        gtk::glib::Propagation::Proceed
    });

    window.present();
}
