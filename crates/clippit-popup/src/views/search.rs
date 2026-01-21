use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::models::SearchContentMap;

/// Sets up the search filter on the list box
pub fn setup_search_filter(
    list_box: &gtk::ListBox,
    search_entry: &gtk::SearchEntry,
    search_map: &SearchContentMap,
) {
    let list_box_for_search = list_box.clone();
    let search_text_ref: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let search_text_clone = search_text_ref.clone();
    let search_content_clone = search_map.clone();
    
    list_box.set_filter_func(move |row: &gtk::ListBoxRow| {
        let search = search_text_clone.borrow();
        
        // Se busca vazia, mostra tudo
        if search.is_empty() {
            return true;
        }
        
        let row_index = row.index();
        let search_lower = search.to_lowercase();
        
        // Tentar buscar no mapa de conteúdo
        if let Some(content) = search_content_clone.borrow().get(&row_index) {
            let matches = content.to_lowercase().contains(&search_lower);
            return matches;
        }
        
        // Fallback: tentar extrair do child
        if let Some(child) = row.child() {
            if let Ok(action_row) = child.downcast::<adw::ActionRow>() {
                let title = action_row.title().to_lowercase();
                let subtitle = action_row.subtitle()
                    .map(|s| s.as_str().to_lowercase())
                    .unwrap_or_default();
                
                let content = format!("{} {}", title, subtitle);
                return content.contains(&search_lower);
            }
        }
        
        // Default: mostra o item
        true
    });
    
    // Conectar mudanças no campo de busca
    search_entry.connect_changed(move |entry| {
        let search_text = entry.text().to_lowercase();
        eprintln!("🔍 Filtering: '{}'", search_text);
        
        *search_text_ref.borrow_mut() = search_text.clone();
        list_box_for_search.invalidate_filter();
        
        eprintln!("✅ Filter applied!");
    });
}
