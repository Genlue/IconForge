use std::f32::consts::PI;

use super::color::parse_hex_rgba;
use super::composite::{over, LinearPremultipliedImage};
use super::mask::{generate_inset_shape_mask, AlphaMask};
use super::pipeline::ValidatedRenderConfig;
use crate::domain::config::IconShape;
use crate::error::app_error::AppError;

/// Opacity of the highlight along the straight left/top edges, slightly below
/// the full-strength corner arcs.
const EDGE_LEVEL: f32 = 0.85;
/// Fraction of each straight edge used for the fade-in/fade-out at the ends
/// and for the transitions between the edge level and the corner level.
const RAMP_FRAC: f32 = 0.3;

/// Shared highlight profile across the two renderers.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlossProfile {
    /// Fraction of the top-left highlight's angular span covered by the left
    /// edge (position of the corner arc start).
    pub k1: f32,
    /// Fraction covered by the left edge plus the corner arc (position of the
    /// corner arc end / top edge start).
    pub k2: f32,
}

fn smoothstep01(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// Weight of the top-left highlight at angular position `phi` (0..span) along
/// its span: fades in from the upper end of the bottom-left rounding, holds
/// the slightly reduced edge level along the straight edges, peaks at full
/// strength across the top-left corner arc, then fades back out to the edge
/// level and finally to transparent at the left end of the top-right rounding.
pub(crate) fn region_a_weight(phi: f32, span: f32, profile: &GlossProfile) -> f32 {
    let s = (phi / span).clamp(0.0, 1.0);
    let k1 = profile.k1;
    let k2 = profile.k2;
    if k1 <= 0.001 && k2 >= 0.999 {
        // No straight edges (squircle/circle): the whole span is the corner
        // arc, fading in at the left end and out at the top end.
        let f_in = smoothstep01((s / RAMP_FRAC).clamp(0.0, 1.0));
        let f_out = 1.0 - smoothstep01(((s - (1.0 - RAMP_FRAC)) / RAMP_FRAC).clamp(0.0, 1.0));
        return f_in.min(f_out);
    }
    let t1 = k1 * RAMP_FRAC;
    let t2 = k1 * (1.0 - RAMP_FRAC);
    let t3 = k2 + (1.0 - k2) * RAMP_FRAC;
    let t4 = k2 + (1.0 - k2) * (1.0 - RAMP_FRAC);
    if s <= t1 {
        EDGE_LEVEL * smoothstep01((s / t1).clamp(0.0, 1.0))
    } else if s <= t2 {
        EDGE_LEVEL
    } else if s <= k1 {
        EDGE_LEVEL + (1.0 - EDGE_LEVEL) * smoothstep01(((s - t2) / (k1 - t2)).clamp(0.0, 1.0))
    } else if s <= k2 {
        1.0
    } else if s <= t3 {
        1.0 - (1.0 - EDGE_LEVEL) * smoothstep01(((s - k2) / (t3 - k2)).clamp(0.0, 1.0))
    } else if s <= t4 {
        EDGE_LEVEL
    } else {
        EDGE_LEVEL * (1.0 - smoothstep01(((s - t4) / (1.0 - t4)).clamp(0.0, 1.0)))
    }
}

/// Weight of the bottom-right highlight at arc position `u` (0..1 from the
/// left side of the rounding to its upper side): fades in from the edge base
/// color at the left side, holds the same reduced level as the straight edges
/// across the middle, then fades back out to transparent at the upper side.
pub(crate) fn bottom_right_weight(u: f32) -> f32 {
    let fade_in = smoothstep01((u / 0.35).clamp(0.0, 1.0));
    let fade_out = 1.0 - smoothstep01(((u - 0.65) / 0.35).clamp(0.0, 1.0));
    EDGE_LEVEL * fade_in.min(fade_out)
}

/// Angular spans (in radians, y-down screen coordinates where the angle grows
/// clockwise) of the two highlight regions, measured from the shape center:
/// `(a1, a2, b1, b2)`, plus the shared opacity profile.
///
/// Top-left highlight: the boundary arc from the upper end of the bottom-left
/// rounding, clockwise along the left edge and top edge, to the left end of the
/// top-right rounding (region A, angular sweep `a1 -> a2`).
///
/// Bottom-right highlight: the bottom-right rounding arc from its left side
/// (bottom edge end) to its upper side (right edge end), region B = the angle
/// interval `[b2, b1]`.
fn gloss_regions(config: &ValidatedRenderConfig) -> ((f32, f32, f32, f32), GlossProfile) {
    match config.shape {
        IconShape::Squircle => (
            (PI, -PI / 2.0, PI / 2.0, 0.0),
            GlossProfile { k1: 0.0, k2: 1.0 },
        ),
        IconShape::Rectangle => rounded_rect_regions(config, 0.0),
        IconShape::RoundedRectangle => {
            let inner_half = 128.0 - config.canvas_inset;
            let r = config.corner_radius.clamp(0.0, inner_half);
            rounded_rect_regions(config, r)
        }
    }
}

fn rounded_rect_regions(config: &ValidatedRenderConfig, r: f32) -> ((f32, f32, f32, f32), GlossProfile) {
    let x0 = config.canvas_inset;
    let x1 = 256.0 - config.canvas_inset;
    let y0 = config.canvas_inset;
    let y1 = 256.0 - config.canvas_inset;
    let c = 128.0;
    let a1 = (y1 - r - c).atan2(x0 - c);
    let a2 = (y0 - c).atan2(x1 - r - c);
    let b1 = (y1 - c).atan2(x1 - r - c);
    let b2 = (y1 - r - c).atan2(x1 - c);
    let span_a = (a2 - a1).rem_euclid(std::f32::consts::TAU);
    // Positions of the corner arc start/end along the top-left span.
    let k1 = ((y0 + r - c).atan2(x0 - c) - a1).rem_euclid(std::f32::consts::TAU) / span_a;
    let k2 = ((y0 - c).atan2(x0 + r - c) - a1).rem_euclid(std::f32::consts::TAU) / span_a;
    ((a1, a2, b1, b2), GlossProfile { k1, k2 })
}

/// Edge gloss: a user-colored ring along the whole icon edge, with a light
/// highlight overlaid on top of it along the left/top edges (region A) and on
/// the bottom-right corner arc (region B). The highlight peaks at the corner
/// arcs, sits slightly lower along the straight edges, and fades to
/// transparent at the ends of both regions so the edge base color shows
/// through with a gradual transition.
pub fn render_edge_gloss(
    mask: &AlphaMask,
    config: &ValidatedRenderConfig,
) -> Result<LinearPremultipliedImage, AppError> {
    let size = 256u32;
    let inset = generate_inset_shape_mask(size, config, config.canvas_inset + config.gloss_width);
    // Softening the inner boundary of the gloss band smooths the transition
    // between the edge highlight and the icon body below it.
    let inset = if config.gloss_feather_blur > 0.0 {
        super::shadow::gaussian_blur_alpha(&inset, size, size, config.gloss_feather_blur)
    } else {
        inset
    };
    let light = parse_hex_rgba(&config.gloss_light_color)?;
    let base = parse_hex_rgba(&config.gloss_base_color)?;
    let ((a1, a2, b1, b2), profile) = gloss_regions(config);
    let span_a = (a2 - a1).rem_euclid(std::f32::consts::TAU);
    let span_b = b1 - b2;

    let mut result = LinearPremultipliedImage::transparent(size, size);
    let mut highlight = LinearPremultipliedImage::transparent(size, size);
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) as usize;
            let edge = (mask[idx] - inset[idx]).clamp(0.0, 1.0);
            if edge <= 0.0 {
                continue;
            }
            let a = base.a * edge * config.gloss_strength;
            result.data[idx] = [base.r * a, base.g * a, base.b * a, a];

            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let theta = (py - 128.0).atan2(px - 128.0);
            let phi = (theta - a1).rem_euclid(std::f32::consts::TAU);
            let weight = if phi <= span_a {
                region_a_weight(phi, span_a, &profile)
            } else if span_b > 0.0 && theta >= b2 && theta <= b1 {
                bottom_right_weight((b1 - theta) / span_b)
            } else {
                continue;
            };
            let alpha = light.a * edge * weight * config.gloss_strength;
            if alpha <= 0.0 {
                continue;
            }
            highlight.data[idx] = [light.r * alpha, light.g * alpha, light.b * alpha, alpha];
        }
    }
    over(&mut result, highlight);
    Ok(result)
}
