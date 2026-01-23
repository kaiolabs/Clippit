use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

use crate::models::SearchContentMap;
use crate::utils::{SuggestionEngine, create_thumbnail};
use crate::views::buttons::{add_delete_button, add_copy_button};
use crate::views::SuggestionsPopover;
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
    let window_for_search = window.clone();
    let app_for_search = app.clone();
    let entry_map_for_search = entry_map.clone();
    let search_map_for_search = search_map.clone();
    
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
        eprintln!("📚 Carregando palavras do histórico para sugestões...");
        match IpcClient::query_history_metadata(100) {
            Ok(entries) => {
                eprintln!("📥 Recebidas {} entradas do histórico", entries.len());
                engine.borrow_mut().update_history_words(&entries);
                eprintln!("✅ {} entradas carregadas para sugestões", entries.len());
            }
            Err(e) => {
                eprintln!("❌ ERRO ao carregar histórico para sugestões: {}", e);
                eprintln!("❌ Detalhes: {:?}", e);
            }
        }
    }
    
    // NO filter_func needed - we'll reload the list with search results from DB
    
    // Função para executar busca (pode ser chamada de vários lugares)
    let perform_search = {
        let list_box_clone = list_box_for_search.clone();
        let window_clone = window_for_search.clone();
        let app_clone = app_for_search.clone();
        let entry_map_clone = entry_map_for_search.clone();
        let search_map_clone = search_map_for_search.clone();
        
        Rc::new(move |query: String| {
            if query.trim().is_empty() {
                eprintln!("🔍 Busca vazia - mantendo lista atual");
                return;
            }
            
            // Buscar TUDO no banco de dados
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            eprintln!("🔍 BUSCANDO NO BANCO: '{}'", query);
            eprintln!("🔍 Query length: {} chars", query.len());
            eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            match IpcClient::search_history(query.clone()) {
                Ok(entries) => {
                    eprintln!("✅ BUSCA RETORNOU {} RESULTADOS", entries.len());
                    
                    if entries.is_empty() {
                        eprintln!("⚠️  NENHUM RESULTADO ENCONTRADO PARA: '{}'", query);
                    } else {
                        eprintln!("📋 Vou adicionar {} resultados na lista...", entries.len());
                    }
                    
                    // Limpar lista atual
                    eprintln!("🗑️  Limpando lista atual...");
                    let mut removed_count = 0;
                    while let Some(child) = list_box_clone.first_child() {
                        list_box_clone.remove(&child);
                        removed_count += 1;
                    }
                    eprintln!("🗑️  Removidos {} itens", removed_count);
                    
                    // Limpar mapas
                    entry_map_clone.borrow_mut().clear();
                    search_map_clone.borrow_mut().clear();
                    eprintln!("🗑️  Mapas limpos");
                    
                    // Repovoar com resultados da busca
                    eprintln!("➕ Começando a adicionar resultados...");
                    for (index, hist_entry) in entries.iter().enumerate() {
                        eprintln!("  ➕ Adicionando item {}/{}: id={}", index+1, entries.len(), hist_entry.id);
                        let row = adw::ActionRow::new();
                        row.set_activatable(true);  // 🔥 Tornar a linha clicável
                        
                        // Format based on type
                        match hist_entry.content_type {
                            clippit_ipc::ContentType::Text => {
                                // Format text preview
                                let content = if let Some(text) = &hist_entry.content_text {
                                    let lines: Vec<&str> = text.lines().take(3).collect();
                                    let preview = lines.join("\n");
                                    let char_limit = 240;
                                    
                                    if text.len() > char_limit {
                                        format!("{}...", preview.chars().take(char_limit).collect::<String>())
                                    } else {
                                        preview
                                    }
                                } else {
                                    "Vazio".to_string()
                                };
                                
                                let escaped_content = gtk::glib::markup_escape_text(&content);
                                row.set_title(&escaped_content);
                            }
                            clippit_ipc::ContentType::Image => {
                                let thumbnail_source = hist_entry.thumbnail_data.as_ref().or(hist_entry.content_data.as_ref());
                                
                                if let Some(data) = thumbnail_source {
                                    let image_info = if let Ok(img) = image::load_from_memory(data) {
                                        let width = img.width();
                                        let height = img.height();
                                        format!("{}x{}", width, height)
                                    } else {
                                        format!("{}", t!("popup.image"))
                                    };
                                    
                                    row.set_title(&image_info);
                                    
                                    match create_thumbnail(data, 128) {
                                        Ok(pixbuf) => {
                                            let picture = gtk::Image::from_pixbuf(Some(&pixbuf));
                                            picture.set_pixel_size(128);
                                            picture.set_margin_start(4);
                                            picture.set_margin_end(4);
                                            picture.set_margin_top(4);
                                            picture.set_margin_bottom(4);
                                            picture.add_css_class("thumbnail-rounded");
                                            row.add_prefix(&picture);
                                        }
                                        Err(_) => {
                                            let icon = gtk::Image::from_icon_name("image-x-generic");
                                            row.add_prefix(&icon);
                                        }
                                    }
                                }
                            }
                        }
                        
                        row.set_subtitle(&hist_entry.timestamp.format("%d/%m/%Y %H:%M:%S").to_string());
                        
                        // Store entry ID and search content
                        entry_map_clone.borrow_mut().insert(index as i32, hist_entry.id);
                        let title_text = row.title().to_string();
                        let subtitle_text = row.subtitle().map(|s| s.to_string()).unwrap_or_default();
                        search_map_clone.borrow_mut().insert(index as i32, format!("{} {}", title_text, subtitle_text));
                        
                        // Add buttons
                        add_delete_button(&row, hist_entry.id, &list_box_clone);
                        add_copy_button(&row, hist_entry.id, &window_clone, &app_clone);
                        
                        list_box_clone.append(&row);
                        eprintln!("  ✅ Item {} adicionado à lista", index+1);
                    }
                    
                    eprintln!("🎉 TODOS OS {} RESULTADOS FORAM ADICIONADOS!", entries.len());
                    
                    // Auto-select first result
                    if let Some(first_row) = list_box_clone.row_at_index(0) {
                        list_box_clone.select_row(Some(&first_row));
                        eprintln!("✅ Primeiro item selecionado");
                    } else {
                        eprintln!("⚠️  Nenhum item para selecionar (lista vazia?)");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Erro na busca: {}", e);
                }
            }
        })
    };
    
    // Conectar mudanças no campo de busca
    let suggestion_engine_for_changed = suggestion_engine.clone();
    let suggestions_popover_for_changed = suggestions_popover.clone();
    let perform_search_for_changed = perform_search.clone();
    
    search_entry.connect_changed(move |entry| {
        let text = entry.text().to_string();
        
        // 🔍 BUSCA EM TEMPO REAL
        perform_search_for_changed(text.clone());
        
        // Autocompletar (só se habilitado)
        if let (Some(ref engine), Some(ref popover)) = (&suggestion_engine_for_changed, &suggestions_popover_for_changed) {
            if let Some(current_word) = extract_current_word(&text, entry.position()) {
                if current_word.len() >= 2 {
                    eprintln!("💡 Buscando sugestões para palavra: '{}'", current_word);
                    let suggestions = engine.borrow()
                        .get_suggestions(&current_word, max_suggestions as usize);
                    
                    eprintln!("💡 Encontradas {} sugestões", suggestions.len());
                    for (i, sug) in suggestions.iter().enumerate() {
                        eprintln!("   {}. '{}' (score: {})", i+1, sug.word, sug.score);
                    }
                    
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
        let perform_search_for_keys = perform_search.clone();
        let list_box_for_keys = list_box.clone();
        let entry_map_for_keys = entry_map.clone();
        let window_for_keys = window.clone();
        let app_for_keys = app.clone();
        
        key_controller.connect_key_pressed(move |_, key, _, _| {
            let popover_visible = suggestions_popover_for_keys.borrow().is_visible();
            
            match key {
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                    eprintln!("🔑 Enter pressed in search_entry");
                    eprintln!("   Popover visible: {}", popover_visible);
                    eprintln!("   Search text: '{}'", search_entry_for_keys.text());
                    
                    if popover_visible {
                        // Se popover está visível, deixa o comportamento padrão
                        eprintln!("   → Popover visible, proceeding");
                        return gtk::glib::Propagation::Proceed;
                    }
                    
                    // Se popover NÃO está visível E search está vazio, copia o item selecionado
                    let text = search_entry_for_keys.text();
                    if text.is_empty() {
                        eprintln!("   → Empty search, copying selected item directly");
                        
                        // Copiar item selecionado diretamente
                        if let Some(selected_row) = list_box_for_keys.selected_row() {
                            let row_index = selected_row.index();
                            eprintln!("   → Selected row index: {}", row_index);
                            
                            if let Some(&entry_id) = entry_map_for_keys.borrow().get(&row_index) {
                                eprintln!("   → Copying entry ID: {}", entry_id);
                                
                                use crate::controllers::copy_to_clipboard;
                                copy_to_clipboard(entry_id);
                                
                                eprintln!("   → Closing window after copy");
                                window_for_keys.close();
                                app_for_keys.quit();
                            } else {
                                eprintln!("   ⚠️  No entry_id found for row index {}", row_index);
                            }
                        } else {
                            eprintln!("   ⚠️  No row selected");
                        }
                        
                        return gtk::glib::Propagation::Stop;
                    } else {
                        eprintln!("   → Search text present, proceeding with search");
                    }
                    
                    return gtk::glib::Propagation::Proceed;
                }
                gtk::gdk::Key::Tab => {
                    if popover_visible {
                        let word_to_complete = suggestions_popover_for_keys.borrow()
                            .get_selected_suggestion()
                            .map(|s| s.word.clone());
                        
                        if let Some(word) = word_to_complete {
                            eprintln!("🔍 Completing with: {}", word);
                            complete_current_word(&search_entry_for_keys, &word);
                            suggestions_popover_for_keys.borrow().hide();
                            
                            // 🔍 BUSCAR EM TEMPO REAL após completar
                            let new_text = search_entry_for_keys.text().to_string();
                            perform_search_for_keys(new_text);
                            
                            return gtk::glib::Propagation::Stop;
                        }
                    }
                }
                gtk::gdk::Key::Up => {
                    if popover_visible {
                        suggestions_popover_for_keys.borrow_mut().navigate_up();
                        
                        // 🔍 BUSCAR EM TEMPO REAL ao navegar nas sugestões
                        let selected = suggestions_popover_for_keys.borrow()
                            .get_selected_suggestion()
                            .map(|s| s.word.clone());
                        
                        if let Some(word) = selected {
                            perform_search_for_keys(word);
                        }
                        
                        return gtk::glib::Propagation::Stop;
                    }
                }
                gtk::gdk::Key::Down => {
                    if popover_visible {
                        suggestions_popover_for_keys.borrow_mut().navigate_down();
                        
                        // 🔍 BUSCAR EM TEMPO REAL ao navegar nas sugestões
                        let selected = suggestions_popover_for_keys.borrow()
                            .get_selected_suggestion()
                            .map(|s| s.word.clone());
                        
                        if let Some(word) = selected {
                            perform_search_for_keys(word);
                        }
                        
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
        eprintln!("✅ Search filter with REAL-TIME database search + autocomplete setup complete!");
    } else {
        eprintln!("✅ Search filter with REAL-TIME database search setup complete (suggestions disabled)!");
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
