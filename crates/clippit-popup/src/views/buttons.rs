use adw::prelude::*;
use gtk::prelude::*;
use libadwaita as adw;
use rust_i18n::t;

use crate::controllers::copy_to_clipboard;

/// Adds a delete button to a row that removes the entry from DB and UI
pub fn add_delete_button(row: &adw::ActionRow, entry_id: i64, list_box: &gtk::ListBox) {
    let delete_button = gtk::Button::from_icon_name("user-trash-symbolic");
    delete_button.set_valign(gtk::Align::Center);
    delete_button.add_css_class("flat");
    delete_button.add_css_class("circular");
    delete_button.set_tooltip_text(Some(&t!("popup.delete_item_tooltip")));

    let delete_entry_id = entry_id;
    let list_box_for_delete = list_box.clone();

    delete_button.connect_clicked(move |btn| {
        eprintln!(
            "🗑️🗑️🗑️ DELETE BUTTON CLICKED for entry ID: {}",
            delete_entry_id
        );

        match clippit_core::HistoryManager::new(get_db_path(), 100) {
            Ok(history) => match history.delete_by_id(delete_entry_id) {
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
            },
            Err(e) => {
                eprintln!("❌ Failed to create HistoryManager: {}", e);
            }
        }
    });

    row.add_suffix(&delete_button);
}

/// Adds a copy button to a row (Wayland-native with system notification)
pub fn add_copy_button(
    row: &adw::ActionRow,
    entry_id: i64,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
) {
    let copy_button = gtk::Button::from_icon_name("edit-copy-symbolic");
    copy_button.set_valign(gtk::Align::Center);
    copy_button.add_css_class("flat");
    copy_button.add_css_class("circular");
    copy_button.set_tooltip_text(Some("Copiar para clipboard"));

    let button_entry_id = entry_id;
    let window_clone = window.clone();
    let app_clone = app.clone();

    copy_button.connect_clicked(move |_| {
        eprintln!("🔵 Copy button clicked for entry ID: {}", button_entry_id);

        // Copy to clipboard (shows system notification and waits for it to be sent)
        copy_to_clipboard(button_entry_id);

        // Close immediately - notification was already sent
        eprintln!("🔵 Closing window (notification sent)...");
        window_clone.close();
        app_clone.quit();
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
