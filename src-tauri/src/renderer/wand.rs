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

fn flood_fill_erase(img: &mut RgbaImage, sx: u32, sy: u32, seed: [u8; 4], max_sq: f32) {
    let (w, h) = img.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut queue = VecDeque::new();
    queue.push_back((sx, sy));
    visited[(sy * w + sx) as usize] = true;

    while let Some((x, y)) = queue.pop_front() {
        let pixel = img.get_pixel(x, y).0;
        if pixel[3] == 0 || dist_sq(pixel, seed) > max_sq {
            continue;
        }
        img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
        for (dx_, dy_) in NEIGHBORS {
            let nx = x as i64 + dx_;
            let ny = y as i64 + dy_;
            if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                continue;
            }
            let idx = (ny as u64 * w as u64 + nx as u64) as usize;
            if !visited[idx] {
                visited[idx] = true;
                queue.push_back((nx as u32, ny as u32));
            }
        }
    }
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