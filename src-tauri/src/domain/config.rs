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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum HqPanelShape {
    Rect,
    Squircle,
    Circle,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HqShadowMode {
    #[default]
    Icon,
    Badge,
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
pub struct GlossConfig {
    pub enabled: bool,
    pub width: f32,
    pub strength: f32,
    pub light_color: String,
    pub dark_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AutoCutoutConfig {
    pub enabled: bool,
    pub tolerance: f32,
    pub feather: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HqRenderConfig {
    pub enabled: bool,
    pub thresh: f32,
    pub icon_ratio: f32,
    pub bg: f32,
    pub light_mix: f32,
    pub dark_mix: f32,
    pub gloss: f32,
    pub icon_light: f32,
    pub corner: f32,
    pub shape: HqPanelShape,
    pub shadow_opacity: f32,
    pub shadow_blur_factor: f32,
    pub shadow_offset_factor: f32,
    pub shadow_fade: f32,
    pub shadow_mode: HqShadowMode,
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
    pub gloss: GlossConfig,
    pub auto_cutout: AutoCutoutConfig,
    pub hq_render: HqRenderConfig,
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
    #[serde(default)]
    pub mode: BrushMode,
    #[serde(default)]
    pub clip_to_mask: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrushMode {
    #[default]
    Paint,
    Erase,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WandPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WandStroke {
    pub points: Vec<WandPoint>,
    pub tolerance: f32,
}
