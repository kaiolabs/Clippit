use gtk::prelude::*;
use std::sync::{Arc, Mutex};

/// Estrutura para armazenar informações de panic
#[derive(Clone)]
pub struct PanicInfo {
    pub message: String,
    pub location: String,
    pub backtrace: String,
}

/// Mostra uma janela de erro com o panic e botão para copiar
pub fn show_panic_dialog(panic_info: PanicInfo) {
    gtk::glib::idle_add_once(move || {
        let dialog = gtk::MessageDialog::builder()
            .message_type(gtk::MessageType::Error)
            .buttons(gtk::ButtonsType::None)
            .title("💥 Clippit Popup - Erro Fatal")
            .text("O aplicativo encontrou um erro inesperado e precisa fechar.")
            .build();

        // Criar área de texto com todo o erro
        let content_area = dialog.content_area();

        let label = gtk::Label::new(Some("Detalhes do erro:"));
        label.set_halign(gtk::Align::Start);
        label.set_margin_top(10);
        content_area.append(&label);

        let scrolled = gtk::ScrolledWindow::builder()
            .height_request(300)
            .width_request(600)
            .margin_top(5)
            .margin_bottom(10)
            .build();

        let text_view = gtk::TextView::builder()
            .editable(false)
            .monospace(true)
            .wrap_mode(gtk::WrapMode::Word)
            .build();

        let buffer = text_view.buffer();
        let error_text = format!(
            "🔴 ERRO FATAL - PANIC DETECTADO\n\n\
            📍 Localização:\n{}\n\n\
            💬 Mensagem:\n{}\n\n\
            📚 Stack Trace:\n{}",
            panic_info.location, panic_info.message, panic_info.backtrace
        );
        buffer.set_text(&error_text);

        scrolled.set_child(Some(&text_view));
        content_area.append(&scrolled);

        // Botão para copiar
        let copy_button = gtk::Button::builder().label("📋 Copiar Erro").build();

        let error_text_clone = error_text.clone();
        copy_button.connect_clicked(move |_| {
            let clipboard = gtk::gdk::Display::default()
                .expect("Failed to get display")
                .clipboard();
            clipboard.set_text(&error_text_clone);
            eprintln!("✅ Erro copiado para área de transferência!");
        });

        dialog.add_action_widget(&copy_button, gtk::ResponseType::None);

        // Botão para fechar
        let close_button = gtk::Button::builder().label("❌ Fechar").build();

        close_button.connect_clicked(move |_| {
            std::process::exit(1);
        });

        dialog.add_action_widget(&close_button, gtk::ResponseType::Close);

        dialog.present();
    });
}

/// Configura o panic handler global
pub fn setup_panic_handler() {
    // Armazenar info do panic para passar ao GTK thread
    let panic_info_storage: Arc<Mutex<Option<PanicInfo>>> = Arc::new(Mutex::new(None));
    let panic_info_storage_clone = panic_info_storage.clone();

    std::panic::set_hook(Box::new(move |panic_info_hook| {
        eprintln!("💥💥💥 PANIC DETECTADO! 💥💥💥");

        let location = if let Some(loc) = panic_info_hook.location() {
            format!("{}:{}:{}", loc.file(), loc.line(), loc.column())
        } else {
            "Localização desconhecida".to_string()
        };

        let message = if let Some(s) = panic_info_hook.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = panic_info_hook.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            format!("{:?}", panic_info_hook)
        };

        // Capturar backtrace (requer RUST_BACKTRACE=1)
        let backtrace = std::backtrace::Backtrace::force_capture();
        let backtrace_str = format!("{}", backtrace);

        eprintln!("Localização: {}", location);
        eprintln!("Mensagem: {}", message);
        eprintln!("Stack Trace:\n{}", backtrace_str);
        eprintln!("💥💥💥 FIM DO PANIC 💥💥💥");

        // Armazenar info
        let info = PanicInfo {
            message: message.clone(),
            location: location.clone(),
            backtrace: backtrace_str,
        };

        *panic_info_storage_clone.lock().unwrap() = Some(info.clone());

        // Remover lock file para que o próximo atalho abra um popup novo
        std::fs::remove_file("/tmp/clippit-popup.lock").ok();

        // Mostrar dialog no GTK thread
        show_panic_dialog(info);

        // IMPORTANTE: nunca bloquear aqui. O panic quase sempre acontece na thread
        // principal do GTK (dentro de callbacks); um sleep aqui congelava a janela
        // (skeletons parados + "não está respondendo") e o dialog nunca aparecia.
        // Deixamos o unwind seguir: o glib aborta o callback que entrou em panic.
        // Se o panic for em outra thread, o dialog aparece normalmente.
        if gtk::is_initialized_main_thread() {
            // Panic na thread do GTK: não dá para mostrar o dialog com o loop
            // travado/abortando. Avisar via notificação e sair em vez de congelar.
            let _ = std::process::Command::new("notify-send")
                .args(["-a", "Clippit", "-u", "critical", "-i", "dialog-error"])
                .arg("Clippit Popup - erro")
                .arg(format!("{}\n{}", location, message))
                .spawn();
            std::process::exit(1);
        }
    }));

    eprintln!("✅ Panic handler configurado - erros serão exibidos em modal");
}
