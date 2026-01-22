use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const SOCKET_PATH: &str = "/tmp/clippit.sock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: Option<String>,
    pub content_data: Option<Vec<u8>>, // Backwards compatibility
    pub image_path: Option<String>, // Path to image file on disk
    pub thumbnail_data: Option<Vec<u8>>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    ShowPopup,
    QueryHistory { limit: usize },           // Existing - kept for compatibility
    QueryHistoryMetadata { limit: usize },   // Get metadata without image data
    QueryHistoryMetadataWithOffset { limit: usize, offset: usize },  // For infinite scroll
    GetEntryData { id: i64 },                // Get full data for specific entry
    SelectItem { id: i64 },
    Ping,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    HistoryResponse { entries: Vec<HistoryEntry> },           // Existing
    HistoryMetadataResponse { entries: Vec<HistoryEntry> },   // NEW: Metadata without image data
    EntryDataResponse { entry: HistoryEntry },                // NEW: Single entry with full data
    ItemContent { entry: HistoryEntry },
    Error { message: String },
    Pong,
}
