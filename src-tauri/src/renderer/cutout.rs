use std::collections::VecDeque;

use image::{Rgba, RgbaImage};

pub fn remove_border_background(source: &RgbaImage, tolerance: f32, feather: f32) -> RgbaImage {
    if source.width() == 0 || source.height() == 0 {
        return source.clone();
    }
    let width = source.width();
    let height = source.height();
    let background = estimate_border_color(source);
    let mut connected = vec![false; (width * height) as usize];
    let mut queue = VecDeque::new();

    for x in 0..width {
        queue.push_back((x, 0));
        queue.push_back((x, height - 1));
    }
    for y in 0..height {
        queue.push_back((0, y));
        queue.push_back((width - 1, y));
    }

    let threshold = tolerance.clamp(0.0, 100.0) * 4.42;
    let feather_range = feather.clamp(0.0, 32.0) * 4.42;
    while let Some((x, y)) = queue.pop_front() {
        let idx = (y * width + x) as usize;
        if connected[idx]
            || color_distance(source.get_pixel(x, y), background) > threshold + feather_range
        {
            continue;
        }
        connected[idx] = true;
        if x > 0 {
            queue.push_back((x - 1, y));
        }
        if x + 1 < width {
            queue.push_back((x + 1, y));
        }
        if y > 0 {
            queue.push_back((x, y - 1));
        }
        if y + 1 < height {
            queue.push_back((x, y + 1));
        }
    }

    let mut result = source.clone();
    for (x, y, pixel) in result.enumerate_pixels_mut() {
        if !connected[(y * width + x) as usize] {
            continue;
        }
        let distance = color_distance(pixel, background);
        let keep = if feather_range <= 0.0 {
            if distance <= threshold {
                0.0
            } else {
                1.0
            }
        } else {
            ((distance - threshold) / feather_range).clamp(0.0, 1.0)
        };
        pixel.0[3] = (pixel.0[3] as f32 * keep).round() as u8;
        if pixel.0[3] == 0 {
            pixel.0[..3].fill(0);
        }
    }
    result
}

fn estimate_border_color(source: &RgbaImage) -> Rgba<u8> {
    let mut channels = [Vec::new(), Vec::new(), Vec::new()];
    for x in 0..source.width() {
        for y in [0, source.height() - 1] {
            let p = source.get_pixel(x, y).0;
            for c in 0..3 {
                channels[c].push(p[c]);
            }
        }
    }
    for y in 0..source.height() {
        for x in [0, source.width() - 1] {
            let p = source.get_pixel(x, y).0;
            for c in 0..3 {
                channels[c].push(p[c]);
            }
        }
    }
    for values in &mut channels {
        values.sort_unstable();
    }
    Rgba([
        channels[0][channels[0].len() / 2],
        channels[1][channels[1].len() / 2],
        channels[2][channels[2].len() / 2],
        255,
    ])
}

fn color_distance(pixel: &Rgba<u8>, background: Rgba<u8>) -> f32 {
    let dr = pixel.0[0] as f32 - background.0[0] as f32;
    let dg = pixel.0[1] as f32 - background.0[1] as f32;
    let db = pixel.0[2] as f32 - background.0[2] as f32;
    (dr * dr + dg * dg + db * db).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_border_connected_background_but_keeps_subject() {
        let mut source = RgbaImage::from_pixel(16, 16, Rgba([255, 255, 255, 255]));
        for y in 4..12 {
            for x in 4..12 {
                source.put_pixel(x, y, Rgba([20, 80, 220, 255]));
            }
        }
        let result = remove_border_background(&source, 10.0, 0.0);
        assert_eq!(result.get_pixel(0, 0).0[3], 0);
        assert_eq!(result.get_pixel(8, 8).0[3], 255);
    }
}
