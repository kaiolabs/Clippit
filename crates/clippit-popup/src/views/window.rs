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
    Rc<RefCell<Option<gtk::glib::SourceId>>>,
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

    // Auto-close on focus loss with intelligent delay (retorna timeout_id para passar ao search_filter)
    let close_timeout_id = setup_auto_close(&window, &search_entry);

    eprintln!("🔵 Window: adw::ApplicationWindow, 700x550 (auto-close inteligente 1500ms + system notifications)");

    (window, list_box, scrolled, search_entry, close_timeout_id)
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
                // CRÍTICO: Não agendar timeout se há texto no campo de pesquisa
                let search_text = search_entry_for_init.text();
                if !search_text.is_empty() {
                    eprintln!("⏸️  Popup perdeu foco MAS há texto ('{}') - auto-close DESABILITADO!", search_text);
                    // Cancelar qualquer timeout existente (proteção adicional)
                    if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                        // NÃO chamar remove() - apenas dropar o SourceId
                        // O GTK remove automaticamente quando o SourceId é dropado
                        drop(id);
                        eprintln!("   ↩️  Timeout existente cancelado (via drop)");
                    }
                    return;
                }
                
                eprintln!("🔴 Popup perdeu foco (campo vazio) - aguardando 3000ms...");
                
                // Cancelar timeout anterior se existir (usuário voltou o foco rapidamente)
                if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                    drop(id); // NÃO chamar remove() - deixa o GTK limpar
                    eprintln!("   ↩️  Timeout anterior cancelado (via drop)");
                }
                
                // Agendar fechamento após 3000ms (tempo maior para evitar fechamento acidental)
                let window_to_close = window_for_focus.clone();
                let search_entry_to_check = search_entry_for_focus.clone();
                let timeout_id = gtk::glib::timeout_add_local_once(
                    std::time::Duration::from_millis(3000),
                    move || {
                        eprintln!("🔔 Auto-close timeout disparou após 3000ms - verificando condições...");
                        
                        // VERIFICAÇÃO 1: Há texto no campo?
                        let search_text = search_entry_to_check.text();
                        if !search_text.is_empty() {
                            eprintln!("   ⏸️  BLOQUEADO: há texto no campo ('{}') - NÃO fechando!", search_text);
                            return;
                        }
                        eprintln!("   ✓ Campo de pesquisa vazio");
                        
                        // VERIFICAÇÃO 2: Janela tem foco?
                        if window_to_close.is_active() {
                            eprintln!("   ⏸️  BLOQUEADO: janela está ativa - NÃO fechando!");
                            return;
                        }
                        eprintln!("   ✓ Janela não está ativa");
                        
                        // VERIFICAÇÃO 3: Campo de pesquisa tem foco?
                        if search_entry_to_check.has_focus() {
                            eprintln!("   ⏸️  BLOQUEADO: campo de pesquisa tem foco - NÃO fechando!");
                            return;
                        }
                        eprintln!("   ✓ Campo não tem foco");
                        
                        // TODAS as verificações passaram - pode fechar
                        eprintln!("   ✅ Fechando popup (sem foco por 3000ms, campo vazio, sem interação)");
                        window_to_close.close();
                    }
                );
                *close_timeout_for_init.borrow_mut() = Some(timeout_id);
            } else {
                eprintln!("🟢 Popup ganhou o foco - cancelando auto-close");
                // Cancelar timeout se ganhar foco de volta
                if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                    drop(id); // NÃO chamar remove() - deixa o GTK limpar
                    eprintln!("   ↩️  Auto-close cancelado (foco recuperado via drop)");
                }
            }
        });
    });
    
    close_timeout_id_return
}
