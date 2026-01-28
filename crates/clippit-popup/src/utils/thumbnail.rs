use anyhow::Result;
use gdk_pixbuf::prelude::*;
use std::io::Cursor;

/// Creates a thumbnail from image data
///
/// # Arguments
/// * `image_data` - Raw image bytes
/// * `size` - Maximum dimension (width or height) for the thumbnail
///
/// # Returns
/// A `gdk_pixbuf::Pixbuf` containing the thumbnail
pub fn create_thumbnail(image_data: &[u8], size: u32) -> Result<gdk_pixbuf::Pixbuf> {
    // Load image from memory
    let img = image::load_from_memory(image_data)?;

    // Calculate aspect ratio and resize
    let (width, height) = (img.width(), img.height());
    let scale = (size as f32) / width.max(height) as f32;
    let new_width = (width as f32 * scale) as u32;
    let new_height = (height as f32 * scale) as u32;

    let thumb = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert to PNG bytes
    let mut buf = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

    // Load into GdkPixbuf
    let loader = gdk_pixbuf::PixbufLoader::new();
    loader.write(&buf)?;
    loader.close()?;

    Ok(loader
        .pixbuf()
        .ok_or_else(|| anyhow::anyhow!("Failed to load pixbuf"))?)
}
