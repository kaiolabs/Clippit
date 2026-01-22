use gtk::prelude::*;
use libadwaita as adw;
use adw::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use clippit_ipc::IpcClient;
use crate::utils::create_thumbnail;
use crate::views::buttons::{add_delete_button, add_copy_button};
use crate::views::image_preview::add_image_hover_preview;
use rust_i18n::t;

/// Creates action rows for history entries (text and images)
/// 
/// Returns: (list_box, entry_map, search_map)
pub fn populate_history_list(
    list_box: &gtk::ListBox,
    window: &adw::ApplicationWindow,
    app: &gtk::Application,
    entry_map: &Rc<RefCell<std::collections::HashMap<i32, i64>>>,
    search_map: &Rc<RefCell<std::collections::HashMap<i32, String>>>,
) {
    // OPTIMIZED: Use metadata query (fast, no image data loaded)
    match IpcClient::query_history_metadata(20) {
        Ok(entries) => {
            eprintln!("✅ Got {} metadata entries from history (images without data)", entries.len());
            for (index, entry) in entries.iter().enumerate() {
                eprintln!("📋 Entry {}: id={}, type={:?}", index, entry.id, entry.content_type);
                let row = adw::ActionRow::new();
                
                // Format content and add prefix based on type
                match entry.content_type {
                    clippit_ipc::ContentType::Text => {
                        create_text_row(&row, entry);
                    }
                    clippit_ipc::ContentType::Image => {
                        create_image_row(&row, entry);
                    }
                }
                
                row.set_subtitle(&entry.timestamp.format("%d/%m/%Y %H:%M:%S").to_string());

                // Store entry ID mapping for keyboard navigation
                let entry_id = entry.id;
                entry_map.borrow_mut().insert(index as i32, entry_id);
                
                // Store search content (title + subtitle) for filtering
                let title_text = row.title().to_string();
                let subtitle_text = row.subtitle().map(|s| s.to_string()).unwrap_or_default();
                let search_content = format!("{} {}", title_text, subtitle_text);
                search_map.borrow_mut().insert(index as i32, search_content);

                // Add delete button
                add_delete_button(&row, entry_id, list_box);
                
                // Add copy button
                add_copy_button(&row, entry_id, window, app);
                
                list_box.append(&row);
            }
            
            // Auto-select first item for keyboard navigation
            if let Some(first_row) = list_box.row_at_index(0) {
                list_box.select_row(Some(&first_row));
                eprintln!("✅ First item auto-selected for keyboard navigation");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to query history: {}", e);
            let empty_row = adw::ActionRow::new();
            empty_row.set_title("Erro ao carregar histórico");
            empty_row.set_subtitle(&format!("Erro: {}", e));
            list_box.append(&empty_row);
        }
    }
}

fn create_text_row(row: &adw::ActionRow, entry: &clippit_ipc::HistoryEntry) {
    // Format text preview - up to 3 lines
    let content = if let Some(text) = &entry.content_text {
        let lines: Vec<&str> = text.lines().take(3).collect();
        let preview = lines.join("\n");
        let char_limit = 240; // ~80 chars per line * 3
        
        if text.len() > char_limit {
            format!("{}...", preview.chars().take(char_limit).collect::<String>())
        } else {
            preview
        }
    } else {
        "Vazio".to_string()
    };
    
    // Escape HTML/XML special characters to avoid markup parsing errors
    let escaped_content = gtk::glib::markup_escape_text(&content);
    row.set_title(&escaped_content);
}

fn create_image_row(row: &adw::ActionRow, entry: &clippit_ipc::HistoryEntry) {
    eprintln!("📸 Processing image entry {}: has_thumbnail={}, has_content={}, has_path={}", 
        entry.id, 
        entry.thumbnail_data.is_some(),
        entry.content_data.is_some(),
        entry.image_path.is_some()
    );
    
    let thumbnail_source = entry.thumbnail_data.as_ref().or(entry.content_data.as_ref());
    
    if let Some(data) = thumbnail_source {
        // Get image dimensions and format title (without emoji)
        let image_info = if let Ok(img) = image::load_from_memory(data) {
            let width = img.width();
            let height = img.height();
            
            // Try to use original content_data size if available, otherwise use thumbnail size
            let (size_kb, size_mb) = if let Some(original_data) = &entry.content_data {
                (original_data.len() / 1024, original_data.len() as f64 / (1024.0 * 1024.0))
            } else {
                // Estimate: thumbnail is ~128x128, original might be 4-16x larger
                let estimated_size = data.len() * 10; // Conservative estimate
                (estimated_size / 1024, estimated_size as f64 / (1024.0 * 1024.0))
            };
            
            let size_str = if size_mb >= 1.0 {
                format!("{:.1} MB", size_mb)
            } else {
                format!("{} KB", size_kb)
            };
            
            format!("{}x{} · {}", width, height, size_str)
        } else {
            format!("{} ({} KB)", t!("popup.image"), data.len() / 1024)
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
                
                // Add CSS for rounded corners
                picture.add_css_class("thumbnail-rounded");
                
                row.add_prefix(&picture);
                eprintln!("✅ Thumbnail created for entry {}", entry.id);
                
                // Add hover preview with larger image (512px)
                add_image_hover_preview(row, data);
            }
            Err(e) => {
                eprintln!("⚠️  Failed to create thumbnail: {}", e);
                // Fallback to icon
                let icon = gtk::Image::from_icon_name("image-x-generic");
                row.add_prefix(&icon);
            }
        }
    } else {
        // No data, use icon
        let icon = gtk::Image::from_icon_name("image-x-generic");
        row.add_prefix(&icon);
    }
}

