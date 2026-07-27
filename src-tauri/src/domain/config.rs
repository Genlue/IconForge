use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum IconShape {
    Rectangle,
    RoundedRectangle,
    Squircle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum BackplateType {
    None,
    Solid,
    Gradient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ForegroundFit {
    Contain,
    Cover,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OuterShadowConfig {
    pub enabled: bool,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StrokeConfig {
    pub width: f32,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderConfig {
    pub foreground_scale_percent: f32,
    pub foreground_fit: ForegroundFit,
    pub foreground_offset_x: f32,
    pub foreground_offset_y: f32,
    pub foreground_rotation_degrees: f32,
    pub canvas_inset: f32,
    pub shape: IconShape,
    pub corner_radius: f32,
    pub squircle_exponent: f32,
    pub backplate_type: BackplateType,
    pub backplate_color: String,
    pub gradient_start_color: String,
    pub gradient_end_color: String,
    pub gradient_angle_degrees: f32,
    pub outer_shadow: OuterShadowConfig,
    pub stroke: StrokeConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpscaleConfig {
    pub enabled: bool,
    pub scale: u32,
    pub denoise: i32,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrushPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrushStroke {
    pub points: Vec<BrushPoint>,
    pub color: String,
    pub size: f32,
    pub opacity: f32,
}
