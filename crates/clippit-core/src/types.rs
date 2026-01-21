use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: Option<String>,
    pub content_data: Option<Vec<u8>>, // Only used for backwards compatibility, prefer image_path
    pub image_path: Option<String>, // Path to image file on disk
    pub thumbnail_data: Option<Vec<u8>>, // 128x128 thumbnail for images
    pub timestamp: DateTime<Utc>,
}

impl ClipboardEntry {
    pub fn new_text(text: String) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Text,
            content_text: Some(text),
            content_data: None,
            image_path: None,
            thumbnail_data: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_image(image_path: String, thumbnail: Option<Vec<u8>>) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Image,
            content_text: None,
            content_data: None,
            image_path: Some(image_path),
            thumbnail_data: thumbnail,
            timestamp: Utc::now(),
        }
    }
}
