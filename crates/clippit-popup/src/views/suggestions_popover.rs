use gtk::prelude::*;
use gtk::{Popover, ListBox};
use crate::utils::suggestions::{Suggestion, SuggestionSource};

pub struct SuggestionsPopover {
    popover: Popover,
    list_box: ListBox,
    suggestions: Vec<Suggestion>,
    selected_index: i32,
}

impl SuggestionsPopover {
    pub fn new(parent: &gtk::SearchEntry) -> Self {
        let popover = Popover::new();
        popover.set_parent(parent);
        popover.set_position(gtk::PositionType::Bottom);
        popover.set_autohide(false);
        popover.add_css_class("suggestions-popover");
        
        let list_box = ListBox::new();
        list_box.set_selection_mode(gtk::SelectionMode::Single);
        list_box.add_css_class("suggestions-list");
        
        let scrolled = gtk::ScrolledWindow::new();
        scrolled.set_child(Some(&list_box));
        scrolled.set_max_content_height(250);
        scrolled.set_propagate_natural_height(true);
        
        popover.set_child(Some(&scrolled));
        
        Self {
            popover,
            list_box,
            suggestions: Vec::new(),
            selected_index: 0,
        }
    }
    
    pub fn update_suggestions(&mut self, suggestions: Vec<Suggestion>) {
        // Limpar itens antigos
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        
        // Adicionar novas sugestões
        for suggestion in suggestions.iter() {
            let row = gtk::ListBoxRow::new();
            row.add_css_class("suggestion-row");
            
            let box_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            box_row.set_margin_start(8);
            box_row.set_margin_end(8);
            box_row.set_margin_top(6);
            box_row.set_margin_bottom(6);
            
            // Ícone baseado na fonte
            let icon = match suggestion.source {
                SuggestionSource::History => {
                    let icon = gtk::Image::from_icon_name("document-open-recent");
                    icon.add_css_class("suggestion-history-icon");
                    icon
                }
            };
            box_row.append(&icon);
            
            // Palavra
            let label = gtk::Label::new(Some(&suggestion.word));
            label.add_css_class("suggestion-word");
            label.set_halign(gtk::Align::Start);
            box_row.append(&label);
            
            row.set_child(Some(&box_row));
            self.list_box.append(&row);
        }
        
        self.suggestions = suggestions;
        self.selected_index = 0;
        
        // Selecionar primeiro item
        if let Some(first_row) = self.list_box.row_at_index(0) {
            self.list_box.select_row(Some(&first_row));
        }
    }
    
    pub fn show(&self) {
        self.popover.popup();
    }
    
    pub fn hide(&self) {
        self.popover.popdown();
    }
    
    pub fn navigate_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            if let Some(row) = self.list_box.row_at_index(self.selected_index) {
                self.list_box.select_row(Some(&row));
            }
        }
    }
    
    pub fn navigate_down(&mut self) {
        let max = self.suggestions.len() as i32 - 1;
        if self.selected_index < max {
            self.selected_index += 1;
            if let Some(row) = self.list_box.row_at_index(self.selected_index) {
                self.list_box.select_row(Some(&row));
            }
        }
    }
    
    pub fn get_selected_suggestion(&self) -> Option<&Suggestion> {
        self.suggestions.get(self.selected_index as usize)
    }
}
