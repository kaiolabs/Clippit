use clippit_core::Config;
use gtk::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::*;

pub fn create_page() -> gtk::Widget {
    let page = adw::PreferencesPage::new();
    page.set_title("Autocompletar Global");
    page.set_icon_name(Some("input-keyboard-symbolic"));

    // ========== SEÇÃO: CONFIGURAÇÕES GERAIS ==========
    let group_general = adw::PreferencesGroup::new();
    group_general.set_title("Configurações Gerais");
    group_general.set_description(Some("Configure o autocomplete global do sistema"));

    // Habilitar/Desabilitar
    let row_enable = adw::ActionRow::new();
    row_enable.set_title("Habilitar Autocomplete Global");
    row_enable.set_subtitle("Sugestões de palavras em qualquer aplicação");
    
    let config = Config::load().unwrap_or_default();
    
    let switch_enable = gtk::Switch::new();
    switch_enable.set_active(config.autocomplete.enabled);
    switch_enable.set_valign(gtk::Align::Center);
    
    switch_enable.connect_active_notify(|switch| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.enabled = switch.is_active();
        let _ = cfg.save();
        eprintln!("✅ Autocomplete enabled: {}", switch.is_active());
    });
    
    row_enable.add_suffix(&switch_enable);
    row_enable.set_activatable_widget(Some(&switch_enable));

    group_general.add(&row_enable);

    // Máximo de Sugestões
    let row_max = adw::ActionRow::new();
    row_max.set_title("Máximo de Sugestões");
    row_max.set_subtitle("Quantidade de sugestões a mostrar (1-10)");
    
    let spin_max = gtk::SpinButton::with_range(1.0, 10.0, 1.0);
    spin_max.set_value(config.autocomplete.max_suggestions as f64);
    spin_max.set_valign(gtk::Align::Center);
    
    spin_max.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.max_suggestions = spin.value() as usize;
        let _ = cfg.save();
        eprintln!("✅ Max suggestions: {}", spin.value());
    });
    
    row_max.add_suffix(&spin_max);
    group_general.add(&row_max);

    // Caracteres Mínimos
    let row_min_chars = adw::ActionRow::new();
    row_min_chars.set_title("Caracteres Mínimos");
    row_min_chars.set_subtitle("Mínimo de letras para mostrar sugestões");
    
    let spin_min = gtk::SpinButton::with_range(1.0, 5.0, 1.0);
    spin_min.set_value(config.autocomplete.min_chars as f64);
    spin_min.set_valign(gtk::Align::Center);
    
    spin_min.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.min_chars = spin.value() as usize;
        let _ = cfg.save();
        eprintln!("✅ Min chars: {}", spin.value());
    });
    
    row_min_chars.add_suffix(&spin_min);
    group_general.add(&row_min_chars);

    // Delay
    let row_delay = adw::ActionRow::new();
    row_delay.set_title("Atraso (ms)");
    row_delay.set_subtitle("Tempo antes de mostrar sugestões (100-1000ms)");
    
    let spin_delay = gtk::SpinButton::with_range(100.0, 1000.0, 50.0);
    spin_delay.set_value(config.autocomplete.delay_ms as f64);
    spin_delay.set_valign(gtk::Align::Center);
    
    spin_delay.connect_value_changed(|spin| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.delay_ms = spin.value() as u64;
        let _ = cfg.save();
        eprintln!("✅ Delay: {}ms", spin.value());
    });
    
    row_delay.add_suffix(&spin_delay);
    group_general.add(&row_delay);

    page.add(&group_general);

    // ========== SEÇÃO: PRIVACIDADE E SEGURANÇA ==========
    let group_privacy = adw::PreferencesGroup::new();
    group_privacy.set_title("Privacidade e Segurança");

    // Mostrar em Senhas
    let row_passwords = adw::ActionRow::new();
    row_passwords.set_title("Mostrar em Campos de Senha");
    row_passwords.set_subtitle("⚠️ Não recomendado por segurança");
    
    let switch_passwords = gtk::Switch::new();
    switch_passwords.set_active(config.autocomplete.show_in_passwords);
    switch_passwords.set_valign(gtk::Align::Center);
    
    switch_passwords.connect_active_notify(|switch| {
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.show_in_passwords = switch.is_active();
        let _ = cfg.save();
        eprintln!("✅ Show in passwords: {}", switch.is_active());
    });
    
    row_passwords.add_suffix(&switch_passwords);
    row_passwords.set_activatable_widget(Some(&switch_passwords));

    group_privacy.add(&row_passwords);

    // Apps Ignorados
    let row_ignored = adw::ExpanderRow::new();
    row_ignored.set_title("Aplicativos Ignorados");
    row_ignored.set_subtitle("Apps onde autocomplete não funciona");
    row_ignored.set_expanded(false);

    // Lista de apps ignorados
    let scrolled = gtk::ScrolledWindow::new();
    scrolled.set_min_content_height(150);
    scrolled.set_margin_start(12);
    scrolled.set_margin_end(12);
    scrolled.set_margin_top(6);
    scrolled.set_margin_bottom(6);
    
    let text_view = gtk::TextView::new();
    text_view.set_wrap_mode(gtk::WrapMode::Word);
    text_view.set_margin_start(12);
    text_view.set_margin_end(12);
    text_view.set_margin_top(12);
    text_view.set_margin_bottom(12);
    
    let buffer = text_view.buffer();
    buffer.set_text(&config.autocomplete.ignored_apps.join("\n"));
    
    buffer.connect_changed(|buf| {
        let text = buf.text(&buf.start_iter(), &buf.end_iter(), false);
        let apps: Vec<String> = text
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
        
        let mut cfg = Config::load().unwrap_or_default();
        cfg.autocomplete.ignored_apps = apps;
        let _ = cfg.save();
        eprintln!("✅ Ignored apps updated");
    });
    
    scrolled.set_child(Some(&text_view));
    
    let ignored_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    ignored_box.append(&scrolled);
    
    let help_label = gtk::Label::new(Some("Um app por linha (ex: gnome-terminal, keepassxc)"));
    help_label.add_css_class("dim-label");
    help_label.set_margin_start(12);
    help_label.set_margin_end(12);
    help_label.set_margin_bottom(6);
    ignored_box.append(&help_label);
    
    row_ignored.add_row(&ignored_box);
    group_privacy.add(&row_ignored);

    page.add(&group_privacy);

    // ========== SEÇÃO: INSTALAÇÃO ==========
    let group_setup = adw::PreferencesGroup::new();
    group_setup.set_title("Instalação");
    group_setup.set_description(Some("Configure o IBus Input Method"));

    let row_install = adw::ActionRow::new();
    row_install.set_title("Como Ativar");
    row_install.set_subtitle("Configurações → Teclado → Fontes de Entrada → Adicionar \"Clippit\"");
    
    let icon_info = gtk::Image::from_icon_name("dialog-information-symbolic");
    row_install.add_prefix(&icon_info);
    
    group_setup.add(&row_install);

    // Botão de verificação
    let row_check = adw::ActionRow::new();
    row_check.set_title("Verificar Instalação");
    row_check.set_subtitle("Verifica se o IBus component está instalado");
    
    let btn_check = gtk::Button::with_label("Verificar");
    btn_check.add_css_class("suggested-action");
    btn_check.set_valign(gtk::Align::Center);
    
    let row_check_clone = row_check.clone();
    btn_check.connect_clicked(move |btn| {
        // Desabilitar botão durante verificação
        btn.set_sensitive(false);
        btn.set_label("Verificando...");
        
        // Verificar se IBus está instalado
        use std::process::Command;
        
        let result = Command::new("ibus")
            .arg("list-engine")
            .output();
        
        let message = match result {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("clippit") {
                    "✅ Clippit IBus engine está instalado e pronto para uso!"
                } else {
                    "⚠️ Clippit IBus engine NÃO encontrado.\n\nExecute no terminal:\nsudo bash scripts/install-ibus.sh"
                }
            }
            Err(_) => {
                "❌ IBus não está instalado no sistema.\n\nInstale com:\nsudo apt install ibus"
            }
        };
        
        // Criar diálogo de resultado
        let parent_window = btn.root().and_downcast::<gtk::Window>();
        let dialog = adw::MessageDialog::new(
            parent_window.as_ref(),
            Some("Verificação do IBus"),
            Some(message),
        );
        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));
        dialog.set_close_response("ok");
        
        let btn_clone = btn.clone();
        dialog.connect_response(None, move |dialog, _| {
            dialog.close();
            btn_clone.set_sensitive(true);
            btn_clone.set_label("Verificar");
        });
        
        dialog.present();
        
        // Também atualizar o subtitle da row
        row_check_clone.set_subtitle(message);
    });
    
    row_check.add_suffix(&btn_check);
    group_setup.add(&row_check);

    page.add(&group_setup);

    // ========== SEÇÃO: IA (FASE 2 - FUTURO) ==========
    let group_ai = adw::PreferencesGroup::new();
    group_ai.set_title("Inteligência Artificial (Fase 2)");
    group_ai.set_description(Some("🚧 Em desenvolvimento - Sugestões via IA"));

    let row_ai_enable = adw::ActionRow::new();
    row_ai_enable.set_title("Habilitar Sugestões IA");
    row_ai_enable.set_subtitle("Sugestões contextuais via modelo de IA");
    
    let switch_ai = gtk::Switch::new();
    switch_ai.set_active(config.autocomplete.ai.enabled);
    switch_ai.set_valign(gtk::Align::Center);
    switch_ai.set_sensitive(false); // Desabilitado por enquanto
    
    row_ai_enable.add_suffix(&switch_ai);
    row_ai_enable.set_activatable_widget(Some(&switch_ai));
    
    group_ai.add(&row_ai_enable);

    page.add(&group_ai);

    page.upcast()
}
