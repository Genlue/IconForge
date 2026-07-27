use image::{imageops, Rgba, RgbaImage};

use crate::error::app_error::AppError;

/// Convert a single RGBA pixel to premultiplied alpha in place.
fn premultiply(pixel: &mut Rgba<u8>) {
    let a = pixel.0[3] as u32;
    if a == 0 || a == 255 {
        return;
    }
    pixel.0[0] = ((pixel.0[0] as u32 * a + 127) / 255) as u8;
    pixel.0[1] = ((pixel.0[1] as u32 * a + 127) / 255) as u8;
    pixel.0[2] = ((pixel.0[2] as u32 * a + 127) / 255) as u8;
}

/// Convert a single RGBA pixel back from premultiplied alpha in place.
/// For fully transparent pixels the RGB channels are set to zero.
fn unpremultiply(pixel: &mut Rgba<u8>) {
    let a = pixel.0[3] as u32;
    if a == 0 {
        pixel.0[0] = 0;
        pixel.0[1] = 0;
        pixel.0[2] = 0;
        return;
    }
    if a == 255 {
        return;
    }
    pixel.0[0] = ((pixel.0[0] as u32 * 255 + a / 2) / a).min(255) as u8;
    pixel.0[1] = ((pixel.0[1] as u32 * 255 + a / 2) / a).min(255) as u8;
    pixel.0[2] = ((pixel.0[2] as u32 * 255 + a / 2) / a).min(255) as u8;
}

pub fn resize_from_master(master: &RgbaImage, size: u32) -> Result<RgbaImage, AppError> {
    if size == 256 {
        return Ok(master.clone());
    }

    // Convert to premultiplied alpha so Lanczos downscaling blends
    // transparent edges correctly (avoids dark/bright fringing).
    let mut premultiplied = master.clone();
    for pixel in premultiplied.pixels_mut() {
        premultiply(pixel);
    }

    // Resize using Lanczos3 on the premultiplied image.
    let resized = imageops::resize(&premultiplied, size, size, imageops::Lanczos3);

    // Convert back from premultiplied alpha.
    let mut result = resized;
    for pixel in result.pixels_mut() {
        unpremultiply(pixel);
    }

    Ok(result)
}
