use clippit_ipc::{HistoryEntry, IpcClient};

pub struct HistoryModel {
    entries: Vec<HistoryEntry>,
}

impl HistoryModel {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn load_history(&mut self, limit: i32) {
        if let Ok(entries) = IpcClient::query_history(limit as usize) {
            self.entries = entries;
        }
    }

    pub fn select_item(&self, id: i64) -> bool {
        IpcClient::select_item(id).is_ok()
    }

    pub fn get_item_count(&self) -> i32 {
        self.entries.len() as i32
    }

    pub fn get_item_content(&self, index: i32) -> String {
        if let Some(entry) = self.entries.get(index as usize) {
            if let Some(text) = &entry.content_text {
                let preview: String = text
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(100)
                    .collect();
                return preview;
            } else if let Some(data) = &entry.content_data {
                return format!("[Image - {} bytes]", data.len());
            }
        }
        String::new()
    }

    pub fn get_item_timestamp(&self, index: i32) -> String {
        if let Some(entry) = self.entries.get(index as usize) {
            return entry.timestamp.format("%H:%M:%S").to_string();
        }
        String::new()
    }

    pub fn get_item_type(&self, index: i32) -> String {
        if let Some(entry) = self.entries.get(index as usize) {
            return match entry.content_type {
                clippit_ipc::ContentType::Text => "text".to_string(),
                clippit_ipc::ContentType::Image => "image".to_string(),
            };
        }
        "text".to_string()
    }
}

impl Default for HistoryModel {
    fn default() -> Self {
        Self::new()
    }
}
