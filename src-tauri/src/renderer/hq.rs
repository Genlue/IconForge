use image::imageops::FilterType;
use image::{GenericImageView, RgbaImage};

use crate::domain::config::{HqPanelShape, HqShadowMode, RenderConfig};
use crate::error::app_error::AppError;

pub const HQ_SIZE: u32 = 256;
const RAYS: usize = 360;
const ANG_SMOOTH: i32 = 10;

#[derive(Debug, Clone, Copy)]
pub struct ValidatedHqRenderConfig {
    pub thresh: f32,
    pub icon_ratio: f32,
    pub bg: f32,
    pub light_mix: f32,
    pub dark_mix: f32,
    pub gloss: f32,
    pub icon_light: f32,
    pub corner: f32,
    pub shape: HqPanelShape,
    pub offset_x: f32,
    pub offset_y: f32,
    pub custom_bg: Option<[f32; 3]>,
    pub shadow_opacity: f32,
    pub shadow_blur_factor: f32,
    pub shadow_offset_factor: f32,
    pub shadow_fade: f32,
    pub shadow_mode: HqShadowMode,
    // Classic edge gloss, applied on top of the adaptive tile when enabled.
    pub edge_gloss_enabled: bool,
    pub edge_gloss_width: f32,
    pub edge_gloss_strength: f32,
    pub edge_gloss_light: [f32; 4],
    pub edge_gloss_base: [f32; 4],
    pub edge_gloss_feather_blur: f32,
}

pub fn validate_hq(config: &RenderConfig) -> Result<ValidatedHqRenderConfig, AppError> {
    fn check(v: f32, lo: f32, hi: f32, name: &str) -> Result<f32, AppError> {
        if !v.is_finite() || !(lo..=hi).contains(&v) {
            return Err(AppError::InvalidArgument(format!(
                "{name} must be in {lo}..={hi}"
            )));
        }
        Ok(v)
    }

    let hq = &config.hq_render;
    Ok(ValidatedHqRenderConfig {
        thresh: check(hq.thresh, 0.0, 1.0, "hq thresh")?,
        icon_ratio: check(hq.icon_ratio, 0.1, 1.0, "hq iconRatio")?,
        bg: check(hq.bg, 0.0, 1.0, "hq bg")?,
        light_mix: check(hq.light_mix, 0.0, 1.0, "hq lightMix")?,
        dark_mix: check(hq.dark_mix, 0.0, 1.0, "hq darkMix")?,
        gloss: check(hq.gloss, 0.0, 1.0, "hq gloss")?,
        icon_light: check(hq.icon_light, 0.0, 1.0, "hq iconLight")?,
        corner: check(hq.corner, 0.02, 0.5, "hq corner")?,
        shape: hq.shape,
        offset_x: check(hq.offset_x, -64.0, 64.0, "hq offsetX")?,
        offset_y: check(hq.offset_y, -64.0, 64.0, "hq offsetY")?,
        custom_bg: if hq.custom_bg_enabled {
            let c = super::color::parse_hex_srgb_rgba(&hq.custom_bg_color)?;
            Some([c[0] * 255.0, c[1] * 255.0, c[2] * 255.0])
        } else {
            None
        },
        shadow_opacity: check(hq.shadow_opacity, 0.0, 1.0, "hq shadowOpacity")?,
        shadow_blur_factor: check(hq.shadow_blur_factor, 0.0, 0.1, "hq shadowBlurFactor")?,
        shadow_offset_factor: check(
            hq.shadow_offset_factor,
            0.0,
            0.1,
            "hq shadowOffsetFactor",
        )?,
        shadow_fade: check(hq.shadow_fade, 0.0, 1.0, "hq shadowFade")?,
        shadow_mode: hq.shadow_mode,
        edge_gloss_enabled: config.gloss.enabled,
        edge_gloss_width: check(config.gloss.width, 0.0, 32.0, "gloss width")?,
        edge_gloss_strength: check(config.gloss.strength, 0.0, 1.0, "gloss strength")?,
        edge_gloss_light: super::color::parse_hex_srgb_rgba(&config.gloss.light_color)?,
        edge_gloss_base: super::color::parse_hex_srgb_rgba(&config.gloss.base_color)?,
        edge_gloss_feather_blur: check(config.gloss.feather_blur, 0.0, 32.0, "gloss featherBlur")?,
    })
}

/// Port of `iconmask.py` — adaptive macOS-style icon tile renderer.
/// All layers are composited in straight (non-premultiplied) sRGB,
/// matching the reference Python implementation 1:1.
pub fn render_hq(
    source: &RgbaImage,
    cfg: &ValidatedHqRenderConfig,
) -> Result<RgbaImage, AppError> {
    let size = HQ_SIZE as usize;
    if source.width() == 0 || source.height() == 0 {
        return Err(AppError::InvalidArgument("source image is empty".into()));
    }
    // A fully transparent source has nothing to base a panel on.
    if alpha_bbox(source).is_none() {
        return Ok(RgbaImage::new(HQ_SIZE, HQ_SIZE));
    }

    // 1. Trim transparent margin, then pad into a square canvas.
    let icon = trim_and_pad_square(source, HQ_SIZE);

    let dom = dominant_color(&icon);
    let lum = luminance_of(&icon);
    let bright = lum > cfg.thresh;
    // Panel base: a user-chosen solid color overrides the automatic
    // light/dark derivation; the radial halo still uses the icon colors.
    let panel = if let Some(bg) = &cfg.custom_bg {
        *bg
    } else if bright {
        [
            dom[0] + (255.0 - dom[0]) * cfg.light_mix,
            dom[1] + (255.0 - dom[1]) * cfg.light_mix,
            dom[2] + (255.0 - dom[2]) * cfg.light_mix,
        ]
    } else {
        [
            dom[0] * (1.0 - cfg.dark_mix),
            dom[1] * (1.0 - cfg.dark_mix),
            dom[2] * (1.0 - cfg.dark_mix),
        ]
    };

    // 2. Mask shape.
    let mask = make_mask(size as u32, cfg.shape, cfg.corner);

    // 3. Icon layer.
    let icon_layer = render_icon_layer(&icon, size, cfg);

    // 4. Radial background: 360 sectors tinted by the icon's ray colors.
    let (rays, icx, icy, ihw, ihh) = radial_ray_colors(&icon_layer, size);
    let edges: Vec<f32> = (0..RAYS)
        .map(|i| {
            edge_radius_at(
                &mask,
                size,
                icx,
                icy,
                std::f32::consts::TAU * i as f32 / RAYS as f32,
            )
        })
        .collect();

    // Canvas holds straight RGBA normalized to 0..1 (sRGB values / 255).
    let mut canvas = vec![[0.0f32; 4]; size * size];
    for y in 0..size {
        let t = y as f32 / (size - 1) as f32;
        let vg = 1.02 - (1.02 - 0.98) * t;
        for x in 0..size {
            let dx = x as f32 - icx;
            let dy = icy - y as f32;
            let mut th = dy.atan2(dx);
            if th < 0.0 {
                th += std::f32::consts::TAU;
            }
            let fi = th / std::f32::consts::TAU * RAYS as f32;
            let i0 = (fi as usize) % RAYS;
            let i1 = (i0 + 1) % RAYS;
            let f = fi - fi.floor();
            let c0 = rays[i0];
            let c1 = rays[i1];
            let ray_c = [
                c0[0] + (c1[0] - c0[0]) * f,
                c0[1] + (c1[1] - c0[1]) * f,
                c0[2] + (c1[2] - c0[2]) * f,
            ];
            let er = edges[i0] + (edges[i1] - edges[i0]) * f;
            let dist = (dx * dx + dy * dy).sqrt();
            let ac = th.cos().abs();
            let as_ = th.sin().abs();
            let ri = (ihw / ac.max(1e-9)).min(ihh / as_.max(1e-9));
            let rf = if dist <= ri {
                1.0
            } else {
                let span = (er - ri).max(1.0);
                let s = ((dist - ri) / span).min(1.0);
                (1.0 - s).max(0.0).sqrt()
            };
            let k = cfg.bg;
            // The halo tints the panel toward the icon ray color. Using the
            // panel base (instead of the dominant color) as reference keeps
            // the tint visible even with user-defined base colors: on a
            // white/black base the icon hue shows through strongly.
            let r_ = (panel[0] + (ray_c[0] - panel[0]) * k * rf) * vg;
            let g_ = (panel[1] + (ray_c[1] - panel[1]) * k * rf) * vg;
            let b_ = (panel[2] + (ray_c[2] - panel[2]) * k * rf) * vg;
            canvas[y * size + x] = [
                clamp_rgb(r_),
                clamp_rgb(g_),
                clamp_rgb(b_),
                1.0,
            ];
        }
    }

    // Apply the mask as the background alpha.
    for i in 0..size * size {
        canvas[i][3] = mask[i];
    }

    // 5. Gloss, shadow, icon over the background.
    let gloss = gloss_layer(size, cfg.gloss);
    let offset = (HQ_SIZE as f32 * cfg.shadow_offset_factor) as i32;
    let blur = if cfg.shadow_blur_factor <= 0.0 {
        0.0
    } else {
        (HQ_SIZE as f32 * cfg.shadow_blur_factor).max(3.0)
    };
    let shadow_src: Vec<f32> = if cfg.shadow_mode == HqShadowMode::Icon {
        icon_layer_alpha(&icon_layer)
    } else {
        mask.clone()
    };
    let shadow = cast_shadow(&shadow_src, size, offset, blur, cfg.shadow_opacity, cfg.shadow_fade);
    let edge_gloss = render_edge_gloss(&mask, size, cfg);
    over(&mut canvas, &gloss);
    over(&mut canvas, &edge_gloss);
    over(&mut canvas, &shadow);
    over(&mut canvas, &icon_layer);

    // 6. Clip everything to the mask.
    let mut bytes = Vec::with_capacity(size * size * 4);
    for i in 0..size * size {
        let p = canvas[i];
        let a = (p[3] * mask[i]).clamp(0.0, 1.0);
        bytes.extend_from_slice(&[
            (p[0] * 255.0).round().clamp(0.0, 255.0) as u8,
            (p[1] * 255.0).round().clamp(0.0, 255.0) as u8,
            (p[2] * 255.0).round().clamp(0.0, 255.0) as u8,
            (a * 255.0).round().clamp(0.0, 255.0) as u8,
        ]);
    }

    RgbaImage::from_raw(size as u32, size as u32, bytes)
        .ok_or_else(|| AppError::Internal("hq render buffer allocation failed".into()))
}

fn clamp_rgb(v: f32) -> f32 {
    if v <= 0.0 {
        0.0
    } else if v >= 255.0 {
        1.0
    } else {
        v / 255.0
    }
}

fn alpha_bbox(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if max_x < min_x {
        return None;
    }
    Some((min_x, min_y, max_x, max_y))
}

fn trim_and_pad_square(src: &RgbaImage, size: u32) -> RgbaImage {
    let bbox = match alpha_bbox(src) {
        Some(b) => b,
        None => return RgbaImage::new(size, size),
    };
    let (x0, y0, x1, y1) = bbox;
    let iw = x1 - x0 + 1;
    let ih = y1 - y0 + 1;
    let f = (size as f32 / iw as f32).min(size as f32 / ih as f32);
    let nw = (iw as f32 * f + 0.5).max(1.0) as u32;
    let nh = (ih as f32 * f + 0.5).max(1.0) as u32;
    let data = src.view(x0, y0, iw, ih).to_image();
    let scaled = image::imageops::resize(&data, nw, nh, FilterType::Lanczos3);
    let mut canvas = RgbaImage::new(size, size);
    let ox = (size - nw) / 2;
    let oy = (size - nh) / 2;
    image::imageops::overlay(&mut canvas, &scaled, ox as i64, oy as i64);
    canvas
}

/// Box-downsample each channel into a GxG grid (PIL `resize(..., BOX)`).
fn box_grid(img: &RgbaImage, grid: usize) -> Vec<[f32; 4]> {
    let (w, h) = img.dimensions();
    let mut sums = vec![[0.0f64; 4]; grid * grid];
    let mut counts = vec![0u64; grid * grid];
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y).0;
            let gy = ((y as u64) * grid as u64 / h as u64) as usize;
            let gx = ((x as u64) * grid as u64 / w as u64) as usize;
            let idx = gy * grid + gx;
            for c in 0..4 {
                sums[idx][c] += p[c] as f64;
            }
            counts[idx] += 1;
        }
    }
    let mut grid_out = vec![[0.0f32; 4]; grid * grid];
    for idx in 0..grid * grid {
        let n = counts[idx].max(1) as f64;
        for c in 0..4 {
            grid_out[idx][c] = (sums[idx][c] / n) as f32;
        }
    }
    grid_out
}

fn dominant_color(icon: &RgbaImage) -> [f32; 3] {
    const G: usize = 8;
    let grid = box_grid(icon, G);
    let mut acc = [0.0f32; 3];
    let mut total_w = 0.0f32;
    for cell in grid.iter() {
        let av = cell[3];
        if av <= 0.0 {
            continue;
        }
        let mx = cell[0].max(cell[1]).max(cell[2]);
        let mn = cell[0].min(cell[1]).min(cell[2]);
        let sat = (mx - mn) / 255.0;
        let w = av * (0.25 + 1.5 * sat);
        acc[0] += cell[0] * w;
        acc[1] += cell[1] * w;
        acc[2] += cell[2] * w;
        total_w += w;
    }
    if total_w <= 0.0 {
        return [255.0, 255.0, 255.0];
    }
    [acc[0] / total_w, acc[1] / total_w, acc[2] / total_w]
}

fn luminance_of(icon: &RgbaImage) -> f32 {
    const G: usize = 16;
    let grid = box_grid(icon, G);
    let mut tot = 0.0f32;
    let mut wsum = 0.0f32;
    for cell in grid.iter() {
        let av = cell[3];
        if av <= 0.0 {
            continue;
        }
        let lum = (0.2126 * cell[0] + 0.7152 * cell[1] + 0.0722 * cell[2]) / 255.0;
        tot += lum * av;
        wsum += av;
    }
    if wsum <= 0.0 {
        0.5
    } else {
        tot / wsum
    }
}

fn make_mask(size: u32, shape: HqPanelShape, corner: f32) -> Vec<f32> {
    const SS: usize = 4;
    let mut mask = vec![0.0f32; (size * size) as usize];
    for y in 0..size {
        for x in 0..size {
            let mut hits = 0usize;
            for sy in 0..SS {
                for sx in 0..SS {
                    let px = x as f32 + (sx as f32 + 0.5) / SS as f32;
                    let py = y as f32 + (sy as f32 + 0.5) / SS as f32;
                    if inside_shape(px, py, size, shape, corner) {
                        hits += 1;
                    }
                }
            }
            mask[(y * size + x) as usize] = hits as f32 / (SS * SS) as f32;
        }
    }
    mask
}

fn inside_shape(px: f32, py: f32, size: u32, shape: HqPanelShape, corner: f32) -> bool {
    let w = size as f32;
    let h = size as f32;
    match shape {
        HqPanelShape::Circle => {
            let cx = (w - 1.0) / 2.0;
            let ry = (h - 1.0) / 2.0;
            let dx = (px - cx) / cx;
            let dy = (py - cx) / ry;
            dx * dx + dy * dy <= 1.0
        }
        HqPanelShape::Rect => {
            let cx = (w - 1.0) / 2.0;
            let cy = (h - 1.0) / 2.0;
            let rad = corner * w;
            let hx = cx - rad;
            let hy = cy - rad;
            let qx = ((px - cx).abs() - hx).max(0.0);
            let qy = ((py - cy).abs() - hy).max(0.0);
            qx * qx + qy * qy <= rad * rad
        }
        HqPanelShape::Squircle => {
            let cx = (w - 1.0) / 2.0;
            let cy = (h - 1.0) / 2.0;
            let rx = cx;
            let ry = cy;
            let n = 5.0;
            let dx = ((px - cx) / rx).abs().powf(n);
            let dy = ((py - cy) / ry).abs().powf(n);
            dx + dy <= 1.0
        }
    }
}

fn render_icon_layer(icon: &RgbaImage, size: usize, cfg: &ValidatedHqRenderConfig) -> Vec<[f32; 4]> {
    let w = size;
    let h = size;
    let mut out = vec![[0.0f32; 4]; w * h];

    let bbox = match alpha_bbox(icon) {
        Some(b) => b,
        None => return out,
    };
    let (x0, y0, x1, y1) = bbox;
    let iw = (x1 - x0 + 1) as usize;
    let ih = (y1 - y0 + 1) as usize;
    let ratio = cfg.icon_ratio;
    let s = ((w as f32 * ratio) / iw as f32).min((h as f32 * ratio) / ih as f32);
    let nw = ((iw as f32 * s) + 0.5).max(1.0) as u32;
    let nh = ((ih as f32 * s) + 0.5).max(1.0) as u32;
    let data = icon.view(x0, y0, iw as u32, ih as u32).to_image();
    let scaled = image::imageops::resize(&data, nw, nh, FilterType::Lanczos3);

    let brighten = if cfg.icon_light > 0.0 {
        1.0 + cfg.icon_light
    } else {
        1.0
    };
    // Centered placement plus the adjustable X/Y offset (and the reference
    // script's fixed 1.2% downward bias, which 0 position keeps).
    let ox = (w as isize - nw as isize) / 2 + cfg.offset_x.round() as isize;
    let oy = (h as isize - nh as isize) / 2
        + ((size as f32 * 0.012).floor() as isize).max(1)
        + cfg.offset_y.round() as isize;
    for yy in 0..nh as isize {
        for xx in 0..nw as isize {
            let dx_ = ox + xx;
            let dy_ = oy + yy;
            if dx_ < 0 || dy_ < 0 || dx_ >= w as isize || dy_ >= h as isize {
                continue;
            }
            let p = scaled.get_pixel(xx as u32, yy as u32).0;
            let a = p[3] as f32 / 255.0;
            if a <= 0.0 {
                continue;
            }
            let r = (p[0] as f32 * brighten).min(255.0);
            let g = (p[1] as f32 * brighten).min(255.0);
            let b = (p[2] as f32 * brighten).min(255.0);
            out[dy_ as usize * w + dx_ as usize] = [r / 255.0, g / 255.0, b / 255.0, a];
        }
    }
    out
}

fn icon_layer_alpha(icon_layer: &[[f32; 4]]) -> Vec<f32> {
    icon_layer.iter().map(|p| p[3]).collect()
}

fn radial_ray_colors(icon_layer: &[[f32; 4]], size: usize) -> (Vec<[f32; 3]>, f32, f32, f32, f32) {
    let mut min_x = size as i64;
    let mut min_y = size as i64;
    let mut max_x = 0i64;
    let mut max_y = 0i64;
    let mut any = false;
    for y in 0..size {
        for x in 0..size {
            if icon_layer[y * size + x][3] > 0.0 {
                min_x = min_x.min(x as i64);
                min_y = min_y.min(y as i64);
                max_x = max_x.max(x as i64);
                max_y = max_y.max(y as i64);
                any = true;
            }
        }
    }
    let (w, h) = (size as f32, size as f32);
    if !any {
        return (vec![[255.0f32; 3]; RAYS], w / 2.0, h / 2.0, w / 4.0, h / 4.0);
    }
    let icx = (min_x + max_x) as f32 / 2.0;
    let icy = (min_y + max_y) as f32 / 2.0;
    let icw_b = (max_x - min_x + 1) as f32;
    let ich_b = (max_y - min_y + 1) as f32;
    let max_r = (icw_b * icw_b + ich_b * ich_b).sqrt() / 2.0 + 1.0;

    let mut raw = vec![[0.0f32; 3]; RAYS];
    for i in 0..RAYS {
        let th = std::f32::consts::TAU * i as f32 / RAYS as f32;
        let (dx_, dy_) = (th.cos(), th.sin());
        let mut sr = 0.0f32;
        let mut sg = 0.0f32;
        let mut sb = 0.0f32;
        let mut sw = 0.0f32;
        for r in 1..max_r as i32 {
            let x = icx + dx_ * r as f32;
            let y = icy - dy_ * r as f32;
            if x < 0.0 || y < 0.0 || x >= w || y >= h {
                break;
            }
            let idx = (y as usize) * size + (x as usize);
            let a = icon_layer[idx][3];
            if a > 10.0 / 255.0 {
                let wgt = a * (r as f32 / max_r).powf(1.4);
                sw += wgt;
                sr += icon_layer[idx][0] * wgt;
                sg += icon_layer[idx][1] * wgt;
                sb += icon_layer[idx][2] * wgt;
            }
        }
        if sw > 0.0 {
            raw[i] = [sr / sw * 255.0, sg / sw * 255.0, sb / sw * 255.0];
        } else {
            raw[i] = [255.0, 255.0, 255.0];
        }
    }

    // Angular smoothing over ±ANG_SMOOTH rays.
    let mut colors = vec![[0.0f32; 3]; RAYS];
    for i in 0..RAYS {
        let mut s = [0.0f32; 3];
        for j in -ANG_SMOOTH..=ANG_SMOOTH {
            let idx = (i as i32 + j).rem_euclid(RAYS as i32) as usize;
            for c in 0..3 {
                s[c] += raw[idx][c];
            }
        }
        let n = (2 * ANG_SMOOTH + 1) as f32;
        for c in 0..3 {
            s[c] /= n;
        }
        colors[i] = s;
    }

    (colors, icx, icy, icw_b / 2.0, ich_b / 2.0)
}

fn edge_radius_at(mask: &[f32], size: usize, icx: f32, icy: f32, th: f32) -> f32 {
    let mut r = 0.0f32;
    let dx_ = th.cos();
    let dy_ = th.sin();
    loop {
        r += 1.0;
        let x = icx + dx_ * r;
        let y = icy - dy_ * r;
        if x < 0.0 || y < 0.0 || x >= size as f32 || y >= size as f32 {
            return r;
        }
        if mask[(y as usize) * size + (x as usize)] < 128.0 / 255.0 {
            return r;
        }
    }
}

fn gloss_layer(size: usize, strength: f32) -> Vec<[f32; 4]> {
    let mut out = vec![[0.0f32; 4]; size * size];
    for y in 0..size {
        let t = y as f32 / (size - 1) as f32;
        let mut f = 1.0 - t / 0.30;
        if f < 0.0 {
            f = 0.0;
        }
        let a = (f * f * strength).clamp(0.0, 1.0);
        for x in 0..size {
            out[y * size + x] = [1.0, 1.0, 1.0, a];
        }
    }
    out
}

/// Angular spans (radians, y-down screen coordinates where the angle grows
/// clockwise) of the two highlight regions of the HQ panel, measured from the
/// panel center: `(a1, a2, b1, b2)`, plus the shared opacity profile.
///
/// Top-left highlight spans clockwise from the upper end of the bottom-left
/// rounding along the left/top edges to the left end of the top-right rounding
/// (region A, sweep `a1 -> a2`); the bottom-right highlight covers the
/// bottom-right rounding from its left side (bottom edge end) to its upper
/// side (right edge end), region B = angle interval `[b2, b1]`.
fn gloss_region_angles(size: usize, shape: HqPanelShape, corner: f32) -> ((f32, f32, f32, f32), super::gloss::GlossProfile) {
    let no_edges = super::gloss::GlossProfile { k1: 0.0, k2: 1.0 };
    match shape {
        HqPanelShape::Circle | HqPanelShape::Squircle => (
            (std::f32::consts::PI, -std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0, 0.0),
            no_edges,
        ),
        HqPanelShape::Rect => {
            let c = (size as f32 - 1.0) / 2.0;
            let rad = corner * size as f32;
            let x0 = rad;
            let x1 = size as f32 - rad;
            let y0 = rad;
            let y1 = size as f32 - rad;
            let a1 = (y1 - rad - c).atan2(x0 - c);
            let a2 = (y0 - c).atan2(x1 - rad - c);
            let b1 = (y1 - c).atan2(x1 - rad - c);
            let b2 = (y1 - rad - c).atan2(x1 - c);
            let span_a = (a2 - a1).rem_euclid(std::f32::consts::TAU);
            let k1 = ((y0 + rad - c).atan2(x0 - c) - a1).rem_euclid(std::f32::consts::TAU) / span_a;
            let k2 = ((y0 - c).atan2(x0 + rad - c) - a1).rem_euclid(std::f32::consts::TAU) / span_a;
            ((a1, a2, b1, b2), super::gloss::GlossProfile { k1, k2 })
        }
    }
}

/// Edge gloss: a base-colored ring along the whole panel edge, with a light
/// highlight overlaid on top of it along the left/top edges (region A) and on
/// the bottom-right corner arc (region B). The highlight peaks at the corner
/// arcs, sits slightly lower along the straight edges, and fades to
/// transparent at the ends of both regions so the edge base color shows
/// through with a gradual transition. Rendered in straight sRGB so it can be
/// composited onto the adaptive tile.
fn render_edge_gloss(mask: &[f32], size: usize, cfg: &ValidatedHqRenderConfig) -> Vec<[f32; 4]> {
    let mut out = vec![[0.0f32; 4]; size * size];
    if !cfg.edge_gloss_enabled || cfg.edge_gloss_width <= 0.0 || cfg.edge_gloss_strength <= 0.0 {
        return out;
    }
    let base_inset = erode_mask(mask, size, cfg.edge_gloss_width.ceil() as usize);
    let inset = if cfg.edge_gloss_feather_blur > 0.0 {
        gaussian_blur(&base_inset, size, size, cfg.edge_gloss_feather_blur)
    } else {
        base_inset
    };
    let light = cfg.edge_gloss_light;
    let base = cfg.edge_gloss_base;
    let ((a1, a2, b1, b2), profile) = gloss_region_angles(size, cfg.shape, cfg.corner);
    let span_a = (a2 - a1).rem_euclid(std::f32::consts::TAU);
    let span_b = b1 - b2;
    let c = (size as f32 - 1.0) / 2.0;

    let mut highlight = vec![[0.0f32; 4]; size * size];
    for y in 0..size {
        for x in 0..size {
            let idx = y * size + x;
            let edge = (mask[idx] - inset[idx]).clamp(0.0, 1.0);
            if edge <= 0.0 {
                continue;
            }
            let a = (base[3] * edge * cfg.edge_gloss_strength).clamp(0.0, 1.0);
            if a > 0.0 {
                out[idx] = [base[0], base[1], base[2], a];
            }

            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let theta = (py - c).atan2(px - c);
            let phi = (theta - a1).rem_euclid(std::f32::consts::TAU);
            let weight = if phi <= span_a {
                super::gloss::region_a_weight(phi, span_a, &profile)
            } else if span_b > 0.0 && theta >= b2 && theta <= b1 {
                super::gloss::bottom_right_weight((b1 - theta) / span_b)
            } else {
                continue;
            };
            let alpha = (light[3] * edge * weight * cfg.edge_gloss_strength).clamp(0.0, 1.0);
            if alpha <= 0.0 {
                continue;
            }
            highlight[idx] = [light[0], light[1], light[2], alpha];
        }
    }
    over(&mut out, &highlight);
    out
}

/// Rectangular erosion of a coverage mask (two separable min passes), used to
/// carve the inset band that defines the edge gloss falloff.
fn erode_mask(mask: &[f32], size: usize, radius: usize) -> Vec<f32> {
    if radius == 0 {
        return mask.to_vec();
    }
    let mut horizontal = vec![0.0f32; size * size];
    for y in 0..size {
        for x in 0..size {
            let mut min_val = f32::INFINITY;
            let lo = x.saturating_sub(radius);
            let hi = (x + radius).min(size - 1);
            for sx in lo..=hi {
                min_val = min_val.min(mask[y * size + sx]);
            }
            horizontal[y * size + x] = min_val;
        }
    }
    let mut out = vec![0.0f32; size * size];
    for y in 0..size {
        let lo = y.saturating_sub(radius);
        let hi = (y + radius).min(size - 1);
        for x in 0..size {
            let mut min_val = f32::INFINITY;
            for sy in lo..=hi {
                min_val = min_val.min(horizontal[sy * size + x]);
            }
            out[y * size + x] = min_val;
        }
    }
    out
}

fn gaussian_blur(mask: &[f32], width: usize, height: usize, sigma: f32) -> Vec<f32> {
    if sigma <= 0.0 {
        return mask.to_vec();
    }
    let kernel_radius = (3.0 * sigma).ceil() as i32;
    let kernel_size = (2 * kernel_radius + 1) as usize;
    let mut kernel = vec![0.0f32; kernel_size];
    let mut sum = 0.0f32;
    for i in -kernel_radius..=kernel_radius {
        let val = (-(i * i) as f32 / (2.0 * sigma * sigma)).exp();
        kernel[(i + kernel_radius) as usize] = val;
        sum += val;
    }
    for k in &mut kernel {
        *k /= sum;
    }

    let mut temp = vec![0.0f32; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sx = x as i32 + ki as i32 - kernel_radius;
                if sx >= 0 && sx < width as i32 {
                    acc += mask[y * width + sx as usize] * kv;
                }
            }
            temp[y * width + x] = acc;
        }
    }

    let mut result = vec![0.0f32; mask.len()];
    for y in 0..height {
        for x in 0..width {
            let mut acc = 0.0f32;
            for (ki, &kv) in kernel.iter().enumerate() {
                let sy = y as i32 + ki as i32 - kernel_radius;
                if sy >= 0 && sy < height as i32 {
                    acc += temp[sy as usize * width + x] * kv;
                }
            }
            result[y * width + x] = acc;
        }
    }
    result
}

fn cast_shadow(
    src_alpha: &[f32],
    size: usize,
    offset: i32,
    blur: f32,
    opacity: f32,
    fade: f32,
) -> Vec<[f32; 4]> {
    let mut shifted = vec![0.0f32; size * size];
    for y in 0..size {
        let sy = y as i32 + offset;
        if sy < 0 || sy >= size as i32 {
            continue;
        }
        for x in 0..size {
            shifted[sy as usize * size + x] = src_alpha[y * size + x];
        }
    }
    let blurred = gaussian_blur(&shifted, size, size, blur);
    let mut out = vec![[0.0f32; 4]; size * size];
    for y in 0..size {
        let t = y as f32 / (size - 1) as f32;
        let factor = 1.0 - fade * ((t - 0.5) / 0.5).max(0.0);
        for x in 0..size {
            let v = blurred[y * size + x];
            if v <= 0.0 {
                continue;
            }
            out[y * size + x] = [0.0, 0.0, 0.0, (v * factor * opacity).min(1.0)];
        }
    }
    out
}

fn over(dst: &mut [[f32; 4]], src: &[[f32; 4]]) {
    for (d, &s) in dst.iter_mut().zip(src.iter()) {
        let sa = s[3];
        if sa <= 0.0 {
            continue;
        }
        let da = d[3];
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            continue;
        }
        for c in 0..3 {
            d[c] = (s[c] * sa + d[c] * da * (1.0 - sa)) / out_a;
        }
        d[3] = out_a;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(color: [u8; 3], size: u32) -> RgbaImage {
        let mut img = RgbaImage::new(size, size);
        for p in img.pixels_mut() {
            *p = Rgba([color[0], color[1], color[2], 255]);
        }
        img
    }

    fn test_cfg() -> ValidatedHqRenderConfig {
        ValidatedHqRenderConfig {
            thresh: 0.5,
            icon_ratio: 0.66,
            bg: 0.22,
            light_mix: 0.9,
            dark_mix: 0.72,
            gloss: 0.16,
            icon_light: 0.08,
            corner: 0.22,
            shape: HqPanelShape::Rect,
            offset_x: 0.0,
            offset_y: 0.0,
            custom_bg: None,
            shadow_opacity: 0.22,
            shadow_blur_factor: 0.022,
            shadow_offset_factor: 0.012,
            shadow_fade: 0.25,
            shadow_mode: HqShadowMode::Icon,
            edge_gloss_enabled: false,
            edge_gloss_width: 2.0,
            edge_gloss_strength: 0.6,
            edge_gloss_light: [1.0, 1.0, 1.0, 0.7],
            edge_gloss_base: [0.0, 0.0, 0.0, 0.5],
            edge_gloss_feather_blur: 0.0,
        }
    }

    #[test]
    fn render_light_icon_produces_light_panel() {
        let src = solid([120, 220, 255], 256);
        let out = render_hq(&src, &test_cfg()).unwrap();
        // Below the rendered icon, the panel color shows through.
        let p = out.get_pixel(128, 236);
        assert!(p[3] >= 250);
        assert!(p[0] >= 190 && p[1] >= 190 && p[2] >= 190);
    }

    #[test]
    fn render_dark_icon_produces_dark_panel() {
        let mut cfg = test_cfg();
        cfg.thresh = 0.9;
        let out = render_hq(&solid([30, 40, 50], 64), &cfg).unwrap();
        let p = out.get_pixel(128, 236);
        assert!(p[3] >= 250);
        assert!(p[0] < 60 && p[1] < 60 && p[2] < 60);
    }

    #[test]
    fn transparent_source_yields_empty_tile() {
        let out = render_hq(&RgbaImage::new(256, 256), &test_cfg()).unwrap();
        assert_eq!(out.pixels().filter(|p| p[3] > 0).count(), 0);
    }
}