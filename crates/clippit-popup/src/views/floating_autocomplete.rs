use gtk::prelude::*;
use gtk::{gdk, glib};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Popup flutuante de autocomplete global
/// Aparece próximo ao cursor com sugestões de palavras
pub struct FloatingAutocomplete {
    window: gtk::Window,
    list_box: gtk::ListBox,
    suggestions: Rc<RefCell<Vec<Suggestion>>>,
    selected_index: Rc<RefCell<usize>>,
    on_accept: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
}

#[derive(Clone, Debug)]
pub struct Suggestion {
    pub word: String,
    pub score: i64,
}

impl FloatingAutocomplete {
    pub fn new(app: &adw::Application) -> Self {
        // Criar janela popup
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .resizable(false)
            .modal(false)
            .default_width(300)
            .default_height(150)
            .css_classes(vec!["floating-autocomplete".to_string()])
            .build();

        // ListBox para sugestões
        let list_box = gtk::ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.add_css_class("autocomplete-list");

        // ScrolledWindow
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        scrolled.set_max_content_height(200);
        scrolled.set_propagate_natural_height(true);
        scrolled.set_child(Some(&list_box));

        window.set_child(Some(&scrolled));

        let suggestions: Rc<RefCell<Vec<Suggestion>>> = Rc::new(RefCell::new(Vec::new()));
        let selected_index = Rc::new(RefCell::new(0));
        let on_accept: Rc<RefCell<Option<Box<dyn Fn(String)>>>> = Rc::new(RefCell::new(None));

        // Keyboard navigation
        let key_controller = gtk::EventControllerKey::new();
        let window_clone = window.clone();
        let list_box_clone = list_box.clone();
        let suggestions_clone = Rc::clone(&suggestions);
        let selected_index_clone = Rc::clone(&selected_index);
        let on_accept_clone = Rc::clone(&on_accept);

        key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gdk::Key::Down | gdk::Key::KP_Down => {
                    let mut idx = selected_index_clone.borrow_mut();
                    let suggs = suggestions_clone.borrow();
                    if *idx < suggs.len().saturating_sub(1) {
                        *idx += 1;
                        if let Some(row) = list_box_clone.row_at_index(*idx as i32) {
                            list_box_clone.select_row(Some(&row));
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Up | gdk::Key::KP_Up => {
                    let mut idx = selected_index_clone.borrow_mut();
                    if *idx > 0 {
                        *idx -= 1;
                        if let Some(row) = list_box_clone.row_at_index(*idx as i32) {
                            list_box_clone.select_row(Some(&row));
                        }
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter | gdk::Key::Tab => {
                    // Aceitar sugestão
                    let idx = *selected_index_clone.borrow();
                    let suggs = suggestions_clone.borrow();
                    if let Some(sugg) = suggs.get(idx) {
                        if let Some(callback) = on_accept_clone.borrow().as_ref() {
                            callback(sugg.word.clone());
                        }
                    }
                    window_clone.close();
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    window_clone.close();
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });

        window.add_controller(key_controller);

        Self {
            window,
            list_box,
            suggestions,
            selected_index,
            on_accept,
        }
    }

    /// Mostrar popup com sugestões próximo à posição do cursor
    pub fn show_at(&self, x: i32, y: i32, suggestions: Vec<Suggestion>) {
        // Limpar lista anterior
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }

        // Guardar sugestões
        *self.suggestions.borrow_mut() = suggestions.clone();
        *self.selected_index.borrow_mut() = 0;

        // Adicionar novas sugestões
        for (i, sugg) in suggestions.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(&sugg.word);
            
            // Mostrar score como subtitle (opcional)
            if sugg.score > 0 {
                let score_text = format!("Score: {}", sugg.score);
                row.set_subtitle(&score_text);
            }

            // Adicionar ícone de sugestão
            let icon = gtk::Image::from_icon_name("document-open-recent-symbolic");
            row.add_prefix(&icon);

            // Marcar primeira como selecionada
            if i == 0 {
                self.list_box.select_row(Some(&row));
            }

            self.list_box.append(&row);
        }

        // Posicionar próximo ao cursor
        self.window.present();
        
        // Ajustar posição (precisaria de layer-shell para posicionamento absoluto)
        // Por enquanto, centraliza na tela
        // TODO: Implementar posicionamento absoluto com gtk4-layer-shell
    }

    /// Esconder popup
    pub fn hide(&self) {
        self.window.hide();
    }

    /// Definir callback quando aceitar sugestão
    pub fn on_accept<F>(&self, callback: F)
    where
        F: Fn(String) + 'static,
    {
        *self.on_accept.borrow_mut() = Some(Box::new(callback));
    }

    /// Aplicar estilos CSS
    pub fn apply_styles(&self) {
        let css = "
        .floating-autocomplete {
            border: 1px solid @borders;
            border-radius: 12px;
            background: @window_bg_color;
            box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
        }

        .autocomplete-list {
            background: transparent;
            border-radius: 12px;
        }

        .autocomplete-list row {
            padding: 8px 12px;
            border-radius: 8px;
            margin: 4px;
        }

        .autocomplete-list row:selected {
            background: @accent_bg_color;
            color: @accent_fg_color;
        }

        .autocomplete-list row:hover:not(:selected) {
            background: alpha(@accent_bg_color, 0.1);
        }
        ";

        let provider = gtk::CssProvider::new();
        provider.load_from_data(css);

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

// Função auxiliar para criar popup flutuante
pub fn create_floating_autocomplete(app: &adw::Application) -> FloatingAutocomplete {
    let popup = FloatingAutocomplete::new(app);
    popup.apply_styles();
    popup
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suggestion_creation() {
        let sugg = Suggestion {
            word: "test".to_string(),
            score: 100,
        };
        assert_eq!(sugg.word, "test");
        assert_eq!(sugg.score, 100);
    }
}
