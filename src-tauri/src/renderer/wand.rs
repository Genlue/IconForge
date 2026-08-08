use std::collections::VecDeque;

use image::RgbaImage;

use crate::domain::config::WandStroke;

const NEIGHBORS: [(i64, i64); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

/// Magic wand: flood-fill from a seed point and delete the connected region
/// (fully transparent). Strokes use 256-space coordinates, mapped linearly
/// onto the source image like the preview coordinate system used by brushes.
pub fn apply_erasures(img: &mut RgbaImage, strokes: &[WandStroke]) {
    if strokes.is_empty() || img.width() == 0 || img.height() == 0 {
        return;
    }
    let (w, h) = (img.width(), img.height());
    for stroke in strokes {
        // Max channel euclidean distance on the RGB cube for 0..=100 tolerance.
        let max_sq = ((stroke.tolerance.clamp(0.0, 100.0)) * 4.41).powi(2);
        for point in &stroke.points {
            let x = ((point.x / 256.0) * w as f32).round() as i64;
            let y = ((point.y / 256.0) * h as f32).round() as i64;
            if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
                continue;
            }
            let seed = img.get_pixel(x as u32, y as u32).0;
            if seed[3] == 0 {
                continue;
            }
            flood_fill_erase(img, x as u32, y as u32, seed, max_sq);
        }
    }
}

fn dist_sq(a: [u8; 4], b: [u8; 4]) -> f32 {
    let dr = a[0] as f32 - b[0] as f32;
    let dg = a[1] as f32 - b[1] as f32;
    let db = a[2] as f32 - b[2] as f32;
    dr * dr + dg * dg + db * db
}

/// Flood a connected region around a seed point. Returns a boolean mask in
/// image space (same dimensions as `img`). Only pixels whose color is within
/// `max_sq` of the seed are marked.
fn flood_region(img: &RgbaImage, sx: u32, sy: u32, seed: [u8; 4], max_sq: f32) -> Vec<bool> {
    let (w, h) = img.dimensions();
    let mut region = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();
    queue.push_back((sx, sy));
    region[(sy * w + sx) as usize] = true;

    while let Some((x, y)) = queue.pop_front() {
        for (dx_, dy_) in NEIGHBORS {
            let nx = x as i64 + dx_;
            let ny = y as i64 + dy_;
            if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                continue;
            }
            let idx = (ny as u64 * w as u64 + nx as u64) as usize;
            if region[idx] {
                continue;
            }
            let pixel = img.get_pixel(nx as u32, ny as u32).0;
            if pixel[3] > 0 && dist_sq(pixel, seed) <= max_sq {
                region[idx] = true;
                queue.push_back((nx as u32, ny as u32));
            }
        }
    }
    region
}

fn flood_fill_erase(img: &mut RgbaImage, sx: u32, sy: u32, seed: [u8; 4], max_sq: f32) {
    let region = flood_region(img, sx, sy, seed, max_sq);
    for (i, &selected) in region.iter().enumerate() {
        if selected {
            img.put_pixel(
                (i % img.width() as usize) as u32,
                (i / img.width() as usize) as u32,
                image::Rgba([0, 0, 0, 0]),
            );
        }
    }
}

fn max_sq_for(tolerance: f32) -> f32 {
    (tolerance.clamp(0.0, 100.0) * 4.41).powi(2)
}

/// Compute the magic wand selection as a 256×256 binarized mask (0 / 255).
/// The flood fill runs at full source resolution, then the mask is box-
/// downsampled for preview overlay use.
pub fn selection_mask_256(img: &RgbaImage, x: f32, y: f32, tolerance: f32) -> Option<Vec<u8>> {
    if img.width() == 0 || img.height() == 0 {
        return None;
    }
    let (w, h) = (img.width() as f32, img.height() as f32);
    let px = ((x / 256.0) * w).round() as u32;
    let py = ((y / 256.0) * h).round() as u32;
    if px >= img.width() || py >= img.height() {
        return None;
    }
    let seed = img.get_pixel(px, py).0;
    if seed[3] == 0 {
        return None;
    }
    let region = flood_region(img, px, py, seed, max_sq_for(tolerance));

    let mut mask = vec![0u8; 256 * 256];
    for my in 0..256u32 {
        let y0 = (my * img.height() / 256).min(img.height() - 1);
        let y1 = ((my + 1) * img.height() / 256 + 1).min(img.height());
        for mx in 0..256u32 {
            let x0 = (mx * img.width() / 256).min(img.width() - 1);
            let x1 = ((mx + 1) * img.width() / 256 + 1).min(img.width());
            let mut hits = 0u32;
            let mut total = 0u32;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    if region[(yy * img.width() + xx) as usize] {
                        hits += 1;
                    }
                    total += 1;
                }
            }
            if total > 0 && hits * 100 >= total * 50 {
                mask[(my * 256 + mx) as usize] = 255;
            }
        }
    }
    Some(mask)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::config::WandPoint;

    fn make_image() -> RgbaImage {
        let mut img = RgbaImage::new(8, 8);
        for y in 0..8 {
            for x in 0..8 {
                let c = if x < 4 { [200, 20, 20, 255] } else { [20, 20, 200, 255] };
                img.put_pixel(x, y, image::Rgba(c));
            }
        }
        img
    }

    fn stroke(point: (f32, f32), tolerance: f32) -> WandStroke {
        WandStroke {
            points: vec![WandPoint { x: point.0, y: point.1 }],
            tolerance,
        }
    }

    #[test]
    fn wand_erases_connected_similar_region() {
        let mut img = make_image();
        apply_erasures(&mut img, &[stroke((32.0, 32.0), 10.0)]);
        // Left half (red, seed) erased, right half (blue) untouched.
        assert_eq!(img.get_pixel(0, 0).0[3], 0);
        assert_eq!(img.get_pixel(3, 7).0[3], 0);
        assert_eq!(img.get_pixel(4, 0).0[3], 255);
        assert_eq!(img.get_pixel(7, 7).0[3], 255);
    }

    #[test]
    fn wand_does_not_bleed_into_dissimilar_region() {
        let mut img = make_image();
        apply_erasures(&mut img, &[stroke((128.0, 128.0), 5.0)]);
        // Blue seed, low tolerance: nothing may bleed into red pixels.
        assert_eq!(img.get_pixel(7, 7).0[3], 0);
        assert_eq!(img.get_pixel(0, 0).0[3], 255);
    }
}