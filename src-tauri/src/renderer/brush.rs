use image::RgbaImage;

use crate::domain::config::{BrushMode, BrushStroke};
use crate::error::app_error::AppError;

use super::color::parse_hex_rgba;
use super::composite::{over, LinearPremultipliedImage};

pub fn composite_brush_strokes(
    image: RgbaImage,
    strokes: &[BrushStroke],
) -> Result<RgbaImage, AppError> {
    if strokes.is_empty() {
        return Ok(image);
    }

    let mut base = LinearPremultipliedImage::from_srgb_rgba8(&image);
    let original_mask = base.alpha_plane();

    for stroke in strokes {
        if stroke.points.is_empty()
            || !stroke.size.is_finite()
            || !(0.5..=128.0).contains(&stroke.size)
            || !stroke.opacity.is_finite()
            || !(0.0..=1.0).contains(&stroke.opacity)
        {
            continue;
        }
        let color = parse_hex_rgba(&stroke.color)?;
        let alpha = color.a * stroke.opacity;
        let rgba = [color.r * alpha, color.g * alpha, color.b * alpha, alpha];
        let radius = stroke.size / 2.0;
        let mut layer = LinearPremultipliedImage::transparent(256, 256);

        if stroke.points.len() == 1 {
            stamp(
                &mut layer,
                stroke.points[0].x,
                stroke.points[0].y,
                radius,
                rgba,
            );
        } else {
            for pair in stroke.points.windows(2) {
                let start = &pair[0];
                let end = &pair[1];
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let distance = (dx * dx + dy * dy).sqrt();
                let steps = (distance / (radius * 0.35).max(0.25)).ceil().max(1.0) as u32;
                for step in 0..=steps {
                    let t = step as f32 / steps as f32;
                    stamp(&mut layer, start.x + dx * t, start.y + dy * t, radius, rgba);
                }
            }
        }

        apply_stroke(
            &mut base,
            &layer,
            stroke.mode,
            stroke.clip_to_mask,
            &original_mask,
        );
    }
    Ok(base.into_srgb_rgba8())
}

fn apply_stroke(
    base: &mut LinearPremultipliedImage,
    layer: &LinearPremultipliedImage,
    mode: BrushMode,
    clip_to_mask: bool,
    mask: &[f32],
) {
    if mode == BrushMode::Paint {
        let mut paint = layer.clone();
        if clip_to_mask {
            for (pixel, coverage) in paint.data.iter_mut().zip(mask.iter()) {
                for channel in pixel.iter_mut() {
                    *channel *= *coverage;
                }
            }
        }
        over(base, paint);
        return;
    }

    for (index, pixel) in base.data.iter_mut().enumerate() {
        let mut erase = layer.data[index][3];
        if clip_to_mask {
            erase *= mask[index];
        }
        let keep = 1.0 - erase.clamp(0.0, 1.0);
        for channel in pixel.iter_mut() {
            *channel *= keep;
        }
    }
}

fn stamp(target: &mut LinearPremultipliedImage, cx: f32, cy: f32, radius: f32, rgba: [f32; 4]) {
    if !cx.is_finite() || !cy.is_finite() || radius <= 0.0 {
        return;
    }
    let min_x = (cx - radius - 1.0).floor().max(0.0) as u32;
    let min_y = (cy - radius - 1.0).floor().max(0.0) as u32;
    let max_x = (cx + radius + 1.0).ceil().min(255.0) as u32;
    let max_y = (cy + radius + 1.0).ceil().min(255.0) as u32;
    let edge = 1.0_f32.min(radius);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let distance = (dx * dx + dy * dy).sqrt();
            let coverage = ((radius - distance) / edge + 0.5).clamp(0.0, 1.0);
            if coverage <= 0.0 {
                continue;
            }
            let idx = (y * 256 + x) as usize;
            let source_alpha = rgba[3] * coverage;
            let inverse = 1.0 - source_alpha;
            target.data[idx][0] = rgba[0] * coverage + target.data[idx][0] * inverse;
            target.data[idx][1] = rgba[1] * coverage + target.data[idx][1] * inverse;
            target.data[idx][2] = rgba[2] * coverage + target.data[idx][2] * inverse;
            target.data[idx][3] = source_alpha + target.data[idx][3] * inverse;
        }
    }
}
