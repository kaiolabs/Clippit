use anyhow::{Context, Result};
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::Stream;
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use std::io::{BufRead, BufReader, Write};

use crate::protocol::{IpcMessage, IpcResponse, SOCKET_PATH};

pub struct IpcClient;

impl IpcClient {
    pub fn send_message(message: IpcMessage) -> Result<IpcResponse> {
        let name = SOCKET_PATH.to_ns_name::<GenericNamespaced>()?;
        let mut stream = LocalSocketStream::connect(name)
            .context("Failed to connect to daemon. Is clippit-daemon running?")?;

        let message_json = serde_json::to_string(&message)?;
        writeln!(stream, "{}", message_json)?;
        stream.flush()?;

        let mut reader = BufReader::new(&stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line)?;

        let response: IpcResponse =
            serde_json::from_str(&response_line).context("Failed to deserialize response")?;

        Ok(response)
    }

    pub fn ping() -> Result<()> {
        match Self::send_message(IpcMessage::Ping)? {
            IpcResponse::Pong => Ok(()),
            _ => Err(anyhow::anyhow!("Unexpected response to ping")),
        }
    }

    pub fn query_history(limit: usize) -> Result<Vec<crate::protocol::HistoryEntry>> {
        match Self::send_message(IpcMessage::QueryHistory { limit })? {
            IpcResponse::HistoryResponse { entries } => Ok(entries),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Query history metadata without loading image data (optimized for listing)
    pub fn query_history_metadata(limit: usize) -> Result<Vec<crate::protocol::HistoryEntry>> {
        match Self::send_message(IpcMessage::QueryHistoryMetadata { limit, offset: 0 })? {
            IpcResponse::HistoryMetadataResponse { entries } => Ok(entries),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Query history metadata with offset (for infinite scroll)
    pub fn query_history_metadata_with_offset(limit: usize, offset: usize) -> Result<Vec<crate::protocol::HistoryEntry>> {
        match Self::send_message(IpcMessage::QueryHistoryMetadata { limit, offset })? {
            IpcResponse::HistoryMetadataResponse { entries} => Ok(entries),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Search history (NO LIMIT - searches ALL entries in database)
    pub fn search_history(query: String) -> Result<Vec<crate::protocol::HistoryEntry>> {
        match Self::send_message(IpcMessage::SearchHistory { query })? {
            IpcResponse::SearchHistoryResponse { entries } => Ok(entries),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Get full data for a specific entry by ID (loads image data on-demand)
    pub fn get_entry_data(id: i64) -> Result<crate::protocol::HistoryEntry> {
        match Self::send_message(IpcMessage::GetEntryData { id })? {
            IpcResponse::EntryDataResponse { entry } => Ok(entry),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    pub fn select_item(id: i64) -> Result<()> {
        match Self::send_message(IpcMessage::SelectItem { id })? {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    // ========== AUTOCOMPLETE GLOBAL METHODS ==========

    /// Request autocomplete suggestions
    pub fn request_autocomplete_suggestions(
        partial_word: String,
        context: crate::protocol::AppContext,
        max_results: usize,
    ) -> Result<Vec<crate::protocol::Suggestion>> {
        match Self::send_message(IpcMessage::RequestAutocompleteSuggestions {
            partial_word,
            context,
            max_results,
        })? {
            IpcResponse::AutocompleteSuggestions { suggestions, .. } => Ok(suggestions),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }

    /// Accept a suggestion (for tracking/learning)
    pub fn accept_suggestion(suggestion: String, partial_word: String) -> Result<()> {
        match Self::send_message(IpcMessage::AcceptSuggestion {
            suggestion,
            partial_word,
        })? {
            IpcResponse::SuggestionAccepted => Ok(()),
            IpcResponse::Ok => Ok(()),
            IpcResponse::Error { message } => Err(anyhow::anyhow!("Server error: {}", message)),
            _ => Err(anyhow::anyhow!("Unexpected response")),
        }
    }
}
