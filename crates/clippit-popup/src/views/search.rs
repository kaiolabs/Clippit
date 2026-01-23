use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::models::SearchContentMap;
use crate::utils::{SuggestionEngine, create_thumbnail};
use crate::views::{SuggestionsPopover, buttons::{add_delete_button, add_copy_button}};
use clippit_ipc::IpcClient;
use clippit_core::Config;
use rust_i18n::t;

/// Sets up the REAL DATABASE SEARCH (no limit) with autocomplete
pub fn setup_search_filter(
    list_box: &gtk::ListBox,
    search_entry: &gtk::SearchEntry,
    search_map: &SearchContentMap,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    entry_map: &Rc<RefCell<std::collections::HashMap<i32, i64>>>,
) {
    let list_box_for_search = list_box.clone();
    let search_text_ref: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let search_text_clone = search_text_ref.clone();
    let search_content_clone = search_map.clone();
    
    // Carregar configurações
    let config = Config::load().unwrap_or_default();
    let suggestions_enabled = config.search.enable_suggestions;
    let max_suggestions = config.search.max_suggestions;
    
    // Criar suggestion engine e popover (só se habilitado)
    let suggestion_engine = if suggestions_enabled {
        Some(Rc::new(RefCell::new(SuggestionEngine::new())))
    } else {
        None
    };
    
    let suggestions_popover = if suggestions_enabled {
        Some(Rc::new(RefCell::new(SuggestionsPopover::new(search_entry))))
    } else {
        None
    };
    
    // Popular histórico no engine (só se habilitado)
    if let Some(ref engine) = suggestion_engine {
        match IpcClient::query_history_metadata(100) {
            Ok(entries) => {
                engine.borrow_mut().update_history_words(&entries);
                eprintln!("✅ {} entradas carregadas para sugestões", entries.len());
            }
            Err(e) => eprintln!("⚠️  Erro ao carregar histórico: {}", e),
        }
    }
    
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
    let suggestion_engine_for_changed = suggestion_engine.clone();
    let suggestions_popover_for_changed = suggestions_popover.clone();
    
    search_entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        
        // Filtro existente (mantém)
        *search_text_ref.borrow_mut() = text.to_lowercase();
        list_box_for_search.invalidate_filter();
        
        // NOVO: Autocompletar (só se habilitado)
        if let (Some(ref engine), Some(ref popover)) = (&suggestion_engine_for_changed, &suggestions_popover_for_changed) {
            if let Some(current_word) = extract_current_word(&text, entry.position()) {
                if current_word.len() >= 2 {
                    let suggestions = engine.borrow()
                        .get_suggestions(&current_word, max_suggestions);
                    
                    if !suggestions.is_empty() {
                        popover.borrow_mut().update_suggestions(suggestions);
                        popover.borrow().show();
                    } else {
                        popover.borrow().hide();
                    }
                } else {
                    popover.borrow().hide();
                }
            } else {
                popover.borrow().hide();
            }
        }
    });
    
    // Adicionar EventController para Tab e navegação (só se sugestões habilitadas)
    if let Some(suggestions_popover_for_keys) = suggestions_popover {
        let key_controller = gtk::EventControllerKey::new();
        key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
        let search_entry_for_keys = search_entry.clone();
        
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let popover_visible = suggestions_popover_for_keys.borrow().is_visible();
            
            match key {
                gtk::gdk::Key::Tab => {
                    if popover_visible {
                        let word_to_complete = suggestions_popover_for_keys.borrow()
                            .get_selected_suggestion()
                            .map(|s| s.word.clone());
                        
                        if let Some(word) = word_to_complete {
                            eprintln!("🔍 Completing with: {}", word);
                            complete_current_word(&search_entry_for_keys, &word);
                            suggestions_popover_for_keys.borrow().hide();
                            return gtk::glib::Propagation::Stop;
                        }
                    }
                }
                gtk::gdk::Key::Up => {
                    if popover_visible {
                        suggestions_popover_for_keys.borrow_mut().navigate_up();
                        return gtk::glib::Propagation::Stop;
                    }
                }
                gtk::gdk::Key::Down => {
                    if popover_visible {
                        suggestions_popover_for_keys.borrow_mut().navigate_down();
                        return gtk::glib::Propagation::Stop;
                    }
                }
                gtk::gdk::Key::Escape => {
                    if popover_visible {
                        suggestions_popover_for_keys.borrow().hide();
                        return gtk::glib::Propagation::Stop;
                    }
                }
                _ => {}
            }
            gtk::glib::Propagation::Proceed
        });
        
        search_entry.add_controller(key_controller);
        eprintln!("✅ Search filter with autocomplete setup complete!");
    } else {
        eprintln!("✅ Search filter setup complete (suggestions disabled)!");
    }
}

/// Extrair a palavra sendo digitada na posição do cursor
fn extract_current_word(text: &str, cursor_pos: i32) -> Option<String> {
    let pos = cursor_pos as usize;
    if pos > text.len() {
        return None;
    }
    
    // Encontrar início da palavra (voltar até espaço/início)
    let start = text[..pos]
        .rfind(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|i| i + 1)
        .unwrap_or(0);
    
    // Encontrar fim da palavra (avançar até espaço/fim)
    let end = text[pos..]
        .find(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
        .map(|i| pos + i)
        .unwrap_or(text.len());
    
    let word = &text[start..end];
    if word.is_empty() {
        None
    } else {
        Some(word.to_string())
    }
}

/// Substituir palavra parcial pela sugestão completa
fn complete_current_word(entry: &gtk::SearchEntry, suggestion: &str) {
    let text = entry.text().to_string();
    let cursor_pos = entry.position() as usize;
    
    if let Some(current_word) = extract_current_word(&text, cursor_pos as i32) {
        // Encontrar posição da palavra atual
        if let Some(word_start) = text[..cursor_pos].rfind(&current_word) {
            let word_end = word_start + current_word.len();
            
            // Substituir palavra
            let new_text = format!(
                "{}{}{}",
                &text[..word_start],
                suggestion,
                &text[word_end..]
            );
            
            entry.set_text(&new_text);
            entry.set_position((word_start + suggestion.len()) as i32);
        }
    }
}
