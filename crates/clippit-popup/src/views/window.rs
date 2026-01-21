use gtk::prelude::*;
use gtk::{ScrolledWindow, SearchEntry};
use libadwaita as adw;
use rust_i18n::t;

/// Creates the main popup window with list and search
/// 
/// Returns: (window, list_box, scrolled, search_entry)
pub fn create_main_window(
    app: &gtk::Application,
) -> (adw::ApplicationWindow, gtk::ListBox, ScrolledWindow, SearchEntry) {
    // Create search entry
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_placeholder_text(Some(&t!("popup.search_placeholder")));
    search_entry.set_hexpand(true);
    
    // Create list box for history items
    let list_box = gtk::ListBox::new();
    list_box.add_css_class("boxed-list");
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_can_focus(true);
    list_box.set_focus_on_click(false);
    list_box.set_activate_on_single_click(false);
    
    // Create scrolled window
    let scrolled = ScrolledWindow::new();
    scrolled.set_child(Some(&list_box));
    scrolled.set_vexpand(true);
    scrolled.set_margin_start(12);
    scrolled.set_margin_end(12);
    scrolled.set_margin_bottom(12);
    
    // Create main vertical box
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    
    // Create header with search - will be populated later
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    header_box.set_margin_top(12);
    header_box.set_margin_start(12);
    header_box.set_margin_end(12);
    header_box.set_margin_bottom(12); // ✅ Padding igual ao topo
    header_box.append(&search_entry);
    
    main_box.append(&header_box);
    main_box.append(&scrolled);
    
    // Create main window
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(&t!("popup.title").to_string())
        .default_width(500)
        .default_height(400)
        .content(&main_box)
        .build();
    
    // Auto-close on focus loss with intelligent delay
    setup_auto_close(&window);
    
    eprintln!("🔵 Window: adw::ApplicationWindow, 500x400 (auto-close inteligente 500ms)");
    
    (window, list_box, scrolled, search_entry)
}

fn setup_auto_close(window: &adw::ApplicationWindow) {
    let window_for_focus = window.clone();
    window.connect_is_active_notify(move |win| {
        if !win.is_active() {
            eprintln!("🔴 Popup perdeu o foco - agendando fechamento inteligente...");
            let window_to_close = window_for_focus.clone();
            
            // Delay de 500ms para evitar fechamento acidental
            gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(500), move || {
                // Verifica novamente se ainda está inativo
                if !window_to_close.is_active() {
                    eprintln!("🔴 Confirmado: popup ainda inativo após 500ms - fechando");
                    window_to_close.close();
                } else {
                    eprintln!("✅ Popup voltou ao foco - cancelando fechamento");
                }
            });
        }
    });
}
