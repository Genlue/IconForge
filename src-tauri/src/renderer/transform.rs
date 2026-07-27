use image::RgbaImage;

use super::color::srgb_channel_to_linear;
use super::composite::LinearPremultipliedImage;
use super::pipeline::ValidatedRenderConfig;
use crate::domain::config::ForegroundFit;

pub fn render_foreground(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
    mask: &super::mask::AlphaMask,
) -> LinearPremultipliedImage {
    let size = 256usize;
    let mut result = LinearPremultipliedImage::transparent(256, 256);
    let content = alpha_content_bounds(source);
    // Normalize every source to its visible alpha bounds before fitting. PE
    // and shell extraction often returns different transparent canvases for
    // otherwise identical icons; using the same bounds makes desktop sizing
    // consistent across imported file types.
    let content_width = (content.max_x - content.min_x + 1) as f32;
    let content_height = (content.max_y - content.min_y + 1) as f32;
    let shape_size = 256.0 - 2.0 * config.canvas_inset;
    let scale_x = shape_size / content_width;
    let scale_y = shape_size / content_height;
    let fit_scale = match config.foreground_fit {
        ForegroundFit::Contain => scale_x.min(scale_y),
        ForegroundFit::Cover => scale_x.max(scale_y),
    };
    let scale = fit_scale * config.foreground_scale_percent / 100.0;
    let cx = 128.0;
    let cy = 128.0;
    let ox = config.foreground_offset_x;
    let oy = config.foreground_offset_y;
    let rotation_rad = -config.foreground_rotation_degrees.to_radians();
    let cos_r = rotation_rad.cos();
    let sin_r = rotation_rad.sin();
    let src_cx = (content.min_x + content.max_x + 1) as f32 / 2.0;
    let src_cy = (content.min_y + content.max_y + 1) as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            // Inverse transform
            let dx = px - (cx + ox);
            let dy = py - (cy + oy);
            let rx = dx * cos_r - dy * sin_r;
            let ry = dx * sin_r + dy * cos_r;
            let sx = rx / scale + src_cx;
            let sy = ry / scale + src_cy;

            if sx < -0.5
                || sx >= source.width() as f32 - 0.5
                || sy < -0.5
                || sy >= source.height() as f32 - 0.5
            {
                continue;
            }

            // Bilinear interpolation in premultiplied alpha
            let ix = (sx - 0.5).floor() as i32;
            let iy = (sy - 0.5).floor() as i32;
            let fx = sx - (ix as f32 + 0.5);
            let fy = sy - (iy as f32 + 0.5);
            let p00 = get_premul(source, ix, iy);
            let p10 = get_premul(source, ix + 1, iy);
            let p01 = get_premul(source, ix, iy + 1);
            let p11 = get_premul(source, ix + 1, iy + 1);

            let top = lerp_premul(p00, p10, fx);
            let bot = lerp_premul(p01, p11, fx);
            let sampled = lerp_premul(top, bot, fy);

            let idx = y * size + x;
            let coverage = mask[idx];
            if coverage <= 0.0 {
                continue;
            }

            result.data[idx][0] = sampled[0] * coverage;
            result.data[idx][1] = sampled[1] * coverage;
            result.data[idx][2] = sampled[2] * coverage;
            result.data[idx][3] = sampled[3] * coverage;
        }
    }

    result
}

#[derive(Debug, Clone, Copy)]
struct ContentBounds {
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
}

fn alpha_content_bounds(img: &RgbaImage) -> ContentBounds {
    let mut min_x = img.width();
    let mut min_y = img.height();
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for (x, y, pixel) in img.enumerate_pixels() {
        // Ignore negligible antialiasing residue so transparent padding does
        // not shrink the useful foreground area.
        if pixel.0[3] < 4 {
            continue;
        }
        found = true;
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }

    if found {
        ContentBounds {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    } else {
        ContentBounds {
            min_x: 0,
            min_y: 0,
            max_x: img.width().saturating_sub(1),
            max_y: img.height().saturating_sub(1),
        }
    }
}

fn get_premul(img: &RgbaImage, x: i32, y: i32) -> [f32; 4] {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return [0.0; 4];
    }
    let p = img.get_pixel(x as u32, y as u32).0;
    let r = srgb_channel_to_linear(p[0] as f32 / 255.0);
    let g = srgb_channel_to_linear(p[1] as f32 / 255.0);
    let b = srgb_channel_to_linear(p[2] as f32 / 255.0);
    let a = p[3] as f32 / 255.0;
    [r * a, g * a, b * a, a]
}

fn lerp_premul(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}
