use image::{GenericImageView, ImageFormat};
use std::io::Cursor;

const MAX_SIZE_BYTES: usize = 200 * 1024; // 200 KiB
const TARGET_MAX_DIMENSION: u32 = 1024;

pub fn optimize_image(data: &[u8], content_type: &str) -> Result<(Vec<u8>, String), String> {
    // If image is already smaller than limit, return as is
    if data.len() <= MAX_SIZE_BYTES {
        return Ok((data.to_vec(), content_type.to_string()));
    }

    // Try to detect format from content type, or fallback to guessing from bytes
    let format = match content_type {
        "image/png" => ImageFormat::Png,
        "image/jpeg" | "image/jpg" => ImageFormat::Jpeg,
        "image/webp" => ImageFormat::WebP,
        "image/gif" => ImageFormat::Gif,
        _ => image::guess_format(data).map_err(|e| format!("Unknown image format: {}", e))?,
    };

    let img = image::load_from_memory_with_format(data, format)
        .map_err(|e| format!("Failed to load image: {}", e))?;

    let (width, height) = img.dimensions();
    
    // Resize if dimensions are too large, otherwise just re-encoding might save space
    let new_img = if width > TARGET_MAX_DIMENSION || height > TARGET_MAX_DIMENSION {
        img.resize(TARGET_MAX_DIMENSION, TARGET_MAX_DIMENSION, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    
    // Convert to JPEG for best compression/quality ratio for receipts
    // This usually results in < 200KB for 1024px images
    new_img.write_to(&mut cursor, ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to write optimized image: {}", e))?;

    Ok((buffer, "image/jpeg".to_string()))
}
