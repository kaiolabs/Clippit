use crate::types::{ClipboardEntry, ContentType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let storage = Self { conn };
        storage.initialize()?;
        Ok(storage)
    }

    fn initialize(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL,
                content_text TEXT,
                content_data BLOB,
                image_path TEXT,
                thumbnail_data BLOB,
                timestamp TEXT NOT NULL
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_history(timestamp DESC)",
            [],
        )?;

        // Migration: Add thumbnail_data column if it doesn't exist
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN thumbnail_data BLOB",
            [],
        );

        // Migration: Add image_path column if it doesn't exist
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN image_path TEXT",
            [],
        );

        Ok(())
    }

    pub fn insert(&self, entry: &ClipboardEntry) -> Result<i64> {
        let content_type_str = match entry.content_type {
            ContentType::Text => "text",
            ContentType::Image => "image",
        };

        let timestamp = entry.timestamp.to_rfc3339();

        self.conn.execute(
            "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                content_type_str,
                entry.content_text,
                entry.content_data,
                entry.image_path,
                entry.thumbnail_data,
                timestamp,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, content_data, image_path, thumbnail_data, timestamp
             FROM clipboard_history
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let entries = stmt.query_map([limit], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = match content_type_str.as_str() {
                "text" => ContentType::Text,
                "image" => ContentType::Image,
                _ => ContentType::Text,
            };

            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                content_text: row.get(2)?,
                content_data: row.get(3)?,
                image_path: row.get(4)?,
                thumbnail_data: row.get(5)?,
                timestamp,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// Get recent entries without loading full image data (metadata only)
    /// This is optimized for listing - images return image_path and thumbnail_data
    pub fn get_recent_metadata(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, 
                    CASE 
                        WHEN content_type = 'image' THEN NULL 
                        ELSE content_data 
                    END as content_data,
                    image_path,
                    thumbnail_data,
                    timestamp
             FROM clipboard_history
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let entries = stmt.query_map([limit], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = match content_type_str.as_str() {
                "text" => ContentType::Text,
                "image" => ContentType::Image,
                _ => ContentType::Text,
            };

            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                content_text: row.get(2)?,
                content_data: row.get(3)?, // Will be None for images due to CASE statement
                image_path: row.get(4)?,
                thumbnail_data: row.get(5)?, // Contains thumbnail for images
                timestamp,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    /// Get recent entries with offset (for infinite scroll)
    pub fn get_recent_metadata_with_offset(&self, limit: usize, offset: usize) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, 
                    CASE 
                        WHEN content_type = 'image' THEN NULL 
                        ELSE content_data 
                    END as content_data,
                    image_path,
                    thumbnail_data,
                    timestamp
             FROM clipboard_history
             ORDER BY timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let entries = stmt.query_map([limit, offset], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = match content_type_str.as_str() {
                "text" => ContentType::Text,
                "image" => ContentType::Image,
                _ => ContentType::Text,
            };

            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                content_text: row.get(2)?,
                content_data: row.get(3)?,
                image_path: row.get(4)?,
                thumbnail_data: row.get(5)?,
                timestamp,
            })
        })?;

        let mut result = Vec::new();
        for entry in entries {
            result.push(entry?);
        }

        Ok(result)
    }

    pub fn get_by_id(&self, id: i64) -> Result<Option<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, content_data, image_path, thumbnail_data, timestamp
             FROM clipboard_history
             WHERE id = ?1",
        )?;

        let mut rows = stmt.query([id])?;

        if let Some(row) = rows.next()? {
            let content_type_str: String = row.get(1)?;
            let content_type = match content_type_str.as_str() {
                "text" => ContentType::Text,
                "image" => ContentType::Image,
                _ => ContentType::Text,
            };

            let timestamp_str: String = row.get(6)?;
            let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                .unwrap_or_else(|_| Utc::now().into())
                .with_timezone(&Utc);

            Ok(Some(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                content_text: row.get(2)?,
                content_data: row.get(3)?,
                image_path: row.get(4)?,
                thumbnail_data: row.get(5)?,
                timestamp,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn count(&self) -> Result<usize> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
                    row.get(0)
                })?;
        Ok(count as usize)
    }

    pub fn prune_old(&self, keep_count: usize) -> Result<usize> {
        let count = self.count()?;

        if count <= keep_count {
            return Ok(0);
        }

        let to_delete = count - keep_count;

        let deleted = self.conn.execute(
            "DELETE FROM clipboard_history
             WHERE id IN (
                 SELECT id FROM clipboard_history
                 ORDER BY timestamp ASC
                 LIMIT ?1
             )",
            [to_delete],
        )?;

        Ok(deleted)
    }

    pub fn delete_by_id(&self, id: i64) -> Result<bool> {
        // First, get the entry to check if it has an image file to delete
        if let Some(entry) = self.get_by_id(id)? {
            if let Some(image_path) = entry.image_path {
                // Try to delete the image file (ignore errors if file doesn't exist)
                let _ = std::fs::remove_file(&image_path);
            }
        }
        
        let deleted = self
            .conn
            .execute("DELETE FROM clipboard_history WHERE id = ?1", [id])?;
        Ok(deleted > 0)
    }
    
    pub fn clear(&self) -> Result<usize> {
        // Get all image paths before deleting
        let entries = self.get_recent(10000)?; // Get all entries (max 10k)
        
        for entry in entries {
            if let Some(image_path) = entry.image_path {
                // Try to delete each image file (ignore errors)
                let _ = std::fs::remove_file(&image_path);
            }
        }
        
        let deleted = self
            .conn
            .execute("DELETE FROM clipboard_history", [])?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = Storage::in_memory().unwrap();
        assert_eq!(storage.count().unwrap(), 0);
    }

    #[test]
    fn test_insert_and_retrieve() {
        let storage = Storage::in_memory().unwrap();
        let entry = ClipboardEntry::new_text("Test content".to_string());

        let id = storage.insert(&entry).unwrap();
        assert!(id > 0);

        let retrieved = storage.get_by_id(id).unwrap().unwrap();
        assert_eq!(retrieved.content_text, Some("Test content".to_string()));
    }

    #[test]
    fn test_prune_old() {
        let storage = Storage::in_memory().unwrap();

        for i in 0..10 {
            let entry = ClipboardEntry::new_text(format!("Entry {}", i));
            storage.insert(&entry).unwrap();
        }

        assert_eq!(storage.count().unwrap(), 10);

        let deleted = storage.prune_old(5).unwrap();
        assert_eq!(deleted, 5);
        assert_eq!(storage.count().unwrap(), 5);
    }
}
