use image::{imageops, RgbaImage};

use crate::error::app_error::AppError;

pub fn resize_from_master(master: &RgbaImage, size: u32) -> Result<RgbaImage, AppError> {
    if size == 256 {
        return Ok(master.clone());
    }

    // Use image::imageops::resize with Lanczos3 filter directly
    let resized = imageops::resize(master, size, size, imageops::Lanczos3);

    // Re-premultiply alpha handling is not needed for imageops::resize
    // as it handles transparency correctly
    Ok(resized)
}
