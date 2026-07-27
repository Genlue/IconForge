use serde::Deserialize;

use super::config::RenderConfig;
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
    pub preview_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportIcoRequest {
    pub source_paths: Vec<String>,
    pub render_config: RenderConfig,
    pub export_mode: ExportMode,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ApplyToLnkRequest {
    pub lnk_paths: Vec<String>,
    pub render_config: RenderConfig,
    pub export_mode: ExportMode,
}
