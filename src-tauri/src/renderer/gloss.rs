use super::color::parse_hex_rgba;
use super::composite::LinearPremultipliedImage;
use super::mask::AlphaMask;
use super::pipeline::ValidatedRenderConfig;
use crate::domain::config::GlossConfig;
use crate::error::app_error::AppError;

const SQRT_2: f32 = std::f32::consts::SQRT_2;

/// Edge gloss profile, shared by the classic and the adaptive HQ renderers.
///
/// Ported from the reference icon pipeline (`edge_glow`): a rim light that
/// hugs the shape boundary and fades inwards as `exp(-d / rim)`, modulated by
/// two soft bands along a rotatable light axis — a strong one on the lit side
/// and a weaker one on the opposite side — so the icon reads as lit from
/// above. Only the silhouette matters, so flat artwork gets the same rim as
/// detailed artwork.
#[derive(Debug, Clone, Copy)]
pub struct EdgeGlossParams {
    /// Rim falloff distance in px: the light decays as `exp(-d / rim)`.
    pub rim: f32,
    /// Overall intensity multiplier.
    pub strength: f32,
    /// Intensity of the band on the lit side of the axis.
    pub top: f32,
    /// Intensity of the band on the opposite side.
    pub bottom: f32,
    /// How far the lit band reaches from the light side, in px.
    pub top_decay: f32,
    /// How far the opposite band reaches, in px.
    pub bottom_decay: f32,
    /// Rotation of the light axis in degrees (0 = light from the top,
    /// positive values rotate the light source clockwise).
    pub angle_degrees: f32,
}

impl EdgeGlossParams {
    /// Whether the profile can produce any light at all.
    pub fn is_active(&self) -> bool {
        self.rim > 0.0 && self.strength > 0.0 && (self.top > 0.0 || self.bottom > 0.0)
    }
}

/// Validate the gloss settings and fold them into a profile. Returns `None`
/// when the effect is switched off or has nothing to draw, so the renderers
/// can skip the whole pass.
pub fn validate_edge_gloss(gloss: &GlossConfig) -> Result<Option<EdgeGlossParams>, AppError> {
    fn check(v: f32, lo: f32, hi: f32, name: &str) -> Result<f32, AppError> {
        if !v.is_finite() || !(lo..=hi).contains(&v) {
            return Err(AppError::InvalidArgument(format!(
                "{name} must be in {lo}..={hi}"
            )));
        }
        Ok(v)
    }
    let params = EdgeGlossParams {
        rim: check(gloss.width, 0.0, 32.0, "gloss width")?,
        strength: check(gloss.strength, 0.0, 1.0, "gloss strength")?,
        top: check(gloss.top_strength, 0.0, 1.0, "gloss topStrength")?,
        bottom: check(gloss.bottom_strength, 0.0, 1.0, "gloss bottomStrength")?,
        top_decay: check(gloss.top_decay, 1.0, 256.0, "gloss topDecay")?,
        bottom_decay: check(gloss.bottom_decay, 1.0, 256.0, "gloss bottomDecay")?,
        angle_degrees: check(gloss.angle_degrees, -360.0, 360.0, "gloss angleDegrees")?,
    };
    Ok(if gloss.enabled && params.is_active() {
        Some(params)
    } else {
        None
    })
}

/// Distance in px from every covered pixel to the nearest uncovered pixel of
/// `mask` (0 at and outside the boundary, growing inwards). Two-pass 3-4
/// chamfer transform: exact for the axis-aligned steps that dominate a thin
/// rim, and within a few percent on the diagonals.
pub fn distance_inside(mask: &[f32], size: usize) -> Vec<f32> {
    const INF: f32 = 1.0e9;
    let mut d = vec![INF; size * size];
    for (i, &m) in mask.iter().enumerate() {
        if m < 0.5 {
            d[i] = 0.0;
        }
    }
    // Everything outside the canvas counts as uncovered — the reference pads
    // the silhouette with one transparent ring before the distance transform.
    // Without it a shape that sits flush against the canvas border (the
    // default preset has canvasInset 0) has no uncovered pixel anywhere near
    // the top edge, so the distance is measured all the way to the corner
    // cut-outs: the rim then only shows around the corners and the edges stay
    // dark.
    for x in 0..size {
        for y in [0, size - 1] {
            let i = y * size + x;
            if d[i] > 0.0 {
                d[i] = 1.0;
            }
        }
    }
    for y in 0..size {
        for x in [0, size - 1] {
            let i = y * size + x;
            if d[i] > 0.0 {
                d[i] = 1.0;
            }
        }
    }
    for y in 0..size {
        for x in 0..size {
            let i = y * size + x;
            let mut best = d[i];
            if y > 0 {
                best = best.min(d[i - size] + 1.0);
                if x > 0 {
                    best = best.min(d[i - size - 1] + SQRT_2);
                }
                if x + 1 < size {
                    best = best.min(d[i - size + 1] + SQRT_2);
                }
            }
            if x > 0 {
                best = best.min(d[i - 1] + 1.0);
            }
            d[i] = best;
        }
    }
    for y in (0..size).rev() {
        for x in (0..size).rev() {
            let i = y * size + x;
            let mut best = d[i];
            if y + 1 < size {
                best = best.min(d[i + size] + 1.0);
                if x > 0 {
                    best = best.min(d[i + size - 1] + SQRT_2);
                }
                if x + 1 < size {
                    best = best.min(d[i + size + 1] + SQRT_2);
                }
            }
            if x + 1 < size {
                best = best.min(d[i + 1] + 1.0);
            }
            d[i] = best;
        }
    }
    d
}

/// The edge-gloss light field: per-pixel intensity in `0..=1`, zero outside
/// the silhouette. `half_extent` is the half size of the square the shape
/// fills, used to place the two vertical bands on the shape rather than on the
/// canvas.
pub fn edge_glow_field(
    mask: &[f32],
    size: usize,
    half_extent: f32,
    params: &EdgeGlossParams,
) -> Vec<f32> {
    let mut out = vec![0.0f32; size * size];
    if !params.is_active() {
        return out;
    }
    let d = distance_inside(mask, size);

    // Light axis: 0° points straight up, positive angles rotate the light
    // source clockwise in screen space (y grows downwards).
    let theta = params.angle_degrees.to_radians();
    let (ax, ay) = (theta.sin(), -theta.cos());
    let centre = size as f32 / 2.0;
    // Half extent of the shape along the light axis, so the lit band starts at
    // full strength right on the shape boundary for any rotation.
    let reach = half_extent.max(1.0) * (ax.abs() + ay.abs());

    let inv_rim = 1.0 / params.rim.max(0.01);
    let inv_top = 1.0 / params.top_decay.max(0.01);
    let inv_bottom = 1.0 / params.bottom_decay.max(0.01);

    for y in 0..size {
        let py = y as f32 + 0.5 - centre;
        for x in 0..size {
            let i = y * size + x;
            if mask[i] < 0.5 {
                continue;
            }
            let px = x as f32 + 0.5 - centre;
            // Projection on the light axis: positive towards the light source.
            let s = px * ax + py * ay;
            let lit = params.top * (-(reach - s).max(0.0) * inv_top).exp();
            let shade = params.bottom * (-(reach + s).max(0.0) * inv_bottom).exp();
            let band = lit + shade;
            if band <= 0.0 {
                continue;
            }
            let rim = (-d[i] * inv_rim).exp();
            // Beyond ~9x the rim width the light is below one part in 10k;
            // dropping it keeps the field exactly zero in the icon interior.
            if rim < 1.0e-4 {
                continue;
            }
            out[i] = (band * rim * params.strength).clamp(0.0, 1.0);
        }
    }
    out
}

/// Edge gloss: a screen-blend lighting pass applied on top of the composed
/// icon. The light follows the shape boundary and decays inwards, so the
/// artwork stays fully visible through the effect instead of being covered by
/// a painted ring; fully transparent pixels are left untouched.
pub fn apply_edge_gloss(
    body: &mut LinearPremultipliedImage,
    mask: &AlphaMask,
    config: &ValidatedRenderConfig,
) -> Result<(), AppError> {
    let Some(params) = &config.gloss else {
        return Ok(());
    };
    let size = 256usize;
    let light = parse_hex_rgba(&config.gloss_light_color)?;
    let half_extent = size as f32 / 2.0 - config.canvas_inset;
    let field = edge_glow_field(mask, size, half_extent, params);

    for (idx, &glow) in field.iter().enumerate() {
        if glow <= 0.0 {
            continue;
        }
        let dst_a = body.data[idx][3];
        if dst_a <= 0.0 {
            continue;
        }
        let t = (light.a * glow).clamp(0.0, 1.0);
        if t <= 0.0 {
            continue;
        }
        // Blend in straight alpha, then re-premultiply.
        let dst_r = body.data[idx][0] / dst_a;
        let dst_g = body.data[idx][1] / dst_a;
        let dst_b = body.data[idx][2] / dst_a;
        let out_r = 1.0 - (1.0 - dst_r) * (1.0 - light.r * t);
        let out_g = 1.0 - (1.0 - dst_g) * (1.0 - light.g * t);
        let out_b = 1.0 - (1.0 - dst_b) * (1.0 - light.b * t);
        body.data[idx][0] = out_r.min(1.0) * dst_a;
        body.data[idx][1] = out_g.min(1.0) * dst_a;
        body.data[idx][2] = out_b.min(1.0) * dst_a;
    }
    Ok(())
}
