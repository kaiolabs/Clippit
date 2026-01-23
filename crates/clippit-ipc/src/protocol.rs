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

/// Contexto da aplicação onde a digitação está ocorrendo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub app_name: String,
    pub window_title: String,
    pub input_field_type: Option<String>, // password, email, text, etc.
}

/// Sugestão de autocompletar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub word: String,
    pub score: i64,
    pub source: SuggestionSource,
}

/// Origem da sugestão
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionSource {
    History,       // Do histórico do clipboard
    Frequency,     // Palavras frequentes
    AI,            // Gerado por IA (futuro)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    ShowPopup,
    QueryHistory { limit: usize },           // Existing - kept for compatibility
    QueryHistoryMetadata { limit: usize, offset: usize },   // Get metadata without image data
    SearchHistory { query: String },         // Search in ALL history (no limit)
    GetEntryData { id: i64 },                // Get full data for specific entry
    SelectItem { id: i64 },
    Ping,
    
    // ========== AUTOCOMPLETE GLOBAL ==========
    /// Evento de keystroke do IBus engine
    KeystrokeEvent {
        key: String,
        timestamp: DateTime<Utc>,
        app_context: AppContext,
    },
    /// Solicita sugestões de autocomplete
    RequestAutocompleteSuggestions {
        partial_word: String,
        context: AppContext,
        max_results: usize,
    },
    /// Usuário aceitou uma sugestão
    AcceptSuggestion {
        suggestion: String,
        partial_word: String,
    },
    /// Mostra o popup de autocomplete flutuante
    ShowAutocompletePopup {
        suggestions: Vec<Suggestion>,
        cursor_x: i32,
        cursor_y: i32,
    },
    /// Fecha o popup de autocomplete
    HideAutocompletePopup,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    HistoryResponse { entries: Vec<HistoryEntry> },           // Existing
    HistoryMetadataResponse { entries: Vec<HistoryEntry> },   // Metadata without image data
    SearchHistoryResponse { entries: Vec<HistoryEntry> },     // Search results (no limit)
    EntryDataResponse { entry: HistoryEntry },                // Single entry with full data
    ItemContent { entry: HistoryEntry },
    Error { message: String },
    Pong,
    
    // ========== AUTOCOMPLETE GLOBAL ==========
    /// Resposta com sugestões de autocomplete
    AutocompleteSuggestions {
        suggestions: Vec<Suggestion>,
        query: String,
    },
    /// Confirmação de que sugestão foi aceita
    SuggestionAccepted,
}
