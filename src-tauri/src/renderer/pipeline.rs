use image::RgbaImage;

use crate::domain::config::{BackplateType, BrushStroke, ForegroundFit, IconShape, RenderConfig, WandStroke};
use crate::error::app_error::AppError;

use super::color::{parse_hex_rgba, LinearRgba};
use super::composite::{over, LinearPremultipliedImage};
use super::hq::{validate_hq, ValidatedHqRenderConfig};
use super::mask::generate_shape_mask;
use super::resize::resize_from_master;
use super::shadow::render_shadow;
use super::stroke::render_inner_stroke;
use super::transform::render_foreground;

#[derive(Debug, Clone)]
pub struct ValidatedRenderConfig {
    pub shape: IconShape,
    pub foreground_scale_percent: f32,
    pub foreground_fit: ForegroundFit,
    pub foreground_offset_x: f32,
    pub foreground_offset_y: f32,
    pub foreground_rotation_degrees: f32,
    pub canvas_inset: f32,
    pub corner_radius: f32,
    pub squircle_exponent: f32,
    pub backplate_type: BackplateType,
    pub backplate_color: LinearRgba,
    pub gradient_start_color: LinearRgba,
    pub gradient_end_color: LinearRgba,
    pub gradient_angle_degrees: f32,
    pub outer_shadow_enabled: bool,
    pub outer_shadow_offset_x: f32,
    pub outer_shadow_offset_y: f32,
    pub outer_shadow_blur_radius: f32,
    pub outer_shadow_spread: f32,
    pub outer_shadow_color: String,
    pub stroke_width: f32,
    pub stroke_color: String,
    pub gloss_enabled: bool,
    pub gloss_width: f32,
    pub gloss_strength: f32,
    pub gloss_light_color: String,
    pub gloss_dark_color: String,
    pub auto_cutout_enabled: bool,
    pub auto_cutout_tolerance: f32,
    pub auto_cutout_feather: f32,
    pub hq: Option<ValidatedHqRenderConfig>,
}

pub fn validate_config(config: &RenderConfig) -> Result<ValidatedRenderConfig, AppError> {
    if !config.foreground_scale_percent.is_finite()
        || !(10.0..=300.0).contains(&config.foreground_scale_percent)
    {
        return Err(AppError::InvalidArgument(
            "foregroundScalePercent must be in 10..=300".into(),
        ));
    }

    // Validate all color strings upfront
    let _backplate = parse_hex_rgba(&config.backplate_color)?;
    let _grad_start = parse_hex_rgba(&config.gradient_start_color)?;
    let _grad_end = parse_hex_rgba(&config.gradient_end_color)?;
    let _shadow_color = parse_hex_rgba(&config.outer_shadow.color)?;
    let _stroke_color = parse_hex_rgba(&config.stroke.color)?;
    let _gloss_light = parse_hex_rgba(&config.gloss.light_color)?;
    let _gloss_dark = parse_hex_rgba(&config.gloss.dark_color)?;

    if !config.foreground_offset_x.is_finite()
        || !(-128.0..=128.0).contains(&config.foreground_offset_x)
    {
        return Err(AppError::InvalidArgument(
            "foregroundOffsetX out of range".into(),
        ));
    }

    if !config.foreground_offset_y.is_finite()
        || !(-128.0..=128.0).contains(&config.foreground_offset_y)
    {
        return Err(AppError::InvalidArgument(
            "foregroundOffsetY out of range".into(),
        ));
    }

    if !config.foreground_rotation_degrees.is_finite()
        || !(-180.0..=180.0).contains(&config.foreground_rotation_degrees)
    {
        return Err(AppError::InvalidArgument(
            "foregroundRotationDegrees out of range".into(),
        ));
    }

    if !config.canvas_inset.is_finite() || !(0.0..=112.0).contains(&config.canvas_inset) {
        return Err(AppError::InvalidArgument(
            "canvasInset must be in 0..=112".into(),
        ));
    }

    if config.shape == IconShape::RoundedRectangle
        && (!config.corner_radius.is_finite() || !(0.0..=128.0).contains(&config.corner_radius))
    {
        return Err(AppError::InvalidArgument(
            "cornerRadius out of range".into(),
        ));
    }

    if config.shape == IconShape::Squircle
        && (!config.squircle_exponent.is_finite()
            || !(2.0..=8.0).contains(&config.squircle_exponent))
    {
        return Err(AppError::InvalidArgument(
            "squircleExponent out of range".into(),
        ));
    }

    if !config.gradient_angle_degrees.is_finite()
        || !(0.0..=360.0).contains(&config.gradient_angle_degrees)
    {
        return Err(AppError::InvalidArgument(
            "gradientAngleDegrees must be in 0..=360".into(),
        ));
    }

    let shadow = &config.outer_shadow;
    if !shadow.offset_x.is_finite() || !(-64.0..=64.0).contains(&shadow.offset_x) {
        return Err(AppError::InvalidArgument(
            "shadow offsetX out of range".into(),
        ));
    }
    if !shadow.offset_y.is_finite() || !(-64.0..=64.0).contains(&shadow.offset_y) {
        return Err(AppError::InvalidArgument(
            "shadow offsetY out of range".into(),
        ));
    }
    if !shadow.blur_radius.is_finite() || !(0.0..=64.0).contains(&shadow.blur_radius) {
        return Err(AppError::InvalidArgument(
            "shadow blurRadius must be in 0..=64".into(),
        ));
    }
    if !shadow.spread.is_finite() || !(0.0..=32.0).contains(&shadow.spread) {
        return Err(AppError::InvalidArgument(
            "shadow spread must be in 0..=32".into(),
        ));
    }
    if !config.stroke.width.is_finite() || !(0.0..=32.0).contains(&config.stroke.width) {
        return Err(AppError::InvalidArgument(
            "stroke width must be in 0..=32".into(),
        ));
    }
    if !config.gloss.width.is_finite() || !(0.0..=32.0).contains(&config.gloss.width) {
        return Err(AppError::InvalidArgument(
            "gloss width must be in 0..=32".into(),
        ));
    }
    if !config.gloss.strength.is_finite() || !(0.0..=1.0).contains(&config.gloss.strength) {
        return Err(AppError::InvalidArgument(
            "gloss strength must be in 0..=1".into(),
        ));
    }
    if !config.auto_cutout.tolerance.is_finite()
        || !(0.0..=100.0).contains(&config.auto_cutout.tolerance)
    {
        return Err(AppError::InvalidArgument(
            "autoCutout tolerance must be in 0..=100".into(),
        ));
    }
    if !config.auto_cutout.feather.is_finite()
        || !(0.0..=32.0).contains(&config.auto_cutout.feather)
    {
        return Err(AppError::InvalidArgument(
            "autoCutout feather must be in 0..=32".into(),
        ));
    }

    let hq = if config.hq_render.enabled {
        Some(validate_hq(config)?)
    } else {
        None
    };

    Ok(ValidatedRenderConfig {
        shape: config.shape,
        foreground_scale_percent: config.foreground_scale_percent,
        foreground_fit: config.foreground_fit,
        foreground_offset_x: config.foreground_offset_x,
        foreground_offset_y: config.foreground_offset_y,
        foreground_rotation_degrees: config.foreground_rotation_degrees,
        canvas_inset: config.canvas_inset,
        corner_radius: config.corner_radius,
        squircle_exponent: config.squircle_exponent,
        backplate_type: config.backplate_type,
        backplate_color: parse_hex_rgba(&config.backplate_color)?,
        gradient_start_color: parse_hex_rgba(&config.gradient_start_color)?,
        gradient_end_color: parse_hex_rgba(&config.gradient_end_color)?,
        gradient_angle_degrees: config.gradient_angle_degrees,
        outer_shadow_enabled: config.outer_shadow.enabled,
        outer_shadow_offset_x: config.outer_shadow.offset_x,
        outer_shadow_offset_y: config.outer_shadow.offset_y,
        outer_shadow_blur_radius: config.outer_shadow.blur_radius,
        outer_shadow_spread: config.outer_shadow.spread,
        outer_shadow_color: config.outer_shadow.color.clone(),
        stroke_width: config.stroke.width,
        stroke_color: config.stroke.color.clone(),
        gloss_enabled: config.gloss.enabled,
        gloss_width: config.gloss.width,
        gloss_strength: config.gloss.strength,
        gloss_light_color: config.gloss.light_color.clone(),
        gloss_dark_color: config.gloss.dark_color.clone(),
        auto_cutout_enabled: config.auto_cutout.enabled,
        auto_cutout_tolerance: config.auto_cutout.tolerance,
        auto_cutout_feather: config.auto_cutout.feather,
        hq,
    })
}

pub fn render_master(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
) -> Result<RgbaImage, AppError> {
    // Auto cutout applies to the original source for both the classic and the
    // adaptive high-quality pipelines.
    let cutout_buffer;
    let prepared = if config.auto_cutout_enabled {
        cutout_buffer = super::cutout::remove_border_background(
            source,
            config.auto_cutout_tolerance,
            config.auto_cutout_feather,
        );
        &cutout_buffer
    } else {
        source
    };

    // Adaptive high-quality renderer (port of iconmask.py) replaces the
    // classic layer stack entirely when enabled.
    if let Some(hq) = &config.hq {
        return super::hq::render_hq(prepared, hq);
    }

    let size = 256;
    let mask = generate_shape_mask(size, config);
    let mut body = LinearPremultipliedImage::transparent(size, size);

    // Backplate
    if config.backplate_type != BackplateType::None {
        let backplate = render_backplate(&mask, config, size);
        over(&mut body, backplate);
    }

    // Foreground
    let foreground = render_foreground(prepared, config, &mask);
    over(&mut body, foreground);

    // Stroke
    if config.stroke_width > 0.0 {
        let stroke = render_inner_stroke(&mask, config);
        over(&mut body, stroke);
    }

    if config.gloss_enabled && config.gloss_width > 0.0 && config.gloss_strength > 0.0 {
        over(&mut body, super::gloss::render_edge_gloss(&mask, config)?);
    }

    // Determine silhouette for shadow
    let silhouette = if config.backplate_type != BackplateType::None {
        mask.clone()
    } else {
        body.alpha_plane()
    };

    // Shadow
    let mut result = if config.outer_shadow_enabled {
        render_shadow(&silhouette, config)
    } else {
        LinearPremultipliedImage::transparent(size, size)
    };
    over(&mut result, body);

    Ok(result.into_srgb_rgba8())
}

fn render_backplate(
    mask: &[f32],
    config: &ValidatedRenderConfig,
    size: u32,
) -> LinearPremultipliedImage {
    let mut result = LinearPremultipliedImage::transparent(size, size);

    match config.backplate_type {
        BackplateType::None => {}
        BackplateType::Solid => {
            let c = &config.backplate_color;
            for (i, &coverage) in mask.iter().enumerate() {
                if coverage <= 0.0 {
                    continue;
                }
                let a = (c.a * coverage).min(1.0);
                result.data[i][0] = c.r * a;
                result.data[i][1] = c.g * a;
                result.data[i][2] = c.b * a;
                result.data[i][3] = a;
            }
        }
        BackplateType::Gradient => {
            let theta = config.gradient_angle_degrees * std::f32::consts::PI / 180.0;
            let dx = theta.cos();
            let dy = theta.sin();
            let half = size as f32 / 2.0;

            // Compute min/max projection
            let corners = [(-half, -half), (half, -half), (half, half), (-half, half)];
            let proj: Vec<f32> = corners.iter().map(|(x, y)| x * dx + y * dy).collect();
            let min_t = proj.iter().cloned().fold(f32::INFINITY, f32::min);
            let max_t = proj.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let range = max_t - min_t;

            let start = &config.gradient_start_color;
            let end = &config.gradient_end_color;

            for (i, &coverage) in mask.iter().enumerate() {
                if coverage <= 0.0 {
                    continue;
                }
                let y = (i as u32 / size) as f32 - half;
                let x = (i as u32 % size) as f32 - half;
                let t = if range > 0.0 {
                    ((x * dx + y * dy) - min_t) / range
                } else {
                    0.5
                };
                let t = t.clamp(0.0, 1.0);
                let r = start.r + (end.r - start.r) * t;
                let g = start.g + (end.g - start.g) * t;
                let b = start.b + (end.b - start.b) * t;
                let a = (start.a + (end.a - start.a) * t) * coverage;
                result.data[i][0] = r * a;
                result.data[i][1] = g * a;
                result.data[i][2] = b * a;
                result.data[i][3] = a;
            }
        }
    }

    result
}

pub fn render_icon_set(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
) -> Result<Vec<(u32, RgbaImage)>, AppError> {
    let master = render_master(source, config)?;
    let mut results = Vec::with_capacity(super::ICO_SIZES.len());

    for &size in &super::ICO_SIZES {
        if size == 256 {
            results.push((size, master.clone()));
        } else {
            let resized = resize_from_master(&master, size)?;
            results.push((size, resized));
        }
    }

    Ok(results)
}

/// Render a master with magic-wand erasures applied to the original source
/// before the (classic or HQ) pipeline stages run.
pub fn render_master_with_wands(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
    wand_strokes: &[WandStroke],
) -> Result<RgbaImage, AppError> {
    if wand_strokes.is_empty() {
        return render_master(source, config);
    }
    let mut edited = source.clone();
    super::wand::apply_erasures(&mut edited, wand_strokes);
    render_master(&edited, config)
}

pub fn render_icon_set_with_brushes(
    source: &RgbaImage,
    config: &ValidatedRenderConfig,
    brush_strokes: &[BrushStroke],
    wand_strokes: &[WandStroke],
) -> Result<Vec<(u32, RgbaImage)>, AppError> {
    let master = super::brush::composite_brush_strokes(
        render_master_with_wands(source, config, wand_strokes)?,
        brush_strokes,
    )?;
    let mut results = Vec::with_capacity(super::ICO_SIZES.len());
    for &size in &super::ICO_SIZES {
        if size == 256 {
            results.push((size, master.clone()));
        } else {
            results.push((size, resize_from_master(&master, size)?));
        }
    }
    Ok(results)
}
