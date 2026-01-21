use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use rust_i18n::t;

use crate::utils::get_focused_window_id;
use crate::controllers::copy_to_clipboard_and_paste_with_target;

/// Adds a delete button to a row that removes the entry from DB and UI
pub fn add_delete_button(row: &adw::ActionRow, entry_id: i64, list_box: &gtk::ListBox) {
    let delete_button = gtk::Button::from_icon_name("user-trash-symbolic");
    delete_button.set_valign(gtk::Align::Center);
    delete_button.add_css_class("flat");
    delete_button.set_tooltip_text(Some(&t!("popup.delete_item_tooltip")));
    
    let delete_entry_id = entry_id;
    let list_box_for_delete = list_box.clone();
    
    delete_button.connect_clicked(move |btn| {
        eprintln!("🗑️🗑️🗑️ DELETE BUTTON CLICKED for entry ID: {}", delete_entry_id);
        
        match clippit_core::HistoryManager::new(get_db_path(), 100) {
            Ok(history) => {
                match history.delete_by_id(delete_entry_id) {
                    Ok(_) => {
                        eprintln!("✅ Entry {} deleted from database", delete_entry_id);
                        
                        if let Some(row) = btn.ancestor(gtk::ListBoxRow::static_type()) {
                            if let Ok(list_box_row) = row.downcast::<gtk::ListBoxRow>() {
                                list_box_for_delete.remove(&list_box_row);
                                eprintln!("✅ Entry removed from UI");
                            } else {
                                eprintln!("❌ Failed to downcast to ListBoxRow");
                            }
                        } else {
                            eprintln!("❌ Could not find parent ListBoxRow");
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to delete entry {}: {}", delete_entry_id, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to create HistoryManager: {}", e);
            }
        }
    });
    
    row.add_suffix(&delete_button);
}

/// Adds a copy-and-paste button to a row
pub fn add_copy_button(
    row: &adw::ActionRow,
    entry_id: i64,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
) {
    let copy_button = gtk::Button::from_icon_name("edit-copy");
    copy_button.set_valign(gtk::Align::Center);
    copy_button.add_css_class("flat");
    copy_button.set_tooltip_text(Some("Copiar e colar"));
    
    let button_entry_id = entry_id;
    let window_clone = window.clone();
    let app_clone = app.clone();
    
    copy_button.connect_clicked(move |_| {
        eprintln!("🔵 Copy button clicked for entry ID: {}", button_entry_id);
        
        let original_window_id = get_focused_window_id();
        eprintln!("🔵 Captured original window ID: {}", original_window_id);
        
        // ✅ CRITICAL: Hold app to prevent early termination
        let _hold = app_clone.hold();
        eprintln!("🔵 App held to prevent early termination");
        
        let paste_done = Arc::new(AtomicBool::new(false));
        let paste_done_check = paste_done.clone();
        
        std::thread::Builder::new()
            .name("clippit-paste-button".to_string())
            .spawn(move || {
                let result = std::panic::catch_unwind(|| {
                    copy_to_clipboard_and_paste_with_target(button_entry_id, original_window_id);
                });
                
                if let Err(e) = result {
                    eprintln!("💥 PANIC in paste thread (button): {:?}", e);
                }
                
                eprintln!("🔵 Paste thread (button) completed!");
                paste_done.store(true, Ordering::SeqCst);
            })
            .expect("Failed to spawn paste thread");
        
        window_clone.close();
        eprintln!("🔵 Window closed (button), monitoring paste completion...");
        
        let app_for_quit = app_clone.clone();
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if paste_done_check.load(Ordering::SeqCst) {
                eprintln!("🔵 Paste done (button), quitting app");
                // Note: hold() keeps app alive, quit() overrides it
                app_for_quit.quit();
                gtk::glib::ControlFlow::Break
            } else {
                gtk::glib::ControlFlow::Continue
            }
        });
    });
    
    row.add_suffix(&copy_button);
}

fn get_db_path() -> std::path::PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    path.push("clippit");
    std::fs::create_dir_all(&path).ok();
    path.push("history.db");
    path
}
