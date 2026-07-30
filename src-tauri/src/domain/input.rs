use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum InputFileType {
    Image,
    Exe,
    Lnk,
    Directory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ExportMode {
    ExportAsIco,
    ExportAsPng,
    ApplyToLnk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputItem {
    pub id: String,
    pub source_path: String,
    pub display_name: String,
    pub file_type: InputFileType,
    pub parent_directory_path: Option<String>,
    pub resolved_target_path: Option<String>,
    pub source_width: u32,
    pub source_height: u32,
    pub thumbnail_png_base64: String,
    pub supported_export_modes: Vec<ExportMode>,
    pub warnings: Vec<String>,
}
