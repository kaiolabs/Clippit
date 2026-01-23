use gtk::prelude::*;
use std::env;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Uso: clippit-tooltip \"texto das sugestões\"");
        std::process::exit(1);
    }

    let text = &args[1];
    
    gtk::init().unwrap();
    
    // Janela tooltip - sem decoração, sem foco
    let window = gtk::Window::new();
    window.set_decorated(false);
    window.set_resizable(false);
    window.set_default_size(280, 130);
    window.set_focusable(false);
    window.set_can_focus(false);
    window.set_modal(false);
    
    // CSS para aparência de tooltip escuro
    let css = r#"
        window {
            background-color: rgba(32, 32, 32, 0.96);
            border-radius: 10px;
            border: 1px solid rgba(255, 255, 255, 0.15);
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
        }
        label {
            color: #f0f0f0;
            font-family: monospace;
            font-size: 13px;
            padding: 16px;
        }
    "#;
    
    let provider = gtk::CssProvider::new();
    provider.load_from_data(css);
    gtk::style_context_add_provider_for_display(
        &gtk::prelude::WidgetExt::display(&window),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    // Label com o texto
    let label = gtk::Label::new(Some(text));
    label.set_halign(gtk::Align::Start);
    label.set_valign(gtk::Align::Start);
    window.set_child(Some(&label));
    
    window.present();
    
    // Auto-fechar após 3 segundos
    let window_weak = window.downgrade();
    gtk::glib::timeout_add_local_once(Duration::from_secs(3), move || {
        if let Some(w) = window_weak.upgrade() {
            w.close();
        }
    });
    
    // Main loop
    let main_loop = gtk::glib::MainLoop::new(None, false);
    let loop_clone = main_loop.clone();
    window.connect_close_request(move |_| {
        loop_clone.quit();
        gtk::glib::Propagation::Proceed
    });
    main_loop.run();
}
