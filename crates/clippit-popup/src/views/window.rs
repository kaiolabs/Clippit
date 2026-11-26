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
    search_entry.add_css_class("clip-search");

    // Create list box for history items
    let list_box = gtk::ListBox::new();
    list_box.add_css_class("clip-list");
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.set_can_focus(true);
    list_box.set_focus_on_click(false);
    list_box.set_activate_on_single_click(true); // 🔥 SINGLE CLICK para copiar!

    // Create scrolled window
    let scrolled = ScrolledWindow::new();
    scrolled.set_child(Some(&list_box));
    scrolled.set_vexpand(true);
    scrolled.set_margin_start(20);
    scrolled.set_margin_end(20);
    scrolled.set_hscrollbar_policy(gtk::PolicyType::Never);
    scrolled.add_css_class("clip-scroll");

    // Create main vertical box
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);

    main_box.add_css_class("clip-root");

    // Busca
    let search_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    search_box.set_margin_top(20);
    search_box.set_margin_start(20);
    search_box.set_margin_end(20);
    search_box.set_margin_bottom(14);
    search_box.append(&search_entry);

    // Rodapé com atalhos
    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    footer.add_css_class("clip-footer");
    footer.set_margin_top(12);
    footer.set_margin_start(20);
    footer.set_margin_end(20);
    footer.set_margin_bottom(20);
    // Atalho de busca vem da configuração (ex.: "ctrl" + "p" → "Ctrl+P")
    let config = clippit_core::Config::load().unwrap_or_default();
    let search_key = format_hotkey(&format!(
        "{}+{}",
        config.search.focus_search_modifier, config.search.focus_search_key
    ));

    for (key, label) in [
        (search_key.as_str(), t!("popup.hint_search").to_string()),
        ("↑↓", t!("popup.hint_navigate").to_string()),
        ("Enter", t!("popup.hint_copy").to_string()),
        ("Espaço", t!("popup.hint_preview").to_string()),
        ("Esc", t!("popup.hint_close").to_string()),
    ] {
        let hint = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let k = gtk::Label::new(Some(key));
        k.add_css_class("clip-key");
        let l = gtk::Label::new(Some(&label));
        l.add_css_class("clip-hint");
        hint.append(&k);
        hint.append(&l);
        footer.append(&hint);
    }

    main_box.append(&search_box);
    main_box.append(&scrolled);
    main_box.append(&footer);

    // Overlay para o modo de imagem ampliada (fica por cima de tudo)
    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&main_box));
    crate::views::lightbox::install(&overlay);

    // Create main window (no toast overlay - using system notifications)
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(t!("popup.title").to_string())
        .default_width(720)
        .default_height(600)
        .content(&overlay)
        .build();

    // Auto-close on focus loss with intelligent delay (retorna timeout_id para passar ao search_filter)
    let close_timeout_id = setup_auto_close(&window, &search_entry);

    eprintln!("🔵 Window: adw::ApplicationWindow, 700x550 (auto-close inteligente 1500ms + system notifications)");

    (window, list_box, scrolled, search_entry, close_timeout_id)
}

fn setup_auto_close(
    window: &adw::ApplicationWindow,
    search_entry: &SearchEntry,
) -> Rc<RefCell<Option<gtk::glib::SourceId>>> {
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
                        let _ = id;
                        eprintln!("   ↩️  Timeout existente cancelado (via drop)");
                    }
                    return;
                }

                eprintln!("🔴 Popup perdeu foco (campo vazio) - aguardando 3000ms...");

                // Cancelar timeout anterior se existir (usuário voltou o foco rapidamente)
                if let Some(id) = close_timeout_for_init.borrow_mut().take() {
                    let _ = id; // NÃO chamar remove() - deixa o GTK limpar
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
                    let _ = id; // NÃO chamar remove() - deixa o GTK limpar
                    eprintln!("   ↩️  Auto-close cancelado (foco recuperado via drop)");
                }
            }
        });
    });

    close_timeout_id_return
}

/// Formata um atalho da config para exibição: "ctrl+shift+p" → "Ctrl+Shift+P"
fn format_hotkey(raw: &str) -> String {
    raw.split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .map(|p| match p.to_lowercase().as_str() {
            "ctrl" | "control" | "primary" => "Ctrl".to_string(),
            "super" | "meta" | "win" => "Super".to_string(),
            "kp_1" => "Num 1".to_string(),
            other => {
                let mut c = other.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join("+")
}
