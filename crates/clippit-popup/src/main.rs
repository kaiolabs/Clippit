use anyhow::Result;
use clippit_core::{set_language, Config};
use gtk::prelude::*;
use gtk::{gio, Application};
use libadwaita as adw;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

mod controllers;
mod models;
mod panic_handler;
mod utils;
mod views;

use controllers::{setup_keyboard_navigation, setup_row_activation};
use models::{new_entry_map, new_search_content_map};
use panic_handler::setup_panic_handler;
use utils::{apply_theme, load_custom_css};
use views::list_item::LoadMoreState;
use views::{
    create_main_window, populate_history_list, setup_infinite_scroll, setup_search_filter,
};

const APP_ID: &str = "com.clippit.Clippit";

// Initialize i18n pointing to the same locales as clippit-core
rust_i18n::i18n!("../clippit-core/locales", fallback = "en");

/// Lock file das versões antigas (toggle via SIGTERM). Não é mais usado: o toggle agora
/// é feito pela própria GApplication (instância única via D-Bus).
const LEGACY_LOCK_FILE: &str = "/tmp/clippit-popup.lock";

/// Popup residente:
/// - A 1ª execução vira a instância principal e fica rodando com a janela escondida.
/// - Execuções seguintes (atalho do GNOME / daemon) só mandam "activate" via D-Bus para
///   a instância principal e saem em poucos ms, sem inicializar GTK.
/// - "activate" alterna: janela visível → esconde; escondida → atualiza a lista e mostra.
/// - `--background`: só pré-carrega (usado no login), não mostra a janela.
fn main() -> Result<()> {
    let background = std::env::args().any(|a| a == "--background");

    let app = Application::builder().application_id(APP_ID).build();

    // Handlers precisam existir ANTES do register(): é nele que o "startup" é emitido
    let popup: Rc<RefCell<Option<Rc<Popup>>>> = Rc::new(RefCell::new(None));

    let popup_for_startup = popup.clone();
    app.connect_startup(move |app| {
        // Só roda na instância principal (dentro do register())
        setup_panic_handler();
        std::fs::remove_file(LEGACY_LOCK_FILE).ok();
        adw::init().expect("Failed to initialize libadwaita");
        *popup_for_startup.borrow_mut() = Some(build_ui(app));
    });

    // Com --background a primeira ativação (emitida pelo próprio run) só pré-carrega
    let skip_first_activate = Cell::new(background);
    app.connect_activate(move |_| {
        let Some(popup) = popup.borrow().clone() else {
            return;
        };
        if skip_first_activate.replace(false) {
            eprintln!("💤 Pré-carregado em background - aguardando atalho");
            return;
        }
        if popup.window.is_visible() {
            eprintln!("🔄 Atalho com popup aberto - escondendo (toggle)");
            popup.window.close();
        } else {
            show_popup(&popup);
        }
    });

    // Registrar antes de qualquer inicialização pesada para descobrir se já existe
    // uma instância residente
    if let Err(e) = app.register(gio::Cancellable::NONE) {
        eprintln!(
            "⚠️  Falha ao registrar GApplication: {} (seguindo sem instância única)",
            e
        );
    }

    if app.is_remote() {
        if background {
            // Já existe instância residente - nada a fazer
            return Ok(());
        }
        // Envia "activate" (toggle) para a instância residente e sai
        app.run_with_args(&Vec::<String>::new());
        return Ok(());
    }

    // ===== Daqui para baixo: instância principal (residente) =====
    eprintln!(
        "🚀 Clippit Popup (residente) iniciado{}",
        if background { " em background" } else { "" }
    );

    // SIGTERM encerra a instância residente de forma limpa
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    let should_quit = Arc::new(AtomicBool::new(false));
    let should_quit_clone = should_quit.clone();
    unsafe {
        signal_hook::low_level::register(signal_hook::consts::SIGTERM, move || {
            should_quit_clone.store(true, Ordering::Relaxed);
        })
        .ok();
    }
    let app_for_quit = app.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(200), move || {
        if should_quit.load(Ordering::Relaxed) {
            eprintln!("🛑 SIGTERM recebido - encerrando popup residente");
            app_for_quit.quit();
            gtk::glib::ControlFlow::Break
        } else {
            gtk::glib::ControlFlow::Continue
        }
    });

    // Mantém o processo vivo com a janela escondida
    let _hold = app.hold();

    let exit_code = app.run_with_args(&Vec::<String>::new());
    if exit_code.value() == 0 {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Application exited with code {}",
            exit_code.value()
        ))
    }
}

/// Estado da janela residente
struct Popup {
    window: adw::ApplicationWindow,
    app: Application,
    list_box: gtk::ListBox,
    scrolled: gtk::ScrolledWindow,
    search_entry: gtk::SearchEntry,
    entry_map: models::EntryMap,
    search_map: models::SearchContentMap,
    load_state: Rc<RefCell<LoadMoreState>>,
    suppress_search: Rc<Cell<bool>>,
}

/// Cria a janela e todos os handlers UMA vez (na inicialização da instância residente)
fn build_ui(app: &Application) -> Rc<Popup> {
    let config = Config::load().unwrap_or_default();
    set_language(&config.ui.language);
    apply_theme(&config);
    load_custom_css();

    let (window, list_box, scrolled, search_entry, close_timeout_id) = create_main_window(app);
    window.add_css_class("clippit-popup");

    // Fechar = esconder (o processo continua residente para reabrir instantaneamente)
    window.connect_close_request(|win| {
        win.set_visible(false);
        gtk::glib::Propagation::Stop
    });

    let entry_map = new_entry_map();
    let search_map = new_search_content_map();
    let suppress_search = Rc::new(Cell::new(false));

    setup_keyboard_navigation(
        &window,
        app,
        &list_box,
        &scrolled,
        &entry_map,
        &search_entry,
    );
    setup_search_filter(
        &list_box,
        &search_entry,
        &search_map,
        &window,
        app,
        &entry_map,
        Some(close_timeout_id),
        suppress_search.clone(),
    );
    setup_row_activation(&list_box, &entry_map, &window, app);
    let load_state =
        setup_infinite_scroll(&scrolled, &list_box, &window, app, &entry_map, &search_map);

    // Pré-carrega a lista (a 1ª abertura já encontra os widgets prontos)
    populate_history_list(&list_box, &window, app, &entry_map, &search_map);
    // Pré-aquecer: cria a superfície/renderer e calcula o layout das linhas agora,
    // com a janela ainda escondida, para a 1ª abertura também ser instantânea
    WidgetExt::realize(&window);
    let _ = WidgetExt::measure(&window, gtk::Orientation::Vertical, 700);

    Rc::new(Popup {
        window,
        app: app.clone(),
        list_box,
        scrolled,
        search_entry,
        entry_map,
        search_map,
        load_state,
        suppress_search,
    })
}

/// Reseta o estado (busca, scroll, seleção), atualiza a lista e mostra a janela
fn show_popup(p: &Popup) {
    let t = std::time::Instant::now();

    crate::views::lightbox::hide();

    // Limpar busca anterior sem disparar uma nova busca
    p.suppress_search.set(true);
    p.search_entry.set_text("");
    p.suppress_search.set(false);

    // Atualiza só se houver entradas novas (senão reaproveita os widgets)
    populate_history_list(&p.list_box, &p.window, &p.app, &p.entry_map, &p.search_map);
    {
        let mut st = p.load_state.borrow_mut();
        st.items_loaded = p.entry_map.borrow().len();
        st.has_more = true;
        st.is_loading = false;
    }

    p.scrolled.vadjustment().set_value(0.0);
    let first_row = p.list_box.row_at_index(0);
    if let Some(ref row) = first_row {
        p.list_box.select_row(Some(row));
    }

    p.window.present();
    p.window.set_focus_visible(true);
    if let Some(row) = first_row {
        row.grab_focus();
    }
    eprintln!("⚡ Popup mostrado em {:?}", t.elapsed());
}
