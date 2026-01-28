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
    pub image_path: Option<String>,    // Path to image file on disk
    pub thumbnail_data: Option<Vec<u8>>, // 128x128 thumbnail for images
    pub image_width: Option<u32>,      // Image width in pixels (avoid loading full image)
    pub image_height: Option<u32>,     // Image height in pixels (avoid loading full image)
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
            image_width: None,
            image_height: None,
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
            image_width: None,
            image_height: None,
            timestamp: Utc::now(),
        }
    }

    pub fn new_image_with_dimensions(
        image_path: String,
        thumbnail: Option<Vec<u8>>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            id: 0,
            content_type: ContentType::Image,
            content_text: None,
            content_data: None,
            image_path: Some(image_path),
            thumbnail_data: thumbnail,
            image_width: Some(width),
            image_height: Some(height),
            timestamp: Utc::now(),
        }
    }
}
