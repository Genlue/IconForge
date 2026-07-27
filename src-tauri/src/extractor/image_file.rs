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

    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("ico"))
    {
        return decode_largest_ico_frame(path);
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

fn decode_largest_ico_frame(path: &Path) -> Result<RgbaImage, AppError> {
    let file = std::fs::File::open(path).map_err(AppError::io_for(path))?;
    let directory = ico::IconDir::read(file)
        .map_err(|error| AppError::UnsupportedImage(error.to_string(), path.to_path_buf()))?;
    let mut entries = directory.entries().iter().collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        let width = ico_dimension(entry.width());
        let height = ico_dimension(entry.height());
        (width * height, entry.bits_per_pixel())
    });

    for entry in entries.into_iter().rev() {
        let Ok(decoded) = entry.decode() else {
            continue;
        };
        if let Some(image) = RgbaImage::from_raw(
            decoded.width(),
            decoded.height(),
            decoded.rgba_data().to_vec(),
        ) {
            return Ok(image);
        }
    }

    Err(AppError::UnsupportedImage(
        "ICO contains no decodable frames".into(),
        path.to_path_buf(),
    ))
}

#[inline]
fn ico_dimension(value: u32) -> u32 {
    // ICO stores 256 as a zero byte in the directory entry.
    if value == 0 {
        256
    } else {
        value
    }
}
