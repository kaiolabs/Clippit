use clippit_core::Config;
use gtk::prelude::*;
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

use crate::controllers::copy_to_clipboard;
use crate::models::EntryMap;

/// Sets up keyboard navigation (arrows, ESC, Enter, and toggle hotkey)
pub fn setup_keyboard_navigation(
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    list_box: &gtk::ListBox,
    scrolled: &gtk::ScrolledWindow,
    entry_map: &EntryMap,
    search_entry: &gtk::SearchEntry,
) {
    let window_nav = window.clone();
    let app_nav = app.clone();
    let entry_map_for_key = entry_map.clone();
    let scrolled_for_key = scrolled.clone();
    let list_box_for_key = list_box.clone();
    let search_entry_for_key = search_entry.clone();

    // Load config to get the hotkey
    let config = Config::load().unwrap_or_default();
    let hotkey_str = format!(
        "{}+{}",
        config.hotkeys.show_history_modifier, config.hotkeys.show_history_key
    );
    eprintln!(
        "🔵 Popup will also listen for configured hotkey: {}",
        hotkey_str
    );

    let hotkey_str_for_closure = hotkey_str.clone();

    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Bubble); // Processa DEPOIS dos widgets filhos

    // Get focus search hotkey from config
    let focus_search_str = format!(
        "{}+{}",
        config.search.focus_search_modifier, config.search.focus_search_key
    );
    eprintln!("🔵 Focus search hotkey configured: {}", focus_search_str);
    let focus_search_str_for_closure = focus_search_str.clone();

    key_controller.connect_key_pressed(move |_, key, _, modifiers| {
        eprintln!("🔑 Key pressed: {:?} with modifiers: {:?}", key, modifiers);

        // Check if this is the configured hotkey (for toggle)
        if is_configured_hotkey(key, modifiers, &hotkey_str_for_closure) {
            eprintln!("🔵 Configured hotkey pressed while popup open - closing (toggle)");
            window_nav.close();
            return gtk::glib::Propagation::Stop;
        }

        // Check if this is the focus search hotkey
        if is_configured_hotkey(key, modifiers, &focus_search_str_for_closure) {
            eprintln!("🔵 Focus search hotkey pressed - setting focus to search entry");
            search_entry_for_key.grab_focus();
            return gtk::glib::Propagation::Stop;
        }

        match key {
            // Esc com imagem ampliada: só fecha a imagem, não o popup
            gtk::gdk::Key::Escape if crate::views::lightbox::hide() => gtk::glib::Propagation::Stop,
            gtk::gdk::Key::Escape => {
                eprintln!("🚨 [CLOSE TRIGGER] ESC pressed - closing popup");
                window_nav.close();
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                // Se search_entry tem foco, não processa Enter (deixa o campo processar)
                if search_entry_for_key.has_focus() {
                    eprintln!("🔑 Enter pressed but search_entry has focus - ignoring");
                    return gtk::glib::Propagation::Proceed;
                }

                eprintln!("🔑 Enter key detected - calling handle_enter_key");
                handle_enter_key(&list_box_for_key, &entry_map_for_key, &window_nav, &app_nav);
                gtk::glib::Propagation::Stop
            }
            gtk::gdk::Key::Up => {
                // Só navega na lista se o search_entry não tiver foco
                if !search_entry_for_key.has_focus() {
                    handle_up_key(&list_box_for_key, &scrolled_for_key);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed // Deixa o popover processar
                }
            }
            gtk::gdk::Key::Down => {
                // Só navega na lista se o search_entry não tiver foco
                if !search_entry_for_key.has_focus() {
                    handle_down_key(&list_box_for_key, &scrolled_for_key);
                    gtk::glib::Propagation::Stop
                } else {
                    gtk::glib::Propagation::Proceed // Deixa o popover processar
                }
            }
            // Tab segue para o campo de busca ou para o popover, conforme o foco.
            gtk::gdk::Key::Tab => gtk::glib::Propagation::Proceed,
            _ => gtk::glib::Propagation::Proceed,
        }
    });

    window.add_controller(key_controller);

    // Fase de CAPTURA: roda antes do widget com foco. Necessário porque o campo de
    // busca (GtkSearchEntry) consome o Esc quando tem foco, e aí o controlador acima
    // (fase Bubble) nunca recebia a tecla — o Esc não fechava nada.
    let capture = gtk::EventControllerKey::new();
    capture.set_propagation_phase(gtk::PropagationPhase::Capture);
    let window_cap = window.clone();
    let list_cap = list_box.clone();
    let entry_map_cap = entry_map.clone();
    let search_cap = search_entry.clone();
    capture.connect_key_pressed(move |_, key, _, _| {
        use crate::views::lightbox;
        if lightbox::is_visible() {
            // Imagem ampliada: Esc/Espaço fecham só a imagem; + - 0 1 Enter tratados lá
            if matches!(key, gtk::gdk::Key::Escape | gtk::gdk::Key::space) {
                lightbox::hide();
                return gtk::glib::Propagation::Stop;
            }
            if lightbox::handle_key(key) {
                return gtk::glib::Propagation::Stop;
            }
            return gtk::glib::Propagation::Proceed;
        }
        // Espaço (fora da busca): amplia a imagem selecionada. Em item de texto não
        // faz nada — sem isso o GTK "ativava" a linha e copiava/fechava sem querer.
        if key == gtk::gdk::Key::space && !search_cap.has_focus() {
            if let Some(row) = list_cap.selected_row() {
                if let Some(&id) = entry_map_cap.borrow().get(&row.index()) {
                    lightbox::show_entry(id);
                }
            }
            return gtk::glib::Propagation::Stop;
        }
        if key == gtk::gdk::Key::Escape {
            eprintln!("🚨 ESC - escondendo popup");
            window_cap.close();
            return gtk::glib::Propagation::Stop;
        }
        gtk::glib::Propagation::Proceed
    });
    window.add_controller(capture);

    eprintln!(
        "🔵 Keyboard navigation setup: ↑↓ to navigate, Enter to copy, ESC or {} to close",
        hotkey_str
    );
}

/// Check if the pressed key matches the configured hotkey
fn is_configured_hotkey(
    key: gtk::gdk::Key,
    modifiers: gtk::gdk::ModifierType,
    hotkey_str: &str,
) -> bool {
    // Parse hotkey string (e.g. "ctrl+kp_1" or "super+v")
    let parts: Vec<String> = hotkey_str
        .split('+')
        .map(|s| s.trim().to_lowercase())
        .collect();

    let mut required_ctrl = false;
    let mut required_alt = false;
    let mut required_shift = false;
    let mut required_super = false;
    let mut required_key = gtk::gdk::Key::VoidSymbol;

    for part in &parts {
        match part.as_ref() {
            "ctrl" | "control" => required_ctrl = true,
            "alt" => required_alt = true,
            "shift" => required_shift = true,
            "super" | "meta" | "win" => required_super = true,
            "kp_1" | "numpad1" => required_key = gtk::gdk::Key::KP_1,
            "kp_2" | "numpad2" => required_key = gtk::gdk::Key::KP_2,
            "kp_3" | "numpad3" => required_key = gtk::gdk::Key::KP_3,
            "kp_4" | "numpad4" => required_key = gtk::gdk::Key::KP_4,
            "kp_5" | "numpad5" => required_key = gtk::gdk::Key::KP_5,
            "kp_6" | "numpad6" => required_key = gtk::gdk::Key::KP_6,
            "kp_7" | "numpad7" => required_key = gtk::gdk::Key::KP_7,
            "kp_8" | "numpad8" => required_key = gtk::gdk::Key::KP_8,
            "kp_9" | "numpad9" => required_key = gtk::gdk::Key::KP_9,
            "kp_0" | "numpad0" => required_key = gtk::gdk::Key::KP_0,
            "v" => required_key = gtk::gdk::Key::v,
            "c" => required_key = gtk::gdk::Key::c,
            "p" => required_key = gtk::gdk::Key::p,
            "f" => required_key = gtk::gdk::Key::f,
            "s" => required_key = gtk::gdk::Key::s,
            _ => {}
        }
    }

    // Check if modifiers match
    let has_ctrl = modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK);
    let has_alt = modifiers.contains(gtk::gdk::ModifierType::ALT_MASK);
    let has_shift = modifiers.contains(gtk::gdk::ModifierType::SHIFT_MASK);
    let has_super = modifiers.contains(gtk::gdk::ModifierType::SUPER_MASK);

    // Match key and modifiers
    key == required_key
        && has_ctrl == required_ctrl
        && has_alt == required_alt
        && has_shift == required_shift
        && has_super == required_super
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
            eprintln!("🔵 Copying entry ID: {}", entry_id);

            // Copia antes de esconder (Wayland exige janela com foco), depois esconde.
            // O popup fica residente: close() só esconde (ver main.rs).
            copy_to_clipboard(entry_id);
            window_for_row.close();
            let _ = &app_for_row;
        }
    });
}

fn handle_enter_key(
    list_box: &gtk::ListBox,
    entry_map: &Rc<RefCell<std::collections::HashMap<i32, i64>>>,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
) {
    eprintln!("🔑 handle_enter_key called");

    if let Some(selected) = list_box.selected_row() {
        let row_index = selected.index();
        eprintln!("🔑 Selected row index: {}", row_index);

        if let Some(&entry_id) = entry_map.borrow().get(&row_index) {
            eprintln!("🔵 Enter pressed - copying entry ID: {}", entry_id);

            // Copia antes de esconder (Wayland exige janela com foco). Popup é residente.
            copy_to_clipboard(entry_id);
            window.close();
            let _ = app;
        } else {
            eprintln!("⚠️  No entry_id found for row index {}", row_index);
        }
    } else {
        eprintln!("⚠️  No row selected");
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
