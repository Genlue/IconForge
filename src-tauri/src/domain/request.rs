use serde::Deserialize;

use super::config::{BrushStroke, EraserStroke, RenderConfig, UpscaleConfig, WandPoint, WandStroke};
use super::input::ExportMode;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportPathsRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderPreviewRequest {
    pub source_path: String,
    pub render_config: RenderConfig,
    pub upscale_config: UpscaleConfig,
    #[serde(default)]
    pub brush_strokes: Vec<BrushStroke>,
    #[serde(default)]
    pub wand_strokes: Vec<WandStroke>,
    #[serde(default)]
    pub eraser_strokes: Vec<EraserStroke>,
    pub preview_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WandSelectionRequest {
    pub source_path: String,
    pub upscale_config: UpscaleConfig,
    pub point: WandPoint,
    pub tolerance: f32,
    #[serde(default)]
    pub wand_strokes: Vec<WandStroke>,
    #[serde(default)]
    pub eraser_strokes: Vec<EraserStroke>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceImageRequest {
    pub source_path: String,
    pub upscale_config: UpscaleConfig,
    #[serde(default)]
    pub wand_strokes: Vec<WandStroke>,
    #[serde(default)]
    pub eraser_strokes: Vec<EraserStroke>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportIcoRequest {
    pub items: Vec<ExportIcoItemRequest>,
    pub export_mode: ExportMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportIcoItemRequest {
    pub source_path: String,
    pub render_config: RenderConfig,
    pub upscale_config: UpscaleConfig,
    #[serde(default)]
    pub brush_strokes: Vec<BrushStroke>,
    #[serde(default)]
    pub wand_strokes: Vec<WandStroke>,
    #[serde(default)]
    pub eraser_strokes: Vec<EraserStroke>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportPngRequest {
    pub items: Vec<ExportIcoItemRequest>,
    #[serde(default)]
    pub raw_source: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyToLnkRequest {
    pub items: Vec<ApplyToLnkItemRequest>,
    pub export_mode: ExportMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyToLnkItemRequest {
    pub lnk_path: String,
    pub render_config: RenderConfig,
    pub upscale_config: UpscaleConfig,
    #[serde(default)]
    pub brush_strokes: Vec<BrushStroke>,
    #[serde(default)]
    pub wand_strokes: Vec<WandStroke>,
    #[serde(default)]
    pub eraser_strokes: Vec<EraserStroke>,
}
