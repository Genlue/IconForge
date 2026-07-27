use std::path::Path;

use image::RgbaImage;

use crate::error::app_error::AppError;

const MAX_FILE_SIZE: u64 = 256 * 1024 * 1024; // 256 MiB
const MAX_PIXEL_COUNT: u64 = 100_000_000;

pub fn decode_image(path: &Path) -> Result<RgbaImage, AppError> {
    let metadata = std::fs::metadata(path).map_err(AppError::io_for(path))?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(AppError::UnsupportedImage(
            format!("file too large: {} bytes", metadata.len()),
            path.to_path_buf(),
        ));
    }

    let reader = image::ImageReader::open(path)
        .map_err(AppError::io_for(path))?
        .with_guessed_format()
        .map_err(|e| AppError::UnsupportedImage(e.to_string(), path.to_path_buf()))?;

    let image = reader
        .decode()
        .map_err(|e| AppError::UnsupportedImage(e.to_string(), path.to_path_buf()))?;

    let pixel_count = (image.width() as u64)
        .checked_mul(image.height() as u64)
        .ok_or_else(|| {
            AppError::UnsupportedImage("image dimensions overflow".into(), path.to_path_buf())
        })?;

    if pixel_count > MAX_PIXEL_COUNT {
        return Err(AppError::UnsupportedImage(
            format!("image too large: {}x{}", image.width(), image.height()),
            path.to_path_buf(),
        ));
    }

    Ok(image.to_rgba8())
}
