use image::{Rgba, RgbaImage};

use crate::domain::config::EraserStroke;

/// Source-eraser: round-tipped strokes applied to the original image before
/// the render pipeline runs. Coordinates use the 256 preview space and are
/// mapped linearly onto the source image, mirroring the magic wand.
pub fn apply_eraser_strokes(img: &mut RgbaImage, strokes: &[EraserStroke]) {
    if img.width() == 0 || img.height() == 0 {
        return;
    }
    let (w, h) = (img.width() as f32, img.height() as f32);
    for stroke in strokes {
        let hardness = (stroke.hardness.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
        // Max outer radius in source pixels; the brush is round by design.
        let radius = ((stroke.size.clamp(1.0, 256.0) / 2.0) / 256.0 * w.max(h)).max(0.5);
        for point in &stroke.points {
            let x = (point.x / 256.0) * w;
            let y = (point.y / 256.0) * h;
            stamp_circle(img, x, y, radius, hardness);
        }
    }
}

fn stamp_circle(img: &mut RgbaImage, cx: f32, cy: f32, radius: f32, hardness: f32) {
    let (w, h) = (img.width() as f32, img.height() as f32);
    let r2 = radius * radius;
    let min_x = (cx - radius).floor().max(0.0) as u32;
    let max_x = ((cx + radius).ceil()).min(w) as u32;
    let min_y = (cy - radius).floor().max(0.0) as u32;
    let max_y = ((cy + radius).ceil()).min(h) as u32;
    let inner_radius = radius * hardness;
    let inner2 = inner_radius * inner_radius;
    let inv = if radius - inner_radius > 0.001 {
        1.0 / (radius - inner_radius)
    } else {
        f32::MAX
    };
    for y in min_y..max_y {
        for x in min_x..max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d2 = dx * dx + dy * dy;
            if d2 > r2 {
                continue;
            }
            // 1 at center / inside the hard core, falling to 0 at the rim.
            let falloff = if d2 <= inner2 {
                1.0
            } else {
                let t = (d2.sqrt() - inner_radius) * inv;
                1.0 - t.clamp(0.0, 1.0)
            };
            if falloff <= 0.0 {
                continue;
            }
            let p = img.get_pixel(x, y);
            let a = p.0[3] as f32 * (1.0 - falloff);
            img.put_pixel(x, y, Rgba([0, 0, 0, a.round() as u8]));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::WandPoint;

    fn stroke(points: Vec<(f32, f32)>, size: f32, hardness: f32) -> EraserStroke {
        EraserStroke {
            points: points.into_iter().map(|(x, y)| WandPoint { x, y }).collect(),
            size,
            hardness,
        }
    }

    #[test]
    fn hard_eraser_zeroes_the_core() {
        let mut img = RgbaImage::from_pixel(32, 32, Rgba([255, 0, 0, 255]));
        apply_eraser_strokes(
            &mut img,
            &[stroke(vec![(128.0, 128.0)], 32.0, 100.0)],
        );
        // Center fully erased, radius 16 in 256-space => 2 px on a 32 px image.
        assert_eq!(img.get_pixel(16, 16).0[3], 0);
        // Outside the stamp still opaque.
        assert_eq!(img.get_pixel(0, 0).0[3], 255);
    }
}