mod hotkey;
mod monitor;

use anyhow::Result;
use clippit_core::HistoryManager;
use clippit_ipc::{ContentType, HistoryEntry, IpcMessage, IpcResponse, IpcServer};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::task;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Check for --version flag
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "--version" || args[1] == "-v") {
        println!("clippit-daemon {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("clippit_daemon=info")
        .init();

    info!("Starting Clippit daemon v{} (Wayland-native)...", env!("CARGO_PKG_VERSION"));

    // Initialize history manager
    let db_path = get_db_path();
    let history_manager = Arc::new(Mutex::new(HistoryManager::new(db_path, 100)?));

    // Clone for clipboard monitor
    let history_clone = Arc::clone(&history_manager);

    // Start clipboard monitor
    let monitor_handle = task::spawn(async move {
        if let Err(e) = monitor::start_monitor(history_clone).await {
            error!("Clipboard monitor error: {}", e);
        }
    });

    // Clone for hotkey handler
    let history_clone = Arc::clone(&history_manager);

    // Start hotkey handler
    let hotkey_handle = task::spawn(async move {
        if let Err(e) = hotkey::start_hotkey_handler(history_clone).await {
            error!("Hotkey handler error: {}", e);
        }
    });

    // Clone for IPC server
    let history_clone = Arc::clone(&history_manager);

    // Start IPC server
    let server = IpcServer::new(Box::new(move |message| {
        handle_ipc_message(message, &history_clone)
    }));

    info!("All components started successfully");

    // Run IPC server (blocks)
    if let Err(e) = server.start().await {
        error!("IPC server error: {}", e);
    }

    // Wait for other tasks
    let _ = tokio::join!(monitor_handle, hotkey_handle);

    Ok(())
}

fn handle_ipc_message(
    message: IpcMessage,
    history_manager: &Arc<Mutex<HistoryManager>>,
) -> IpcResponse {
    match message {
        IpcMessage::Ping => IpcResponse::Pong,

        IpcMessage::QueryHistory { limit } => {
            let manager = history_manager.lock().unwrap();
            match manager.get_recent(limit) {
                Ok(entries) => {
                    let ipc_entries: Vec<HistoryEntry> = entries
                        .into_iter()
                        .map(|e| HistoryEntry {
                            id: e.id,
                            content_type: match e.content_type {
                                clippit_core::ContentType::Text => ContentType::Text,
                                clippit_core::ContentType::Image => ContentType::Image,
                            },
                            content_text: e.content_text,
                            content_data: e.content_data,
                            image_path: e.image_path,
                            thumbnail_data: e.thumbnail_data,
                            timestamp: e.timestamp,
                        })
                        .collect();
                    IpcResponse::HistoryResponse {
                        entries: ipc_entries,
                    }
                }
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to get history: {}", e),
                },
            }
        }

        IpcMessage::QueryHistoryMetadata { limit, offset } => {
            let manager = history_manager.lock().unwrap();
            match manager.get_recent_metadata_with_offset(limit, offset) {
                Ok(entries) => {
                    let ipc_entries: Vec<HistoryEntry> = entries
                        .into_iter()
                        .map(|e| HistoryEntry {
                            id: e.id,
                            content_type: match e.content_type {
                                clippit_core::ContentType::Text => ContentType::Text,
                                clippit_core::ContentType::Image => ContentType::Image,
                            },
                            content_text: e.content_text,
                            content_data: e.content_data, // Already None for images from get_recent_metadata
                            image_path: e.image_path, // Include image path
                            thumbnail_data: e.thumbnail_data, // Include thumbnail data
                            timestamp: e.timestamp,
                        })
                        .collect();
                    info!("Returned {} metadata entries (images without data)", ipc_entries.len());
                    IpcResponse::HistoryMetadataResponse {
                        entries: ipc_entries,
                    }
                }
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to get history metadata: {}", e),
                },
            }
        }

        IpcMessage::SearchHistory { query } => {
            let manager = history_manager.lock().unwrap();
            match manager.search(&query) {
                Ok(entries) => {
                    let ipc_entries: Vec<HistoryEntry> = entries
                        .into_iter()
                        .map(|e| HistoryEntry {
                            id: e.id,
                            content_type: match e.content_type {
                                clippit_core::ContentType::Text => ContentType::Text,
                                clippit_core::ContentType::Image => ContentType::Image,
                            },
                            content_text: e.content_text,
                            content_data: e.content_data,
                            image_path: e.image_path,
                            thumbnail_data: e.thumbnail_data,
                            timestamp: e.timestamp,
                        })
                        .collect();
                    info!("Search '{}' returned {} results (NO LIMIT)", query, ipc_entries.len());
                    IpcResponse::SearchHistoryResponse {
                        entries: ipc_entries,
                    }
                }
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to search history: {}", e),
                },
            }
        }

        IpcMessage::GetEntryData { id } => {
            let manager = history_manager.lock().unwrap();
            match manager.get_by_id(id) {
                Ok(Some(mut entry)) => {
                    info!("📦 Preparing response for entry {}", id);
                    info!("   Content type: {:?}", entry.content_type);
                    
                    // If it's an image with a file path, read from disk
                    if matches!(entry.content_type, clippit_core::ContentType::Image) {
                        if let Some(ref path) = entry.image_path {
                            info!("📂 Reading image from file: {}", path);
                            match std::fs::read(path) {
                                Ok(data) => {
                                    info!("✅ Read {} bytes ({:.2} MB) from disk", data.len(), data.len() as f64 / (1024.0 * 1024.0));
                                    entry.content_data = Some(data);
                                }
                                Err(e) => {
                                    error!("❌ Failed to read image file: {}", e);
                                    // Return error if we can't read the file
                                    return IpcResponse::Error {
                                        message: format!("Failed to read image file: {}", e),
                                    };
                                }
                            }
                        }
                    }
                    
                    if let Some(ref data) = entry.content_data {
                        info!("   Data size: {} bytes ({:.2} MB)", data.len(), data.len() as f64 / (1024.0 * 1024.0));
                    }
                    
                    let ipc_entry = HistoryEntry {
                        id: entry.id,
                        content_type: match entry.content_type {
                            clippit_core::ContentType::Text => ContentType::Text,
                            clippit_core::ContentType::Image => ContentType::Image,
                        },
                        content_text: entry.content_text,
                        content_data: entry.content_data, // Full data included (backwards compat)
                        image_path: entry.image_path, // Include image path
                        thumbnail_data: entry.thumbnail_data, // Include thumbnail data
                        timestamp: entry.timestamp,
                    };
                    info!("✅ Returned full data for entry {}", id);
                    IpcResponse::EntryDataResponse { entry: ipc_entry }
                }
                Ok(None) => IpcResponse::Error {
                    message: format!("Entry with id {} not found", id),
                },
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to get entry: {}", e),
                },
            }
        }

        IpcMessage::SelectItem { id } => {
            let manager = history_manager.lock().unwrap();
            match manager.get_by_id(id) {
                Ok(Some(_entry)) => {
                    // Clipboard copy is handled by popup with arboard
                    info!("Item {} selected (clipboard copy handled by popup)", id);
                    IpcResponse::Ok
                }
                Ok(None) => IpcResponse::Error {
                    message: format!("Entry with id {} not found", id),
                },
                Err(e) => IpcResponse::Error {
                    message: format!("Failed to get entry: {}", e),
                },
            }
        }

        IpcMessage::ShowPopup => {
            // This is handled by the UI, daemon just acknowledges
            IpcResponse::Ok
        }

        // ========== AUTOCOMPLETE GLOBAL MESSAGES ==========
        IpcMessage::KeystrokeEvent { .. } => {
            // TODO: Processar keystroke event
            IpcResponse::Ok
        }

        IpcMessage::RequestAutocompleteSuggestions {
            partial_word,
            context,
            max_results,
        } => {
            // TODO: Usar typing_monitor para gerar sugestões
            info!(
                "Autocomplete request: '{}' in {} (max: {})",
                partial_word, context.app_name, max_results
            );
            IpcResponse::AutocompleteSuggestions {
                suggestions: vec![],
                query: partial_word,
            }
        }

        IpcMessage::AcceptSuggestion {
            suggestion,
            partial_word,
        } => {
            info!(
                "Suggestion accepted: '{}' (was: '{}')",
                suggestion, partial_word
            );
            IpcResponse::SuggestionAccepted
        }

        IpcMessage::ShowAutocompletePopup { .. } => IpcResponse::Ok,
        IpcMessage::HideAutocompletePopup => IpcResponse::Ok,
    }
}

fn get_db_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("clippit");
    std::fs::create_dir_all(&path).ok();
    path.push("history.db");
    path
}

