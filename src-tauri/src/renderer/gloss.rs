use super::color::parse_hex_rgba;
use super::composite::LinearPremultipliedImage;
use super::mask::{generate_inset_shape_mask, AlphaMask};
use super::pipeline::ValidatedRenderConfig;
use crate::error::app_error::AppError;

pub fn render_edge_gloss(
    mask: &AlphaMask,
    config: &ValidatedRenderConfig,
) -> Result<LinearPremultipliedImage, AppError> {
    let size = 256u32;
    let inset = generate_inset_shape_mask(size, config, config.canvas_inset + config.gloss_width);
    let light = parse_hex_rgba(&config.gloss_light_color)?;
    let dark = parse_hex_rgba(&config.gloss_dark_color)?;
    let mut result = LinearPremultipliedImage::transparent(size, size);

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) as usize;
            let edge = (mask[idx] - inset[idx]).clamp(0.0, 1.0);
            if edge <= 0.0 {
                continue;
            }
            let nx = x as f32 / 255.0 * 2.0 - 1.0;
            let ny = y as f32 / 255.0 * 2.0 - 1.0;
            let direction = ((-nx - ny) * 0.5).clamp(-1.0, 1.0);
            let (color, weight) = if direction >= 0.0 {
                (&light, direction)
            } else {
                (&dark, -direction)
            };
            let alpha = color.a * edge * weight * config.gloss_strength;
            result.data[idx] = [color.r * alpha, color.g * alpha, color.b * alpha, alpha];
        }
    }
    Ok(result)
}
