use anyhow::Result;
use clippit_ipc::{ContentType, IpcClient};
use std::io::{self, Write};
use tracing::{error, info};

pub fn run_ui() -> Result<()> {
    info!("Clippit UI started");

    // Check if daemon is running
    match IpcClient::ping() {
        Ok(_) => {
            info!("Connected to daemon");
        }
        Err(e) => {
            eprintln!("Error: Daemon not running. Please start clippit-daemon first.");
            eprintln!("Details: {}", e);
            return Err(e);
        }
    }

    // Query history
    match IpcClient::query_history(20) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("\nClipboard history is empty.");
                return Ok(());
            }

            println!("\n╔══════════════════════════════════════════════════════════════╗");
            println!("║                    CLIPPIT - Clipboard History                ║");
            println!("╚══════════════════════════════════════════════════════════════╝\n");

            for (idx, entry) in entries.iter().enumerate() {
                let preview = match &entry.content_type {
                    ContentType::Text => {
                        if let Some(text) = &entry.content_text {
                            let preview_text = text
                                .lines()
                                .next()
                                .unwrap_or("")
                                .chars()
                                .take(60)
                                .collect::<String>();

                            if text.len() > 60 {
                                format!("{}...", preview_text)
                            } else {
                                preview_text
                            }
                        } else {
                            "[Empty]".to_string()
                        }
                    }
                    ContentType::Image => {
                        if let Some(data) = &entry.content_data {
                            format!("[Image - {} bytes]", data.len())
                        } else {
                            "[Empty Image]".to_string()
                        }
                    }
                };

                let time_str = entry.timestamp.format("%H:%M:%S");
                println!("  {}. {} │ {}", idx + 1, time_str, preview);
            }

            println!("\n─────────────────────────────────────────────────────────────");
            print!("\nSelect item (1-{}) or 'q' to quit: ", entries.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input == "q" || input.is_empty() {
                return Ok(());
            }

            if let Ok(selection) = input.parse::<usize>() {
                if selection > 0 && selection <= entries.len() {
                    let entry = &entries[selection - 1];

                    match IpcClient::select_item(entry.id) {
                        Ok(_) => {
                            println!("\n✓ Item copied to clipboard!");
                        }
                        Err(e) => {
                            error!("Failed to select item: {}", e);
                            eprintln!("Error: Failed to copy item to clipboard");
                        }
                    }
                } else {
                    eprintln!("Invalid selection");
                }
            } else {
                eprintln!("Invalid input");
            }
        }
        Err(e) => {
            error!("Failed to query history: {}", e);
            eprintln!("Error: Failed to get clipboard history");
            return Err(e);
        }
    }

    Ok(())
}
