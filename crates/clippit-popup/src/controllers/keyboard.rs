use gtk::prelude::*;
use libadwaita as adw;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::utils::get_focused_window_id;
use crate::controllers::copy_to_clipboard_and_paste_with_target;
use crate::models::EntryMap;

/// Sets up keyboard navigation (arrows, ESC, Enter)
pub fn setup_keyboard_navigation(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    list_box: &gtk::ListBox,
    scrolled: &gtk::ScrolledWindow,
    entry_map: &EntryMap,
) {
    let window_nav = window.clone();
    let app_nav = app.clone();
    let entry_map_for_key = entry_map.clone();
    let scrolled_for_key = scrolled.clone();
    let list_box_for_key = list_box.clone();
    
    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    
    key_controller.connect_key_pressed(move |_, key, _, _| {
        match key {
            gtk::gdk::Key::Escape => {
                eprintln!("🔵 ESC pressed - closing popup");
                window_nav.close();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                handle_enter_key(&list_box_for_key, &entry_map_for_key, &window_nav, &app_nav);
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Up => {
                handle_up_key(&list_box_for_key, &scrolled_for_key);
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Down => {
                handle_down_key(&list_box_for_key, &scrolled_for_key);
                gtk::glib::Propagation::Stop
            }
            _ => gtk::glib::Propagation::Proceed
        }
    });
    
    window.add_controller(key_controller);
    
    eprintln!("🔵 Keyboard navigation setup: ↑↓ to navigate, Enter to paste, ESC to close");
}

/// Sets up row activation handler (click on row)
pub fn setup_row_activation(
    list_box: &gtk::ListBox,
    entry_map: &EntryMap,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
) {
    let window_for_row = window.clone();
    let app_for_row = app.clone();
    let entry_map_for_row = entry_map.clone();
    
    list_box.connect_row_activated(move |_, row| {
        let row_index = row.index();
        eprintln!("🔵 Row activated (clicked): index {}", row_index);
        
        if let Some(&entry_id) = entry_map_for_row.borrow().get(&row_index) {
            eprintln!("🔵 Activating entry ID: {}", entry_id);
            
            let original_window_id = get_focused_window_id();
            eprintln!("🔵 Captured original window ID: {}", original_window_id);
            
            spawn_paste_thread(entry_id, original_window_id, &window_for_row, &app_for_row, "row");
        }
    });
}

fn handle_enter_key(
    list_box: &gtk::ListBox,
    entry_map: &Rc<RefCell<std::collections::HashMap<i32, i64>>>,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
) {
    if let Some(selected) = list_box.selected_row() {
        let row_index = selected.index();
        
        if let Some(&entry_id) = entry_map.borrow().get(&row_index) {
            eprintln!("🔵 Enter pressed - pasting entry ID: {}", entry_id);
            
            let original_window_id = get_focused_window_id();
            eprintln!("🔵 Captured original window ID: {}", original_window_id);
            
            spawn_paste_thread(entry_id, original_window_id, window, app, "enter");
        }
    }
}

fn handle_up_key(list_box: &gtk::ListBox, scrolled: &gtk::ScrolledWindow) {
    if let Some(selected) = list_box.selected_row() {
        let index = selected.index();
        if index > 0 {
            if let Some(prev_row) = list_box.row_at_index(index - 1) {
                list_box.select_row(Some(&prev_row));
                eprintln!("🔵 ↑ Selected row {}", index - 1);
                
                // Scroll para garantir que o item INTEIRO fique visível
                let adjustment = scrolled.vadjustment();
                let row_y = prev_row.allocation().y() as f64;
                let current_scroll = adjustment.value();
                
                if row_y < current_scroll {
                    adjustment.set_value(row_y);
                    eprintln!("  Scroll para topo: {}", row_y);
                }
            }
        }
    }
}

fn handle_down_key(list_box: &gtk::ListBox, scrolled: &gtk::ScrolledWindow) {
    if let Some(selected) = list_box.selected_row() {
        let index = selected.index();
        if let Some(next_row) = list_box.row_at_index(index + 1) {
            list_box.select_row(Some(&next_row));
            eprintln!("🔵 ↓ Selected row {}", index + 1);
            
            // Scroll para garantir que o item INTEIRO fique visível
            let adjustment = scrolled.vadjustment();
            let row_y = next_row.allocation().y() as f64;
            let row_height = next_row.allocation().height() as f64;
            let page_size = adjustment.page_size();
            let current_scroll = adjustment.value();
            
            let row_bottom = row_y + row_height;
            let viewport_bottom = current_scroll + page_size;
            
            if row_bottom > viewport_bottom {
                adjustment.set_value(row_bottom - page_size);
                eprintln!("  Scroll para baixo: {}", row_bottom - page_size);
            } else if row_y < current_scroll {
                adjustment.set_value(row_y);
                eprintln!("  Scroll para topo: {}", row_y);
            }
        }
    }
}

fn spawn_paste_thread(
    entry_id: i64,
    window_id: u64,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    source: &str,
) {
    let paste_done = Arc::new(AtomicBool::new(false));
    let paste_done_check = paste_done.clone();
    
    // ✅ CRITICAL: Hold app to prevent early termination
    let _hold = app.hold();
    eprintln!("🔵 App held to prevent early termination");
    
    let source_name = format!("clippit-paste-{}", source);
    let source_str = source.to_string();
    std::thread::Builder::new()
        .name(source_name.clone())
        .spawn(move || {
            eprintln!("🔵 Paste thread ({}) started!", source_str);
            
            let result = std::panic::catch_unwind(|| {
                copy_to_clipboard_and_paste_with_target(entry_id, window_id);
            });
            
            if let Err(e) = result {
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                eprintln!("💥 PANIC in paste thread ({}): {:?}", source_str, e);
                eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            }
            
            eprintln!("🔵 Paste thread ({}) completed!", source_str);
            paste_done.store(true, Ordering::SeqCst);
        })
        .expect("Failed to spawn paste thread");
    
    window.close();
    eprintln!("🔵 Window closed ({}), monitoring paste completion...", source);
    
    let source_str2 = source.to_string();
    let app_for_quit = app.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
        if paste_done_check.load(Ordering::SeqCst) {
            eprintln!("🔵 Paste done ({}), quitting app", source_str2);
            // Note: hold() keeps app alive, quit() overrides it
            app_for_quit.quit();
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });
}
