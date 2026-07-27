use image::RgbaImage;

use super::color::srgb_channel_to_linear;
use super::composite::LinearPremultipliedImage;
use super::pipeline::ValidatedRenderConfig;

pub fn render_foreground(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
    mask: &super::mask::AlphaMask,
) -> LinearPremultipliedImage {
    let size = 256usize;
    let mut result = LinearPremultipliedImage::transparent(256, 256);
    let sw = source.width() as f32;
    let sh = source.height() as f32;
    let shape_size = 224.0f32;
    let fit_scale = (shape_size / sw).min(shape_size / sh);
    let scale = fit_scale * config.foreground_scale_percent / 100.0;
    let cx = 128.0;
    let cy = 128.0;
    let ox = config.foreground_offset_x;
    let oy = config.foreground_offset_y;
    let rotation_rad = -config.foreground_rotation_degrees.to_radians();
    let cos_r = rotation_rad.cos();
    let sin_r = rotation_rad.sin();
    let src_cx = sw / 2.0;
    let src_cy = sh / 2.0;

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

            if sx < 0.0 || sx >= (sw - 1.0) || sy < 0.0 || sy >= (sh - 1.0) {
                continue;
            }

            let ix = sx as u32;
            let iy = sy as u32;
            let fx = sx - ix as f32;
            let fy = sy - iy as f32;

            if ix >= source.width() - 1 || iy >= source.height() - 1 {
                continue;
            }

            // Bilinear interpolation in premultiplied alpha
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

fn get_premul(img: &RgbaImage, x: u32, y: u32) -> [f32; 4] {
    let p = img.get_pixel(x, y).0;
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
