use image::RgbaImage;

use super::color::{parse_hex_rgba, LinearRgba};
use super::composite::{over, LinearPremultipliedImage};
use super::mask::{generate_shape_mask, AlphaMask};
use super::pipeline::ValidatedRenderConfig;
use crate::domain::config::RenderConfig;
use crate::error::app_error::AppError;

/// Glass-texture foreground renderer.
///
/// The glass mode does ONE thing: turn the source image into a black & white
/// glass relief (max RGB channel keeps bright glyphs white, SDF bevel, normals,
/// soft multi-light shading, Fresnel edges, AO and subtle screen-space
/// refraction). Everything else — shape, backplate, stroke, edge gloss and the
/// outer shadow — is handled by the classic pipeline with the user's own
/// parameters, so nothing is hardcoded.

#[derive(Debug, Clone)]
pub struct ValidatedGlassRenderConfig {
    pub color_retention: f32,
    pub bevel_radius: f32,
    pub normal_strength: f32,
    pub specular_strength: f32,
    pub fresnel_strength: f32,
    pub ao_strength: f32,
    pub contrast_strength: f32,
}

pub fn validate_glass(config: &RenderConfig) -> Result<ValidatedGlassRenderConfig, AppError> {
    fn check(v: f32, lo: f32, hi: f32, name: &str) -> Result<f32, AppError> {
        if !v.is_finite() || !(lo..=hi).contains(&v) {
            return Err(AppError::InvalidArgument(format!(
                "{name} must be in {lo}..={hi}"
            )));
        }
        Ok(v)
    }
    let g = &config.glass_render;
    Ok(ValidatedGlassRenderConfig {
        color_retention: check(g.color_retention, 0.0, 1.0, "glass colorRetention")?,
        bevel_radius: check(g.bevel_radius, 0.0, 0.5, "glass bevelRadius")?,
        normal_strength: check(g.normal_strength, 0.0, 2.0, "glass normalStrength")?,
        specular_strength: check(g.specular_strength, 0.0, 1.0, "glass specularStrength")?,
        fresnel_strength: check(g.fresnel_strength, 0.0, 1.0, "glass fresnelStrength")?,
        ao_strength: check(g.ao_strength, 0.0, 1.0, "glass aoStrength")?,
        contrast_strength: check(g.contrast_strength, 0.0, 1.0, "glass contrastStrength")?,
    })
}

const SIZE: usize = 256;
const CENTER: f32 = 128.0;
const SUBJECT_MAX_H: f32 = 0.75;
/// Amplification of the height-field gradient when deriving normals, so the
/// gentle bevel slopes produce a clearly visible relief.
const NORMAL_AMPLIFY: f32 = 5.5;

pub fn render_glass(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
) -> Result<RgbaImage, AppError> {
    let size = 256u32;
    let glass = config
        .glass
        .as_ref()
        .expect("glass config must be present when rendering glass");
    let mask = generate_shape_mask(size, config);
    let mut body = LinearPremultipliedImage::transparent(size, size);

    // Backplate: classic gradient/solid, driven by the user's parameters.
    if config.backplate_type != crate::domain::config::BackplateType::None {
        over(&mut body, super::pipeline::render_backplate(&mask, config, size));
    }

    // Glass foreground: monochrome + relief, refracted against the backplate.
    let foreground = render_glass_foreground(source, config, glass, &mask, &body)?;
    over(&mut body, foreground);

    // Stroke: classic.
    if config.stroke_width > 0.0 {
        over(&mut body, super::stroke::render_inner_stroke(&mask, config));
    }

    // Edge gloss: classic (base ring + directional highlights).
    if config.gloss_enabled && config.gloss_width > 0.0 && config.gloss_strength > 0.0 {
        over(&mut body, super::gloss::render_edge_gloss(&mask, config)?);
    }

    // Outer shadow: classic, from the user's shadow parameters.
    let silhouette = if config.backplate_type != crate::domain::config::BackplateType::None {
        mask.clone()
    } else {
        body.alpha_plane()
    };
    let mut result = if config.outer_shadow_enabled {
        super::shadow::render_shadow(&silhouette, config)
    } else {
        LinearPremultipliedImage::transparent(size, size)
    };
    over(&mut result, body);
    Ok(result.into_srgb_rgba8())
}

/// The black & white relief foreground, placed through the classic foreground
/// transform (fit / scale / offset / rotation / shape clipping).
fn render_glass_foreground(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
    glass: &ValidatedGlassRenderConfig,
    mask: &AlphaMask,
    backplate: &LinearPremultipliedImage,
) -> Result<LinearPremultipliedImage, AppError> {
    let mono = to_grayscale(source);
    let subj = super::transform::render_foreground(&mono, config, mask);
    let subj_color = super::transform::render_foreground(source, config, mask);
    let subj_mask = alpha_of(&subj);
    let subj_sdf = chamfer_edt(&subj_mask);
    let subj_h = subject_height(&subj_sdf, &subj_color, glass.bevel_radius);
    // Slightly smooth the relief so thin shapes don't show jagged shading.
    let subj_h = super::hq::gaussian_blur(&subj_h, SIZE, SIZE, 1.0);
    let subj_n = normals_from_height(&subj_h, glass.normal_strength);
    let ao = ambient_occlusion(&subj_h, glass.ao_strength);
    // Soft inner shadow just inside the silhouette, lifting it off the plate.
    let inner_shadow: Vec<f32> = subj_sdf
        .iter()
        .map(|&d| 0.2 * (1.0 - smoothstep01((d / 5.0).clamp(0.0, 1.0))))
        .collect();

    let mut lit = shade(&subj_n, |x, y| {
        let idx = y * SIZE + x;
        let a = subj.data[idx][3];
        if a <= 0.0 {
            return [0.0, 0.0, 0.0];
        }
        let gray = [
            subj.data[idx][0] / a,
            subj.data[idx][1] / a,
            subj.data[idx][2] / a,
        ];
        let orig = [
            subj_color.data[idx][0] / a,
            subj_color.data[idx][1] / a,
            subj_color.data[idx][2] / a,
        ];
        // Contrast boost applied to the source values before any shading:
        // pivots around 0.5 so highlights brighten and shadows deepen,
        // making the glass relief read more clearly.
        let k = 1.0 + glass.contrast_strength;
        let cont = |c: f32| ((c - 0.5) * k + 0.5).clamp(0.0, 1.0);
        let gray = [cont(gray[0]), cont(gray[1]), cont(gray[2])];
        let orig = [cont(orig[0]), cont(orig[1]), cont(orig[2])];
        let mut c = [
            gray[0] + (orig[0] - gray[0]) * glass.color_retention,
            gray[1] + (orig[1] - gray[1]) * glass.color_retention,
            gray[2] + (orig[2] - gray[2]) * glass.color_retention,
        ];
        // Dark regions read as smoky translucent glass.
        let v = gray[0].max(gray[1]).max(gray[2]);
        let dim = 0.9 + 0.1 * v;
        c[0] *= dim;
        c[1] *= dim;
        c[2] *= dim;
        c
    }, glass);

    // AO and inner shadow per pixel.
    for (i, p) in lit.iter_mut().enumerate() {
        if subj_mask[i] <= 0.0 {
            continue;
        }
        let occluded = (ao[i] - inner_shadow[i]).clamp(0.15, 1.0);
        p[0] *= occluded;
        p[1] *= occluded;
        p[2] *= occluded;
    }

    // Screen-space refraction: sample the blurred backplate behind the subject.
    let plate_colors = premultiplied_rgb(backplate);
    let plate_blur = blur_rgb(&plate_colors, 2.0);
    let mut subject = LinearPremultipliedImage::transparent(SIZE as u32, SIZE as u32);
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = y * SIZE + x;
            let a = subj_mask[idx];
            if a <= 0.0 {
                continue;
            }
            let h = subj_h[idx];
            let nx = subj_n[idx][0];
            let ny = subj_n[idx][1];
            let off = (0.6 + 1.6 * h) * 2.2;
            let sx = ((x as f32 + 0.5 + nx * off).clamp(0.0, SIZE as f32 - 1.0)) as usize;
            let sy = ((y as f32 + 0.5 + ny * off).clamp(0.0, SIZE as f32 - 1.0)) as usize;
            let base = plate_blur[sy * SIZE + sx];
            let v = lit[idx][0].max(lit[idx][1]).max(lit[idx][2]);
            let transmission = (0.22 + 0.1 * (1.0 - v)).min(0.35);
            let c = [
                lit[idx][0] * (1.0 - transmission) + base[0] * transmission,
                lit[idx][1] * (1.0 - transmission) + base[1] * transmission,
                lit[idx][2] * (1.0 - transmission) + base[2] * transmission,
            ];
            // Translucent glass: the whole subject lets a little of the plate
            // show through.
            let glass_alpha = a * 0.78;
            subject.data[idx] = [c[0] * glass_alpha, c[1] * glass_alpha, c[2] * glass_alpha, glass_alpha];
        }
    }
    add_noise_to_img(&mut subject, &subj_mask);
    Ok(subject)
}

/// Black & white conversion: max RGB channel (HSV value) keeps bright colored
/// glyphs white, matching the mono glass look of the reference icons.
fn to_grayscale(source: &RgbaImage) -> RgbaImage {
    let mut mono = source.clone();
    for pixel in mono.pixels_mut() {
        let [r, g, b, a] = pixel.0;
        let v = r.max(g).max(b);
        pixel.0 = [v, v, v, a];
    }
    mono
}

fn alpha_of(img: &LinearPremultipliedImage) -> Vec<f32> {
    img.data.iter().map(|p| p[3]).collect()
}

fn premultiplied_rgb(img: &LinearPremultipliedImage) -> Vec<[f32; 3]> {
    img.data.iter().map(|p| [p[0], p[1], p[2]]).collect()
}

/// Beveled relief: height ramps up smoothly from the boundary over the bevel.
fn height_from_sdf(sdf: &[f32], bevel: f32, max_h: f32) -> Vec<f32> {
    sdf.iter()
        .map(|&d| {
            let t = (d / bevel).clamp(0.0, 1.0);
            max_h * t * t * (3.0 - 2.0 * t)
        })
        .collect()
}

/// Subject relief: beveled slab with height offsets for the different color
/// regions (light areas raised, dark areas lowered), smoothed so the height
/// map has no visible steps.
fn subject_height(sdf: &[f32], color: &LinearPremultipliedImage, bevel_radius: f32) -> Vec<f32> {
    let bevel = (bevel_radius * SIZE as f32).max(1.0);
    let mut h = height_from_sdf(sdf, bevel, SUBJECT_MAX_H);
    let mut region = vec![0.0f32; SIZE * SIZE];
    for i in 0..SIZE * SIZE {
        let a = color.data[i][3];
        if a <= 0.0 {
            region[i] = 0.0;
            continue;
        }
        let v = [
            color.data[i][0] / a,
            color.data[i][1] / a,
            color.data[i][2] / a,
        ]
        .iter()
        .cloned()
        .fold(0.0f32, f32::max);
        region[i] = (v - 0.5) * 0.26;
    }
    let region = super::hq::gaussian_blur(&region, SIZE, SIZE, 4.0);
    for i in 0..SIZE * SIZE {
        h[i] = (h[i] + region[i]).clamp(0.01, 0.95);
    }
    h
}

/// Normal map from the height field (screen y down, z up). The gradient is
/// amplified so the gentle bevel slopes produce a clearly visible relief.
fn normals_from_height(h: &[f32], strength: f32) -> Vec<[f32; 3]> {
    let s = strength * NORMAL_AMPLIFY;
    let mut out = vec![[0.0f32; 3]; SIZE * SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let l = h[(y * SIZE + x.saturating_sub(1)) as usize];
            let r = h[(y * SIZE + (x + 1).min(SIZE - 1)) as usize];
            let u = h[((y.saturating_sub(1)) * SIZE + x) as usize];
            let d = h[((y + 1).min(SIZE - 1) * SIZE + x) as usize];
            let dx = (r - l) * 0.5 * s;
            let dy = (d - u) * 0.5 * s;
            let len = (dx * dx + dy * dy + 1.0).sqrt();
            out[y * SIZE + x] = [-dx / len, -dy / len, 1.0 / len];
        }
    }
    out
}

/// Multi-light shading: ambient + key (top-left) + fill (right) diffuse,
/// soft Blinn-Phong specular sheen, and Schlick Fresnel edge reflection.
fn shade(
    normals: &[[f32; 3]],
    base: impl Fn(usize, usize) -> [f32; 3],
    glass: &ValidatedGlassRenderConfig,
) -> Vec<[f32; 3]> {
    let l1 = normalize([-0.45, -0.45, 0.77]);
    let l2 = normalize([0.65, 0.05, 0.55]);
    let v = [0.0f32, 0.0, 1.0];
    let h = normalize([l1[0] + v[0], l1[1] + v[1], l1[2] + v[2]]);
    let f0 = 0.04f32;
    let mut out = vec![[0.0f32; 3]; SIZE * SIZE];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let idx = y * SIZE + x;
            let n = normals[idx];
            let b = base(x, y);
            if b[0] <= 0.0 && b[1] <= 0.0 && b[2] <= 0.0 {
                continue;
            }
            let d1 = (n[0] * l1[0] + n[1] * l1[1] + n[2] * l1[2]).max(0.0);
            let d2 = (n[0] * l2[0] + n[1] * l2[1] + n[2] * l2[2]).max(0.0);
            let ndh = (n[0] * h[0] + n[1] * h[1] + n[2] * h[2]).max(0.0);
            let spec = ndh.powf(32.0) * glass.specular_strength * 0.9;
            let nv = n[2].clamp(0.0, 1.0);
            let fres = f0 + (1.0 - f0) * (1.0 - nv).powi(5);
            // The edge reflection is directional: strongest where the surface
            // faces the light (top), so it doesn't draw a uniform white
            // outline around the whole silhouette.
            let fres_dir = 0.3 + 0.7 * (-n[1]).clamp(0.0, 1.0);
            let fres = fres * fres_dir;
            let light = 0.22 + d1 * 0.85 + d2 * 0.14;
            let lit = light.min(1.35);
            let boost = 1.15;
            let refl = (spec + fres * glass.fresnel_strength) * boost;
            // The edge reflection is tinted by the material color: bright
            // glyphs keep a white rim, dark glyphs get a dim gray one instead
            // of a white outline.
            let c = [
                (b[0] * lit + refl * (b[0] * 0.5 + 0.5)).min(1.15),
                (b[1] * lit + refl * (b[1] * 0.5 + 0.5)).min(1.15),
                (b[2] * lit + refl * (b[2] * 0.5 + 0.5)).min(1.15),
            ];
            out[idx] = c;
        }
    }
    out
}

/// AO from the height field: crevices (blurred height above the local height)
/// get darkened.
fn ambient_occlusion(h: &[f32], strength: f32) -> Vec<f32> {
    let blurred = super::hq::gaussian_blur(h, SIZE, SIZE, 3.0);
    h.iter()
        .zip(blurred.iter())
        .map(|(&hi, &bi)| (1.0 - strength * (bi - hi).max(0.0)).clamp(0.3, 1.0))
        .collect()
}

fn blur_rgb(img: &[[f32; 3]], sigma: f32) -> Vec<[f32; 3]> {
    let mut channels = vec![0.0f32; SIZE * SIZE];
    let mut out = vec![[0.0f32; 3]; SIZE * SIZE];
    for c in 0..3 {
        for i in 0..SIZE * SIZE {
            channels[i] = img[i][c];
        }
        let blurred = super::hq::gaussian_blur(&channels, SIZE, SIZE, sigma);
        for i in 0..SIZE * SIZE {
            out[i][c] = blurred[i];
        }
    }
    out
}

fn add_noise_to_img(img: &mut LinearPremultipliedImage, mask: &[f32]) {
    for i in 0..SIZE * SIZE {
        if mask[i] <= 0.0 {
            continue;
        }
        let n = (hash2(i as u32) as f32 / 255.0 - 0.5) * 0.006 * mask[i];
        img.data[i][0] += n;
        img.data[i][1] += n;
        img.data[i][2] += n;
    }
}

fn smoothstep01(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn hash2(i: u32) -> u32 {
    let mut h = i.wrapping_mul(0x45d9_f3b);
    h ^= h >> 16;
    h = h.wrapping_mul(0x45d9_f3b);
    h ^= h >> 16;
    h & 0xFF
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1e-6);
    [v[0] / len, v[1] / len, v[2] / len]
}

/// Two-pass chamfer (3-4 weights) Euclidean distance transform: positive
/// inside the mask, negative outside.
fn chamfer_edt(mask: &[f32]) -> Vec<f32> {
    let inside = edt_pass(mask);
    let inverted: Vec<f32> = mask.iter().map(|&a| (1.0 - a).clamp(0.0, 1.0)).collect();
    let outside = edt_pass(&inverted);
    inside
        .iter()
        .zip(outside.iter())
        .map(|(&i, &o)| i - o)
        .collect()
}

fn edt_pass(mask: &[f32]) -> Vec<f32> {
    let inf = 1e9f32;
    let mut d = vec![inf; SIZE * SIZE];
    for i in 0..SIZE * SIZE {
        if mask[i] < 0.5 {
            d[i] = 0.0;
        }
    }
    // Forward pass.
    for y in 0..SIZE {
        for x in 0..SIZE {
            let i = y * SIZE + x;
            let mut best = d[i];
            if y > 0 {
                best = best.min(d[i - SIZE] + 1.0);
                if x > 0 {
                    best = best.min(d[i - SIZE - 1] + 1.414);
                }
                if x + 1 < SIZE {
                    best = best.min(d[i - SIZE + 1] + 1.414);
                }
            }
            if x > 0 {
                best = best.min(d[i - 1] + 1.0);
            }
            d[i] = best;
        }
    }
    // Backward pass.
    for y in (0..SIZE).rev() {
        for x in (0..SIZE).rev() {
            let i = y * SIZE + x;
            let mut best = d[i];
            if y + 1 < SIZE {
                best = best.min(d[i + SIZE] + 1.0);
                if x > 0 {
                    best = best.min(d[i + SIZE - 1] + 1.414);
                }
                if x + 1 < SIZE {
                    best = best.min(d[i + SIZE + 1] + 1.414);
                }
            }
            if x + 1 < SIZE {
                best = best.min(d[i + 1] + 1.0);
            }
            d[i] = best;
        }
    }
    d
}

#[allow(dead_code)]
fn _unused(_: LinearRgba) {}
