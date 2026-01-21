use gtk::prelude::*;
use gtk::Application;
use libadwaita as adw;
use clippit_core::{Config, set_language};
use anyhow::Result;

mod utils;
mod controllers;
mod views;
mod models;

use utils::{apply_theme, load_custom_css};
use views::{create_main_window, populate_history_list, setup_search_filter};
use controllers::{setup_keyboard_navigation, setup_row_activation};
use models::{new_entry_map, new_search_content_map};

const APP_ID: &str = "com.clippit.Popup";

// Initialize i18n pointing to the same locales as clippit-core
rust_i18n::i18n!("../clippit-core/locales", fallback = "en");

fn main() -> Result<()> {
    // Single instance check
    handle_lock_file()?;
    
    // Initialize GTK
    adw::init().expect("Failed to initialize libadwaita");

    // Create application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    
    // Clean up lock file when app quits
    app.connect_shutdown(|_| {
        eprintln!("🔓 Removing lock file on shutdown...");
        std::fs::remove_file("/tmp/clippit-popup.lock").ok();
    });
    
    // Run application
    let exit_code = app.run_with_args(&Vec::<String>::new());
    
    // Extra safety: remove lock on exit
    eprintln!("🔓 Removing lock file on exit...");
    std::fs::remove_file("/tmp/clippit-popup.lock").ok();

    if exit_code.value() == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Application exited with code {}", exit_code.value()))
    }
}

fn handle_lock_file() -> Result<()> {
    let lock_file = std::path::Path::new("/tmp/clippit-popup.lock");
    
    if lock_file.exists() {
        if let Ok(content) = std::fs::read_to_string(lock_file) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                let check = std::process::Command::new("kill")
                    .args(&["-0", &pid.to_string()])
                    .output();
                
                if check.is_ok() && check.unwrap().status.success() {
                    eprintln!("Clippit popup already running (PID: {})", pid);
                    std::process::exit(0);
                } else {
                    eprintln!("Removing stale lock file (PID {} not running)", pid);
                    std::fs::remove_file(lock_file).ok();
                }
            }
        }
    }
    
    let my_pid = std::process::id();
    std::fs::write(lock_file, my_pid.to_string())?;
    eprintln!("🔒 Created lock file with PID: {}", my_pid);
    
    Ok(())
}

fn build_ui(app: &Application) {
    eprintln!("🔵 build_ui() called!");
    
    // Load configuration and apply settings
    let config = Config::load().unwrap_or_default();
    set_language(&config.ui.language);
    apply_theme(&config);
    load_custom_css();
    
    // Create main window structure
    let (window, list_box, scrolled, search_entry) = create_main_window(app);
    
    // Create data structures
    let entry_map = new_entry_map();
    let search_map = new_search_content_map();
    
    // Populate history list with entries
    populate_history_list(&list_box, &window, app, &entry_map, &search_map);
    
    // Setup search filtering
    setup_search_filter(&list_box, &search_entry, &search_map);
    
    // Setup keyboard navigation (arrows, Enter, ESC)
    setup_keyboard_navigation(&window, app, &list_box, &scrolled, &entry_map);
    
    // Setup row activation (click)
    setup_row_activation(&list_box, &entry_map, &window, app);
    
    eprintln!("🔵 Presenting window...");
    window.present();
    eprintln!("✅ Window presented - GTK will center automatically!");
}
