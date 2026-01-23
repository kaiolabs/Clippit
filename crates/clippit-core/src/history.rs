use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::storage::Storage;
use crate::types::{ClipboardEntry, ContentType};
use crate::validator::ContentValidator;

pub struct HistoryManager {
    storage: Storage,
    last_hash: Option<String>,
    max_entries: usize,
}

impl HistoryManager {
    pub fn new(db_path: PathBuf, max_entries: usize) -> Result<Self> {
        let storage = Storage::new(db_path)?;
        Ok(Self {
            storage,
            last_hash: None,
            max_entries,
        })
    }

    pub fn new_in_memory(max_entries: usize) -> Result<Self> {
        let storage = Storage::in_memory()?;
        Ok(Self {
            storage,
            last_hash: None,
            max_entries,
        })
    }

    fn compute_hash(entry: &ClipboardEntry) -> String {
        let mut hasher = Sha256::new();

        match &entry.content_type {
            ContentType::Text => {
                if let Some(text) = &entry.content_text {
                    hasher.update(b"text:");
                    hasher.update(text.as_bytes());
                }
            }
            ContentType::Image => {
                // For file-based images, use image_path for hash
                if let Some(path) = &entry.image_path {
                    hasher.update(b"image:path:");
                    hasher.update(path.as_bytes());
                } else if let Some(data) = &entry.content_data {
                    // Legacy: use content_data if available (old entries)
                    hasher.update(b"image:data:");
                    hasher.update(data);
                }
            }
        }

        format!("{:x}", hasher.finalize())
    }

    pub fn add_entry(&mut self, mut entry: ClipboardEntry) -> Result<Option<i64>> {
        // Validate content
        match &entry.content_type {
            ContentType::Text => {
                if let Some(text) = &entry.content_text {
                    ContentValidator::validate_text(text)?;
                } else {
                    warn!("Text entry without content_text");
                    return Ok(None);
                }
            }
            ContentType::Image => {
                // For file-based images, we only have image_path (no content_data)
                // For legacy images, we might have content_data
                if let Some(data) = &entry.content_data {
                    ContentValidator::validate_image(data)?;
                } else if entry.image_path.is_none() {
                    // Only warn if BOTH content_data AND image_path are missing
                    warn!("Image entry without content_data or image_path");
                    return Ok(None);
                }
                // If image_path exists, it's valid (file-based storage)
            }
        }

        // Check for duplicates nos últimos 10 itens do histórico
        let hash = Self::compute_hash(&entry);
        
        // Verifica se já existe nos últimos itens
        let recent = self.storage.get_recent(10)?;
        for existing in recent {
            let existing_hash = Self::compute_hash(&existing);
            if existing_hash == hash {
                info!("Skipping duplicate entry (already exists in history)");
                return Ok(None);
            }
        }

        // Insert into storage
        let id = self.storage.insert(&entry)?;
        entry.id = id;

        // Update last hash
        self.last_hash = Some(hash);

        // Prune old entries if necessary
        let count = self.storage.count()?;
        if count > self.max_entries {
            let pruned = self.storage.prune_old(self.max_entries)?;
            info!("Pruned {} old entries", pruned);
        }

        info!("Added entry with id {}", id);
        Ok(Some(id))
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        self.storage.get_recent(limit)
    }

    /// Get recent entries without loading image data (metadata only)
    /// Optimized for fast listing in UI
    pub fn get_recent_metadata(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        self.storage.get_recent_metadata(limit)
    }

    /// Get recent entries with offset (for infinite scroll)
    pub fn get_recent_metadata_with_offset(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
        self.storage.get_recent_metadata_with_offset(limit, offset)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        self.storage.get_by_id(id)
    }

    pub fn prune_old(&self) -> Result<usize> {
        self.storage.prune_old(self.max_entries)
    }

    pub fn count(&self) -> Result<usize> {
        self.storage.count()
    }

    pub fn delete_by_id(&self, id: i64) -> Result<bool> {
        self.storage.delete_by_id(id)
    }
    
    pub fn clear(&self) -> Result<usize> {
        self.storage.clear()
    }

    /// Search in ALL history (no limit) - returns metadata only for images
    pub fn search(&self, query: &str) -> Result<Vec<ClipboardEntry>> {
        self.storage.search(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_entry() {
        let mut manager = HistoryManager::new_in_memory(100).unwrap();
        let entry = ClipboardEntry::new_text("Test".to_string());

        let id = manager.add_entry(entry).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn test_duplicate_detection() {
        let mut manager = HistoryManager::new_in_memory(100).unwrap();
        let entry1 = ClipboardEntry::new_text("Test".to_string());
        let entry2 = ClipboardEntry::new_text("Test".to_string());

        let id1 = manager.add_entry(entry1).unwrap();
        assert!(id1.is_some());

        let id2 = manager.add_entry(entry2).unwrap();
        assert!(id2.is_none()); // Should be skipped as duplicate
    }

    #[test]
    fn test_max_entries() {
        let mut manager = HistoryManager::new_in_memory(5).unwrap();

        for i in 0..10 {
            let entry = ClipboardEntry::new_text(format!("Entry {}", i));
            manager.add_entry(entry).unwrap();
        }

        assert_eq!(manager.count().unwrap(), 5);
    }
}
