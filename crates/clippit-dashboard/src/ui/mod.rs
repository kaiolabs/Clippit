mod general;
mod hotkeys;
mod search;
mod theme;
mod privacy;
mod autocomplete;

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
    sidebar.set_width_request(200);

    // Create menu items with icons
    let items = vec![
        (t!("menu.general").to_string(), "preferences-system-symbolic"),
        (t!("menu.hotkeys").to_string(), "input-keyboard-symbolic"),
        ("Pesquisa".to_string(), "edit-find-symbolic"),
        ("Autocompletar".to_string(), "input-keyboard-symbolic"),
        (t!("menu.theme").to_string(), "applications-graphics-symbolic"),
        (t!("menu.privacy").to_string(), "security-high-symbolic"),
    ];

    for (label, icon) in items {
        let row = gtk::ListBoxRow::new();
        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        hbox.set_margin_top(8);
        hbox.set_margin_bottom(8);
        
        let icon_widget = gtk::Image::from_icon_name(icon);
        let label_widget = gtk::Label::new(Some(&label));
        label_widget.set_halign(gtk::Align::Start);
        
        hbox.append(&icon_widget);
        hbox.append(&label_widget);
        row.set_child(Some(&hbox));
        
        sidebar.append(&row);
    }

    // Create stack for pages
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(200);
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    
    stack.add_named(&general::create_page(), Some("0"));
    stack.add_named(&hotkeys::create_page(), Some("1"));
    stack.add_named(&search::create_page(), Some("2"));
    stack.add_named(&autocomplete::create_page(), Some("3"));
    stack.add_named(&theme::create_page(), Some("4"));
    stack.add_named(&privacy::create_page(), Some("5"));

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
    paned.set_position(200);
    paned.set_shrink_start_child(false);
    paned.set_resize_start_child(false);

    // Create main container
    let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&paned);

    main_box.upcast()
}
