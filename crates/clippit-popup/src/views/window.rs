use gtk::prelude::*;
use gtk::{ScrolledWindow, SearchEntry};
use libadwaita as adw;
use rust_i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates the main popup window with list and search
///
/// Returns: (window, list_box, scrolled, search_entry)
pub fn create_main_window(
    app: &gtk::Application,
) -> (
    adw::ApplicationWindow,
    gtk::ListBox,
    ScrolledWindow,
    SearchEntry,
) {
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
    list_box.set_activate_on_single_click(true); // 🔥 SINGLE CLICK para copiar!

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

    // Create main window (no toast overlay - using system notifications)
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(&t!("popup.title").to_string())
        .default_width(700)
        .default_height(550)
        .content(&main_box)
        .build();

    // Auto-close on focus loss with intelligent delay (passa search_entry para verificar foco)
    let close_timeout_id = setup_auto_close(&window, &search_entry);
    
    // Cancelar auto-close quando usuário digitar (proteção adicional)
    let search_entry_for_typing = search_entry.clone();
    search_entry.connect_changed(move |_| {
        // Quando usuário digita, CANCELA qualquer timeout de fechamento pendente
        if let Some(id) = close_timeout_id.borrow_mut().take() {
            id.remove();
            eprintln!("⚡ Usuário digitando - auto-close CANCELADO!");
        }
    });

    eprintln!("🔵 Window: adw::ApplicationWindow, 700x550 (auto-close inteligente 1500ms + system notifications)");

    (window, list_box, scrolled, search_entry_for_typing)
}

fn setup_auto_close(window: &adw::ApplicationWindow, search_entry: &SearchEntry) -> Rc<RefCell<Option<gtk::glib::SourceId>>> {
    let window_for_focus = window.clone();
    let search_entry_for_focus = search_entry.clone();
    let close_timeout_id: Rc<RefCell<Option<gtk::glib::SourceId>>> = Rc::new(RefCell::new(None));
    let close_timeout_id_return = close_timeout_id.clone();

    // Delay inicial antes de habilitar auto-close (dar tempo para o usuário começar a usar)
    let window_for_init = window.clone();
    let search_entry_for_init = search_entry.clone();
    let close_timeout_for_init = close_timeout_id.clone();
    
    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(300), move || {
        eprintln!("🔵 Auto-close habilitado após 300ms");

        window_for_init.connect_is_active_notify(move |win| {
            if !win.is_active() {
                // CRÍTICO: Não fechar se há texto no campo de pesquisa (usuário está usando)
                let search_text = search_entry_for_init.text();
                if !search_text.is_empty() {
                    eprintln!("⏸️  Popup perdeu foco MAS há texto no campo ('{}') - NÃO fechando!", search_text);
                    return;
                }
                
                eprintln!("🔴 Popup perdeu o foco (campo vazio) - aguardando 1500ms antes de fechar...");
                
                // Cancelar timeout anterior se existir (usuário voltou o foco rapidamente)
                if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                    id.remove();
                    eprintln!("   ↩️  Timeout anterior cancelado");
                }
                
                // Agendar fechamento após 1500ms (dar tempo para retornar foco)
                let window_to_close = window_for_focus.clone();
                let search_entry_to_check = search_entry_for_focus.clone();
                let timeout_id = gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(1500),
                    move || {
                        // Verificar novamente se há texto no campo antes de fechar
                        let search_text = search_entry_to_check.text();
                        if !search_text.is_empty() {
                            eprintln!("   ⏸️  Ainda há texto no campo ('{}') - NÃO fechando!", search_text);
                            return;
                        }
                        
                        if !window_to_close.is_active() {
                            eprintln!("   ✅ Fechando popup (sem foco por 1500ms, campo vazio)");
                            window_to_close.close();
                        } else {
                            eprintln!("   ⏸️  Não fechando - foco recuperado!");
                        }
                    }
                );
                *close_timeout_for_init.borrow_mut() = Some(timeout_id);
            } else {
                eprintln!("🟢 Popup ganhou o foco - cancelando auto-close");
                // Cancelar timeout se ganhar foco de volta
                if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                    id.remove();
                    eprintln!("   ↩️  Auto-close cancelado (foco recuperado)");
                }
            }
        });
    });
    
    close_timeout_id_return
}
