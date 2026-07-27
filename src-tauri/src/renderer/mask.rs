use super::pipeline::ValidatedRenderConfig;
use crate::domain::config::IconShape;

pub type AlphaMask = Vec<f32>;

const SUB_SAMPLES: usize = 4;

pub fn generate_shape_mask(size: u32, config: &ValidatedRenderConfig) -> AlphaMask {
    let pixel_count = (size * size) as usize;
    let mut mask = vec![0.0f32; pixel_count];

    let inset = super::BASE_SHAPE_INSET;
    let half_size = (size as f32) / 2.0;

    for y in 0..size {
        for x in 0..size {
            let mut coverage = 0u32;
            for sy in 0..SUB_SAMPLES {
                for sx in 0..SUB_SAMPLES {
                    let px = x as f32 + (sx as f32 + 0.5) / SUB_SAMPLES as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SUB_SAMPLES as f32;
                    if is_inside_shape(px, py, size, inset, half_size, config) {
                        coverage += 1;
                    }
                }
            }
            let idx = (y * size + x) as usize;
            mask[idx] = coverage as f32 / (SUB_SAMPLES * SUB_SAMPLES) as f32;
        }
    }

    mask
}

pub fn generate_inset_shape_mask(
    size: u32,
    config: &ValidatedRenderConfig,
    inset: f32,
) -> AlphaMask {
    let pixel_count = (size * size) as usize;
    let mut mask = vec![0.0f32; pixel_count];

    let half_size = (size as f32) / 2.0;

    for y in 0..size {
        for x in 0..size {
            let mut coverage = 0u32;
            for sy in 0..SUB_SAMPLES {
                for sx in 0..SUB_SAMPLES {
                    let px = x as f32 + (sx as f32 + 0.5) / SUB_SAMPLES as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SUB_SAMPLES as f32;
                    if is_inside_shape(px, py, size, inset, half_size, config) {
                        coverage += 1;
                    }
                }
            }
            let idx = (y * size + x) as usize;
            mask[idx] = coverage as f32 / (SUB_SAMPLES * SUB_SAMPLES) as f32;
        }
    }

    mask
}

fn is_inside_shape(
    px: f32,
    py: f32,
    size: u32,
    inset: f32,
    half_size: f32,
    config: &ValidatedRenderConfig,
) -> bool {
    match config.shape {
        IconShape::Rectangle => {
            px >= inset && px <= (size as f32 - inset) && py >= inset && py <= (size as f32 - inset)
        }
        IconShape::RoundedRectangle => {
            let r = config.corner_radius.min(112.0);
            let inner_half = half_size - inset;
            let cx = half_size;
            let cy = half_size;
            let qx = (px - cx).abs() - (inner_half - r);
            let qy = (py - cy).abs() - (inner_half - r);
            let distance =
                (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.min(qy).min(0.0) - r;
            distance <= 0.0
        }
        IconShape::Squircle => {
            let cx = half_size;
            let cy = half_size;
            let a = half_size - inset;
            let b = half_size - inset;
            let n = config.squircle_exponent;
            let fx = ((px - cx).abs() / a).powf(n);
            let fy = ((py - cy).abs() / b).powf(n);
            fx + fy <= 1.0
        }
    }
}
