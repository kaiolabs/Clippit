mod assistente;
mod atalhos;
mod conf;
mod corretor;
mod funcionalidades;
mod general;
mod hotkeys;
mod modelos;
pub mod style;

use gtk::prelude::*;
use libadwaita as adw;
use rust_i18n::t;

pub fn create_content() -> gtk::Widget {
    // Create header bar with window controls
    let header = adw::HeaderBar::new();
    header.set_show_end_title_buttons(true);
    header.set_show_start_title_buttons(true);

    let title = adw::WindowTitle::new(&t!("dashboard.title"), &t!("dashboard.subtitle"));
    header.set_title_widget(Some(&title));

    // Create navigation sidebar
    let sidebar = gtk::ListBox::new();
    sidebar.add_css_class("navigation-sidebar");
    sidebar.add_css_class("clip-sidebar");
    sidebar.set_width_request(210);

    // Create menu items with icons
    let items = vec![
        ("Geral", "preferences-system-symbolic"),
        ("Funcionalidades", "view-app-grid-symbolic"),
        ("Modelos de IA", "starred-symbolic"),
        ("Corretor", "tools-check-spelling-symbolic"),
        ("Assistente", "mail-reply-sender-symbolic"),
        ("Atalhos", "input-keyboard-symbolic"),
    ];

    for (label, icon) in items {
        let row = gtk::ListBoxRow::new();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        hbox.set_margin_start(10);
        hbox.set_margin_end(10);
        hbox.set_margin_top(9);
        hbox.set_margin_bottom(9);

        let icon_widget = gtk::Image::from_icon_name(icon);
        let label_widget = gtk::Label::new(Some(label));
        label_widget.set_halign(gtk::Align::Start);

        hbox.append(&icon_widget);
        hbox.append(&label_widget);
        row.set_child(Some(&hbox));

        sidebar.append(&row);
    }

    // Create stack for pages
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::None);
    stack.set_hexpand(true);
    stack.set_vexpand(true);

    stack.add_named(&general::create_page(), Some("0"));
    stack.add_named(&funcionalidades::create_page(), Some("1"));
    stack.add_named(&modelos::create_page(), Some("2"));
    stack.add_named(&corretor::create_page(), Some("3"));
    stack.add_named(&assistente::create_page(), Some("4"));
    stack.add_named(&hotkeys::create_page(), Some("5"));

    // Connect sidebar selection to stack
    let stack_clone = stack.clone();
    sidebar.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            let index = row.index();
            stack_clone.set_visible_child_name(&index.to_string());
        }
    });

    // Select first item by default
    sidebar.select_row(sidebar.row_at_index(0).as_ref());

    // Create main horizontal split
    let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&stack));
    paned.set_position(210);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(false);

    // Create main container
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&paned);

    main_box.upcast()
}
