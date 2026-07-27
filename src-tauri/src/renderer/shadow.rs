use super::color::{parse_hex_rgba, LinearRgba};
use super::composite::LinearPremultipliedImage;
use super::mask::AlphaMask;
use super::pipeline::ValidatedRenderConfig;

pub fn dilate_alpha(mask: &AlphaMask, width: u32, height: u32, radius: f32) -> AlphaMask {
    if radius <= 0.0 {
        return mask.clone();
    }
    let r = radius.ceil() as i32;
    let mut dilated = vec![0.0f32; mask.len()];

    for y in 0..height {
        for x in 0..width {
            let mut max_val = 0.0f32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy > r * r {
                        continue;
                    }
                    let sx = x as i32 + dx;
                    let sy = y as i32 + dy;
                    if sx >= 0 && sx < width as i32 && sy >= 0 && sy < height as i32 {
                        let idx = (sy as u32 * width + sx as u32) as usize;
                        max_val = max_val.max(mask[idx]);
                    }
                }
            }
            let idx = (y * width + x) as usize;
            dilated[idx] = max_val;
        }
    }

    dilated
}

pub fn gaussian_blur_alpha(
    mask: &AlphaMask,
    width: u32,
    height: u32,
    blur_radius: f32,
) -> AlphaMask {
    if blur_radius <= 0.0 {
        return mask.clone();
    }

    let sigma = (blur_radius / 2.0).max(0.01);
    let kernel_radius = (3.0 * sigma).ceil() as i32;
    let kernel_size = (2 * kernel_radius + 1) as usize;

    // Compute 1D kernel
    let mut kernel = vec![0.0f32; kernel_size];
    let mut sum = 0.0f32;
    for i in -kernel_radius..=kernel_radius {
        let val = (-(i * i) as f32 / (2.0 * sigma * sigma)).exp();
        let idx = (i + kernel_radius) as usize;
        kernel[idx] = val;
        sum += val;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    // Horizontal pass
    let mut temp = vec![0.0f32; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sx = x as i32 + ki as i32 - kernel_radius;
                if sx >= 0 && sx < width as i32 {
                    let idx = (y * width + sx as u32) as usize;
                    acc += mask.get(idx).copied().unwrap_or(0.0) * kv;
                }
            }
            let idx = (y * width + x) as usize;
            temp[idx] = acc;
        }
    }

    // Vertical pass
    let mut result = vec![0.0f32; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sy = y as i32 + ki as i32 - kernel_radius;
                if sy >= 0 && sy < height as i32 {
                    let idx = (sy as u32 * width + x) as usize;
                    acc += temp.get(idx).copied().unwrap_or(0.0) * kv;
                }
            }
            let idx = (y * width + x) as usize;
            result[idx] = acc;
        }
    }

    result
}

pub fn render_shadow(mask: &AlphaMask, config: &ValidatedRenderConfig) -> LinearPremultipliedImage {
    let size = 256u32;
    let mut result = LinearPremultipliedImage::transparent(size, size);

    let shadow_color = parse_hex_rgba(&config.outer_shadow_color).unwrap_or(LinearRgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });

    // Dilate (spread)
    let dilated = dilate_alpha(mask, size, size, config.outer_shadow_spread);

    // Blur
    let blurred = gaussian_blur_alpha(&dilated, size, size, config.outer_shadow_blur_radius);

    // Translate by offset
    let off_x = config.outer_shadow_offset_x;
    let off_y = config.outer_shadow_offset_y;

    for y in 0..size {
        for x in 0..size {
            let sx = x as f32 - off_x;
            let sy = y as f32 - off_y;
            let alpha = sample_bilinear(&blurred, size, size, sx, sy);
            if alpha <= 0.0 {
                continue;
            }
            let idx = (y * size + x) as usize;
            result.data[idx][0] = shadow_color.r * alpha;
            result.data[idx][1] = shadow_color.g * alpha;
            result.data[idx][2] = shadow_color.b * alpha;
            result.data[idx][3] = alpha;
        }
    }

    result
}

fn sample_bilinear(mask: &AlphaMask, width: u32, height: u32, x: f32, y: f32) -> f32 {
    if x < -0.5 || x >= width as f32 - 0.5 || y < -0.5 || y >= height as f32 - 0.5 {
        return 0.0;
    }
    let ix = (x - 0.5).floor() as i32;
    let iy = (y - 0.5).floor() as i32;
    let fx = x - (ix as f32 + 0.5);
    let fy = y - (iy as f32 + 0.5);

    let sample = |px: i32, py: i32| -> f32 {
        if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
            mask[(py as u32 * width + px as u32) as usize]
        } else {
            0.0
        }
    };

    let v00 = sample(ix, iy);
    let v10 = sample(ix + 1, iy);
    let v01 = sample(ix, iy + 1);
    let v11 = sample(ix + 1, iy + 1);

    let top = v00 + (v10 - v00) * fx.max(0.0).min(1.0);
    let bot = v01 + (v11 - v01) * fx.max(0.0).min(1.0);
    top + (bot - top) * fy.max(0.0).min(1.0)
}
