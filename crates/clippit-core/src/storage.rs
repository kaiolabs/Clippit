use crate::types::{ClipboardEntry, ContentType};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use std::time::Duration;

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
        // Configure SQLite for concurrent access (OCR thread writes while monitor reads)
        self.conn.pragma_update(None, "journal_mode", "WAL")?;
        self.conn.busy_timeout(Duration::from_secs(5))?; // Wait up to 5s for locks

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

        // Migration: Add image_width column if it doesn't exist
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN image_width INTEGER",
            [],
        );

        // Migration: Add image_height column if it doesn't exist
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN image_height INTEGER",
            [],
        );

        // Migration: Add ocr_text column if it doesn't exist (for OCR feature)
        let _ = self.conn.execute(
            "ALTER TABLE clipboard_history ADD COLUMN ocr_text TEXT",
            [],
        );

        // Check if FTS5 table has ocr_text column (migration)
        let needs_fts_migration = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('clipboard_history_fts') WHERE name='ocr_text'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap_or(0)
            == 0;

        if needs_fts_migration {
            // Drop and recreate FTS5 table with ocr_text
            let _ = self.conn.execute("DROP TABLE IF EXISTS clipboard_history_fts", []);
            let _ = self.conn.execute("DROP TRIGGER IF EXISTS clipboard_history_ai", []);
            let _ = self.conn.execute("DROP TRIGGER IF EXISTS clipboard_history_au", []);
            let _ = self.conn.execute("DROP TRIGGER IF EXISTS clipboard_history_ad", []);
        }

        // Create FTS5 virtual table for fast text search (includes ocr_text)
        let _ = self.conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_history_fts
             USING fts5(content_text, ocr_text, content='clipboard_history', content_rowid='id')",
            [],
        );

        // Trigger to keep FTS5 table synchronized on INSERT (includes ocr_text)
        // COALESCE ensures NULL is converted to empty string (FTS5 doesn't support NULL)
        let _ = self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS clipboard_history_ai
             AFTER INSERT ON clipboard_history BEGIN
                 INSERT INTO clipboard_history_fts(rowid, content_text, ocr_text)
                 VALUES (new.id, new.content_text, COALESCE(new.ocr_text, ''));
             END",
            [],
        );

        // Trigger to keep FTS5 table synchronized on UPDATE (includes ocr_text)
        // COALESCE ensures NULL is converted to empty string (FTS5 doesn't support NULL)
        let _ = self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS clipboard_history_au
             AFTER UPDATE ON clipboard_history BEGIN
                 UPDATE clipboard_history_fts
                 SET content_text = new.content_text,
                     ocr_text = COALESCE(new.ocr_text, '')
                 WHERE rowid = new.id;
             END",
            [],
        );

        // Trigger to keep FTS5 table synchronized on DELETE
        let _ = self.conn.execute(
            "CREATE TRIGGER IF NOT EXISTS clipboard_history_ad
             AFTER DELETE ON clipboard_history BEGIN
                 DELETE FROM clipboard_history_fts WHERE rowid = old.id;
             END",
            [],
        );

        // Rebuild FTS5 index if table exists but is empty (migration case)
        let fts_count: Result<i64, _> =
            self.conn
                .query_row("SELECT COUNT(*) FROM clipboard_history_fts", [], |row| {
                    row.get(0)
                });
        let main_count: Result<i64, _> =
            self.conn
                .query_row("SELECT COUNT(*) FROM clipboard_history", [], |row| {
                    row.get(0)
                });

        if let (Ok(fts_cnt), Ok(main_cnt)) = (fts_count, main_count) {
            if fts_cnt == 0 && main_cnt > 0 {
                // Rebuild FTS index from existing data (includes ocr_text)
                let _ = self.conn.execute(
                    "INSERT INTO clipboard_history_fts(rowid, content_text, ocr_text)
                     SELECT id, content_text, ocr_text FROM clipboard_history 
                     WHERE content_text IS NOT NULL OR ocr_text IS NOT NULL",
                    [],
                );
            }
        }

        Ok(())
    }

    pub fn insert(&self, entry: &ClipboardEntry) -> Result<i64> {
        let content_type_str = match entry.content_type {
            ContentType::Text => "text",
            ContentType::Image => "image",
        };

        let timestamp = entry.timestamp.to_rfc3339();

        self.conn.execute(
            "INSERT INTO clipboard_history (content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, ocr_text, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                content_type_str,
                entry.content_text,
                entry.content_data,
                entry.image_path,
                entry.thumbnail_data,
                entry.image_width,
                entry.image_height,
                entry.ocr_text,
                timestamp,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, ocr_text, timestamp
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

            let timestamp_str: String = row.get(9)?;
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
                image_width: row.get(6)?,
                image_height: row.get(7)?,
                ocr_text: row.get(8)?,
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
                    image_width,
                    image_height,
                    ocr_text,
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

            let timestamp_str: String = row.get(9)?;
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
                image_width: row.get(6)?,
                image_height: row.get(7)?,
                ocr_text: row.get(8)?,
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
    pub fn get_recent_metadata_with_offset(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, content_text, 
                    CASE 
                        WHEN content_type = 'image' THEN NULL 
                        ELSE content_data 
                    END as content_data,
                    image_path,
                    thumbnail_data,
                    image_width,
                    image_height,
                    ocr_text,
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

            let timestamp_str: String = row.get(9)?;
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
                image_width: row.get(6)?,
                image_height: row.get(7)?,
                ocr_text: row.get(8)?,
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
            "SELECT id, content_type, content_text, content_data, image_path, thumbnail_data, image_width, image_height, ocr_text, timestamp
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

            let timestamp_str: String = row.get(9)?;
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
                image_width: row.get(6)?,
                image_height: row.get(7)?,
                ocr_text: row.get(8)?,
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

        let deleted = self.conn.execute("DELETE FROM clipboard_history", [])?;
        Ok(deleted)
    }

    /// Search in ALL history entries (no limit) - metadata only for images
    /// Uses FTS5 for fast text search when possible
    pub fn search(&self, query: &str) -> Result<Vec<ClipboardEntry>> {
        // Determine if we should use FTS5 (fast) or LIKE (compatible)
        // FTS5 is faster but doesn't support % wildcards
        let use_fts = !query.contains('%') && !query.contains('_') && !query.is_empty();

        if use_fts {
            // Fast FTS5 search with prefix matching
            // Add * to each word for prefix search: "lingua" → "lingua*"
            let fts_query = query
                .split_whitespace()
                .map(|word| format!("{}*", word))
                .collect::<Vec<_>>()
                .join(" OR ");
            let search_pattern = format!("%{}%", query);

            let mut stmt = self.conn.prepare(
                "SELECT h.id, h.content_type, h.content_text,
                        CASE
                            WHEN h.content_type = 'image' THEN NULL
                            ELSE h.content_data
                        END as content_data,
                        h.image_path,
                        h.thumbnail_data,
                        h.image_width,
                        h.image_height,
                        h.ocr_text,
                        h.timestamp
                 FROM clipboard_history h
                 INNER JOIN clipboard_history_fts fts ON h.id = fts.rowid
                 WHERE fts.content_text MATCH ?1 OR fts.ocr_text MATCH ?1
                 UNION
                 SELECT id, content_type, content_text,
                        CASE
                            WHEN content_type = 'image' THEN NULL
                            ELSE content_data
                        END as content_data,
                        image_path,
                        thumbnail_data,
                        image_width,
                        image_height,
                        ocr_text,
                        timestamp
                 FROM clipboard_history
                 WHERE content_type = 'image' AND image_path LIKE ?2
                 ORDER BY timestamp DESC",
            )?;

            let entries = stmt
                .query_map(params![&fts_query, &search_pattern], |row| {
                    let content_type_str: String = row.get(1)?;
                    let content_type = match content_type_str.as_str() {
                        "text" => ContentType::Text,
                        "image" => ContentType::Image,
                        _ => ContentType::Text,
                    };

                    let timestamp_str: String = row.get(9)?;
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
                        image_width: row.get(6)?,
                        image_height: row.get(7)?,
                        ocr_text: row.get(8)?,
                        timestamp,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(entries)
        } else {
            // Fallback to LIKE for pattern matching
            let search_pattern = format!("%{}%", query);

            let mut stmt = self.conn.prepare(
                "SELECT id, content_type, content_text,
                        CASE
                            WHEN content_type = 'image' THEN NULL
                            ELSE content_data
                        END as content_data,
                        image_path,
                        thumbnail_data,
                        image_width,
                        image_height,
                        ocr_text,
                        timestamp
                 FROM clipboard_history
                 WHERE content_text LIKE ?1
                    OR image_path LIKE ?1
                    OR ocr_text LIKE ?1
                 ORDER BY timestamp DESC",
            )?;

            let entries = stmt
                .query_map([&search_pattern], |row| {
                    let content_type_str: String = row.get(1)?;
                    let content_type = match content_type_str.as_str() {
                        "text" => ContentType::Text,
                        "image" => ContentType::Image,
                        _ => ContentType::Text,
                    };

                    let timestamp_str: String = row.get(9)?;
                    let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)
                        .unwrap_or_else(|_| Utc::now().into())
                        .with_timezone(&Utc);

                    Ok(ClipboardEntry {
                        id: row.get(0)?,
                        content_type,
                        content_text: row.get(2)?,
                        content_data: row.get(3)?, // Will be None for images due to CASE
                        image_path: row.get(4)?,
                        thumbnail_data: row.get(5)?,
                        image_width: row.get(6)?,
                        image_height: row.get(7)?,
                        ocr_text: row.get(8)?,
                        timestamp,
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;

            Ok(entries)
        }
    }

    /// Atualiza texto OCR de uma entrada existente (usado pelo OCR processor)
    pub fn update_ocr_text(&self, id: i64, ocr_text: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE clipboard_history SET ocr_text = ?1 WHERE id = ?2",
            params![ocr_text, id],
        )?;
        Ok(())
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
