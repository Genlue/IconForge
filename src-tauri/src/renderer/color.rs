use crate::error::app_error::AppError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn parse_hex_rgba(value: &str) -> Result<LinearRgba, AppError> {
    let hex = value
        .strip_prefix('#')
        .ok_or_else(|| AppError::InvalidArgument(format!("color must start with #: {}", value)))?;

    if hex.len() != 8 {
        return Err(AppError::InvalidArgument(format!(
            "color must be 8 hex digits (RRGGBBAA): {}",
            value
        )));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)
        .map_err(|_| AppError::InvalidArgument(format!("invalid hex color: {}", value)))?;
    let g = u8::from_str_radix(&hex[2..4], 16)
        .map_err(|_| AppError::InvalidArgument(format!("invalid hex color: {}", value)))?;
    let b = u8::from_str_radix(&hex[4..6], 16)
        .map_err(|_| AppError::InvalidArgument(format!("invalid hex color: {}", value)))?;
    let a = u8::from_str_radix(&hex[6..8], 16)
        .map_err(|_| AppError::InvalidArgument(format!("invalid hex color: {}", value)))?;

    Ok(LinearRgba {
        r: srgb_channel_to_linear(r as f32 / 255.0),
        g: srgb_channel_to_linear(g as f32 / 255.0),
        b: srgb_channel_to_linear(b as f32 / 255.0),
        a: a as f32 / 255.0,
    })
}

pub fn srgb_channel_to_linear(value: f32) -> f32 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_channel_to_srgb(value: f32) -> f32 {
    let clamped = value.clamp(0.0, 1.0);
    if clamped <= 0.0031308 {
        12.92 * clamped
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    }
}
