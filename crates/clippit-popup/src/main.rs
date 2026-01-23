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
use views::{create_main_window, populate_history_list, setup_search_filter, setup_infinite_scroll};
use controllers::{setup_keyboard_navigation, setup_row_activation};
use models::{new_entry_map, new_search_content_map};

const APP_ID: &str = "com.clippit.Clippit";

// Initialize i18n pointing to the same locales as clippit-core
rust_i18n::i18n!("../clippit-core/locales", fallback = "en");

fn main() -> Result<()> {
    // Single instance check
    handle_lock_file()?;
    
    // Setup signal handler for SIGTERM (triggered when hotkey is pressed again)
    // This must be done before GTK init to avoid thread safety issues
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let should_quit = Arc::new(AtomicBool::new(false));
    let should_quit_clone = should_quit.clone();
    
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGTERM, move || {
            eprintln!("🛑 Received SIGTERM - marking for quit...");
            should_quit_clone.store(true, Ordering::Relaxed);
        }).ok();
    }
    
    // Initialize GTK
    adw::init().expect("Failed to initialize libadwaita");

    // Create application
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    // Periodically check if we should quit
    let app_for_quit = app.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if should_quit.load(Ordering::Relaxed) {
            eprintln!("🛑 Quitting application due to SIGTERM...");
            app_for_quit.quit();
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });

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
                // Verificar se processo está rodando
                let check = std::process::Command::new("ps")
                    .args(&["-p", &pid.to_string(), "-o", "stat="])
                    .output();
                
                if let Ok(output) = check {
                    let stat = String::from_utf8_lossy(&output.stdout);
                    
                    // Se processo está rodando (não é zombie)
                    if !stat.trim().is_empty() && !stat.trim().starts_with('Z') {
                        eprintln!("🔄 Popup já rodando (PID: {}) - fechando (toggle)", pid);
                        
                        // Enviar SIGTERM para fechar
                        let _ = std::process::Command::new("kill")
                            .args(&["-TERM", &pid.to_string()])
                            .output();
                        
                        // Aguardar processo fechar (até 500ms)
                        for i in 0..10 {
                            std::thread::sleep(std::time::Duration::from_millis(50));
                            
                            let check = std::process::Command::new("ps")
                                .args(&["-p", &pid.to_string()])
                                .output();
                            
                            if let Ok(output) = check {
                                if !output.status.success() {
                                    eprintln!("✅ Popup fechado após {}ms", (i + 1) * 50);
                                    break;
                                }
                            }
                        }
                        
                        // Limpar lock file
                        std::fs::remove_file(lock_file).ok();
                        eprintln!("✅ Toggle completo - saindo");
                        std::process::exit(0);
                    } else {
                        eprintln!("💀 Limpando lock file de processo zombie/inexistente (PID: {})", pid);
                        std::fs::remove_file(lock_file).ok();
                    }
                } else {
                    eprintln!("⚠️  Removendo lock file - não foi possível verificar processo");
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
    // Load configuration and apply settings
    let config = Config::load().unwrap_or_default();
    set_language(&config.ui.language);
    apply_theme(&config);
    load_custom_css();
    
    // Create main window structure (no toast overlay - using system notifications)
    let (window, list_box, scrolled, search_entry) = create_main_window(app);
    
    // Create data structures
    let entry_map = new_entry_map();
    let search_map = new_search_content_map();
    
    // Add minimal skeleton loaders (3 instead of 5)
    add_skeleton_loaders(&list_box, 3);
    
    // 🚀 SHOW WINDOW IMMEDIATELY (before loading data)
    window.present_with_time(gtk::gdk::CURRENT_TIME);
    window.set_focus_visible(true);
    
    // Setup keyboard navigation first (so ESC works immediately)
    setup_keyboard_navigation(&window, app, &list_box, &scrolled, &entry_map, &search_entry);
    
    // Load data asynchronously using idle_add (more efficient than timeout)
    let list_box_clone = list_box.clone();
    let scrolled_clone = scrolled.clone();
    let window_clone = window.clone();
    let app_clone = app.clone();
    let entry_map_clone = entry_map.clone();
    let search_map_clone = search_map.clone();
    let search_entry_clone = search_entry.clone();
    
    gtk::glib::idle_add_local_once(move || {
        // Remove skeleton loaders
        remove_skeleton_loaders(&list_box_clone);
        
        // Populate history list with entries
        populate_history_list(&list_box_clone, &window_clone, &app_clone, &entry_map_clone, &search_map_clone);
        
        // ⚠️ IMPORTANTE: Garantir que primeiro item está selecionado E focado
        // Usar timeout para garantir que GTK processou a lista completamente
        let list_box_for_focus = list_box_clone.clone();
        gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(50), move || {
            // Selecionar primeiro item explicitamente
            if let Some(first_row) = list_box_for_focus.row_at_index(0) {
                list_box_for_focus.select_row(Some(&first_row));
                eprintln!("✅ Primeiro item selecionado explicitamente");
                
                // Dar foco ao primeiro item (não ao list_box)
                first_row.grab_focus();
                eprintln!("✅ Foco dado ao primeiro item");
            } else {
                eprintln!("⚠️  Nenhum item encontrado para focar");
            }
        });
        
        // Setup search filtering (with ability to reload list)
        setup_search_filter(&list_box_clone, &search_entry_clone, &search_map_clone, &window_clone, &app_clone, &entry_map_clone);
        
        // Setup row activation (click)
        setup_row_activation(&list_box_clone, &entry_map_clone, &window_clone, &app_clone);
        
        // Setup infinite scroll
        setup_infinite_scroll(&scrolled_clone, &list_box_clone, &window_clone, &app_clone, &entry_map_clone, &search_map_clone);
    });
}

fn add_skeleton_loaders(list_box: &gtk::ListBox, count: usize) {
    use gtk::prelude::*;
    
    // Add skeleton rows that match the real item design
    for i in 0..count {
        let row = gtk::ListBoxRow::new();
        row.set_widget_name(&format!("skeleton-{}", i));
        
        // Main horizontal box
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        hbox.set_margin_top(12);
        hbox.set_margin_bottom(12);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        
        // Skeleton thumbnail (matches real thumbnail: 48x48, rounded)
        let skeleton_thumb = gtk::Box::new(gtk::Orientation::Vertical, 0);
        skeleton_thumb.set_width_request(48);
        skeleton_thumb.set_height_request(48);
        skeleton_thumb.set_valign(gtk::Align::Center);
        skeleton_thumb.add_css_class("skeleton-thumb");
        skeleton_thumb.add_css_class("skeleton-pulse");
        
        // Vertical box for title + subtitle
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        vbox.set_hexpand(true);
        vbox.set_valign(gtk::Align::Center);
        
        // Skeleton title
        let skeleton_title = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        skeleton_title.set_height_request(14);
        skeleton_title.set_width_request(280);
        skeleton_title.set_halign(gtk::Align::Start);
        skeleton_title.add_css_class("skeleton-text");
        skeleton_title.add_css_class("skeleton-pulse");
        
        // Skeleton subtitle
        let skeleton_subtitle = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        skeleton_subtitle.set_height_request(10);
        skeleton_subtitle.set_width_request(180);
        skeleton_subtitle.set_halign(gtk::Align::Start);
        skeleton_subtitle.add_css_class("skeleton-text");
        skeleton_subtitle.add_css_class("skeleton-pulse");
        
        vbox.append(&skeleton_title);
        vbox.append(&skeleton_subtitle);
        
        hbox.append(&skeleton_thumb);
        hbox.append(&vbox);
        
        row.set_child(Some(&hbox));
        list_box.append(&row);
    }
}

fn remove_skeleton_loaders(list_box: &gtk::ListBox) {
    use gtk::prelude::*;
    
    // Remove all skeleton rows
    let mut row = list_box.first_child();
    while let Some(current) = row {
        let next = current.next_sibling();
        if let Some(list_row) = current.downcast_ref::<gtk::ListBoxRow>() {
            if list_row.widget_name().starts_with("skeleton-") {
                list_box.remove(list_row);
            }
        }
        row = next;
    }
}
