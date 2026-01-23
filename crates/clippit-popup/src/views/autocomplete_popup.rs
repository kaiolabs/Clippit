use gtk::prelude::*;
use gtk::{gdk, glib};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use clippit_ipc::protocol::Suggestion;
use tracing::{debug, info};

/// Popup flutuante de autocomplete (estilo Android/iOS)
pub struct AutocompletePopup {
    window: gtk::Window,
    list_box: gtk::ListBox,
    suggestions: Rc<RefCell<Vec<Suggestion>>>,
    selected_index: Rc<RefCell<usize>>,
    on_accept: Arc<Mutex<Option<Box<dyn Fn(String) + Send>>>>,
}

impl AutocompletePopup {
    pub fn new(app: &gtk::Application) -> Rc<RefCell<Self>> {
        // Janela flutuante (sem decoração)
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .resizable(false)
            .modal(false)
            .default_width(300)
            .default_height(200)
            .build();

        // Tornar janela sempre no topo
        window.set_keep_above(true);
        
        // Estilo CSS
        let css = gtk::CssProvider::new();
        css.load_from_data(
            br#"
            window {
                background: alpha(@window_bg_color, 0.95);
                border-radius: 12px;
                border: 1px solid alpha(@borders, 0.3);
                box-shadow: 0 8px 24px rgba(0,0,0,0.3);
            }
            
            .suggestion-row {
                padding: 12px 16px;
                border-radius: 8px;
                margin: 4px;
                transition: all 200ms;
            }
            
            .suggestion-row:hover {
                background: alpha(@accent_bg_color, 0.1);
            }
            
            .suggestion-row:selected {
                background: @accent_bg_color;
                color: @accent_fg_color;
            }
            
            .suggestion-word {
                font-size: 14px;
                font-weight: 500;
            }
            
            .suggestion-score {
                font-size: 11px;
                opacity: 0.6;
                margin-left: 8px;
            }
            "#
        );
        
        gtk::style_context_add_provider_for_display(
            &window.display(),
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        // ListBox para sugestões
        let list_box = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::Single)
            .css_classes(vec!["suggestions-list"])
            .build();

        // ScrolledWindow
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .max_content_height(250)
            .propagate_natural_height(true)
            .child(&list_box)
            .build();

        // Box principal com padding
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .margin_top(8)
            .margin_bottom(8)
            .margin_start(8)
            .margin_end(8)
            .build();

        main_box.append(&scrolled);
        window.set_child(Some(&main_box));

        let suggestions = Rc::new(RefCell::new(Vec::new()));
        let selected_index = Rc::new(RefCell::new(0));
        let on_accept = Arc::new(Mutex::new(None));

        let popup = Rc::new(RefCell::new(Self {
            window: window.clone(),
            list_box: list_box.clone(),
            suggestions: suggestions.clone(),
            selected_index: selected_index.clone(),
            on_accept: on_accept.clone(),
        }));

        // Conectar eventos de teclado
        let key_controller = gtk::EventControllerKey::new();
        let popup_clone = Rc::clone(&popup);
        
        key_controller.connect_key_pressed(move |_, keyval, _, _| {
            let popup_ref = popup_clone.borrow();
            
            match keyval {
                gdk::Key::Up => {
                    popup_ref.navigate_up();
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    popup_ref.navigate_down();
                    glib::Propagation::Stop
                }
                gdk::Key::Tab | gdk::Key::Return | gdk::Key::KP_Enter => {
                    popup_ref.accept_current();
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    popup_ref.hide();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        window.add_controller(key_controller);

        // Conectar clique em item
        let popup_clone2 = Rc::clone(&popup);
        list_box.connect_row_activated(move |_, row| {
            let popup_ref = popup_clone2.borrow();
            let index = row.index() as usize;
            *popup_ref.selected_index.borrow_mut() = index;
            popup_ref.accept_current();
        });

        popup
    }

    /// Mostra popup com sugestões na posição especificada
    pub fn show(&self, suggestions: Vec<Suggestion>, x: i32, y: i32) {
        if suggestions.is_empty() {
            return;
        }

        info!("📍 Mostrando popup com {} sugestões em ({}, {})", suggestions.len(), x, y);

        *self.suggestions.borrow_mut() = suggestions.clone();
        *self.selected_index.borrow_mut() = 0;

        // Limpar lista
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Adicionar sugestões
        for (i, sugg) in suggestions.iter().enumerate() {
            let row = self.create_suggestion_row(sugg, i == 0);
            self.list_box.append(&row);
        }

        // Selecionar primeira
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }

        // Posicionar janela
        // Nota: Wayland não permite posicionamento absoluto via GTK
        // Vamos usar uma abordagem alternativa com layer-shell ou simplesmente centro
        self.window.present();
        self.window.set_focus_visible(true);

        debug!("✅ Popup exibido");
    }

    /// Esconde popup
    pub fn hide(&self) {
        self.window.hide();
        debug!("❌ Popup escondido");
    }

    /// Cria linha de sugestão
    fn create_suggestion_row(&self, suggestion: &Suggestion, selected: bool) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::builder()
            .css_classes(vec!["suggestion-row"])
            .build();

        let hbox = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();

        // Ícone indicador
        let indicator = if selected {
            gtk::Label::new(Some("➜"))
        } else {
            gtk::Label::new(Some(" "))
        };
        indicator.set_width_chars(2);
        hbox.append(&indicator);

        // Palavra
        let word_label = gtk::Label::builder()
            .label(&suggestion.word)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .css_classes(vec!["suggestion-word"])
            .build();
        hbox.append(&word_label);

        // Score (opcional, pode remover)
        // let score_label = gtk::Label::builder()
        //     .label(&format!("{}", suggestion.score))
        //     .halign(gtk::Align::End)
        //     .css_classes(vec!["suggestion-score"])
        //     .build();
        // hbox.append(&score_label);

        row.set_child(Some(&hbox));
        row
    }

    /// Navega para cima
    fn navigate_up(&self) {
        let mut index = self.selected_index.borrow_mut();
        let len = self.suggestions.borrow().len();
        
        if len == 0 {
            return;
        }

        if *index == 0 {
            *index = len - 1;
        } else {
            *index -= 1;
        }

        if let Some(row) = self.list_box.row_at_index(*index as i32) {
            self.list_box.select_row(Some(&row));
        }

        debug!("⬆️ Navegando para sugestão {}/{}", *index + 1, len);
    }

    /// Navega para baixo
    fn navigate_down(&self) {
        let mut index = self.selected_index.borrow_mut();
        let len = self.suggestions.borrow().len();
        
        if len == 0 {
            return;
        }

        *index = (*index + 1) % len;

        if let Some(row) = self.list_box.row_at_index(*index as i32) {
            self.list_box.select_row(Some(&row));
        }

        debug!("⬇️ Navegando para sugestão {}/{}", *index + 1, len);
    }

    /// Aceita sugestão atual
    fn accept_current(&self) {
        let index = *self.selected_index.borrow();
        let suggestions = self.suggestions.borrow();

        if let Some(suggestion) = suggestions.get(index) {
            info!("✅ Aceitando sugestão: '{}'", suggestion.word);
            
            if let Some(callback) = self.on_accept.lock().unwrap().as_ref() {
                callback(suggestion.word.clone());
            }

            self.hide();
        }
    }

    /// Define callback quando aceitar sugestão
    pub fn set_on_accept<F>(&self, callback: F)
    where
        F: Fn(String) + Send + 'static,
    {
        *self.on_accept.lock().unwrap() = Some(Box::new(callback));
    }
}
