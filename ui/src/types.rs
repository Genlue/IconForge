//! 前端自己的视图状态类型。
//!
//! 凡是「跨 IPC 的结构」一律来自 `iconforge_domain`（宿主与前端共用一份），
//! 这里只放纯前端概念：工具选择、检查器页签、预设、预览快照。

pub use iconforge_domain::*;

use serde::{Deserialize, Serialize};

/// 预览画布边长（逻辑像素）。与宿主 `preview_size` 一致。
pub const PREVIEW_SIZE: u32 = 420;
/// 渲染坐标系：所有笔迹、魔棒坐标都以 256 为基准。
pub const CANVAS_SPACE: f32 = 256.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Tool {
    #[default]
    None,
    Brush,
    Eraser,
    Eyedropper,
    MagicWand,
    SourceEraser,
}

impl Tool {
    pub fn hint(self) -> &'static str {
        match self {
            Tool::None => "",
            Tool::Brush => "在预览上拖动绘制",
            Tool::Eraser => "在预览上拖动擦除",
            Tool::Eyedropper => "点击预览取色",
            Tool::MagicWand => "在原图编辑中点击选中相似区域",
            Tool::SourceEraser => "在原图编辑中涂抹擦除",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorTarget {
    Brush,
    Backplate,
    GradientStart,
    GradientEnd,
    Shadow,
    Stroke,
    GlossLight,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum InspectorTab {
    #[default]
    Style,
    Source,
    Export,
}

impl InspectorTab {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Style => "style",
            Self::Source => "source",
            Self::Export => "export",
        }
    }
    pub fn from_str(value: &str) -> Self {
        match value {
            "source" => Self::Source,
            "export" => Self::Export,
            _ => Self::Style,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

/// 单个导入项的完整编辑状态。渲染配置之外，画笔 / 魔棒 / 橡皮擦笔迹
/// 也属于它 —— 换一个图标就是换一套笔迹，不能混。
#[derive(Clone, Debug)]
pub struct ItemConfig {
    pub render: RenderConfig,
    pub upscale: UpscaleConfig,
    pub brush_strokes: Vec<BrushStroke>,
    pub brush_color: String,
    pub brush_size: f32,
    pub brush_mode: BrushMode,
    pub brush_clip_to_mask: bool,
    pub wand_strokes: Vec<WandStroke>,
    pub wand_tolerance: f32,
    pub eraser_strokes: Vec<EraserStroke>,
    pub eraser_size: f32,
    pub eraser_hardness: f32,
    pub active_preset: Option<String>,
}

impl ItemConfig {
    pub fn render_request(&self, source_path: String) -> RenderPreviewRequest {
        RenderPreviewRequest {
            source_path,
            render_config: self.render.clone(),
            upscale_config: self.upscale.clone(),
            brush_strokes: self.brush_strokes.clone(),
            wand_strokes: self.wand_strokes.clone(),
            eraser_strokes: self.eraser_strokes.clone(),
            preview_size: PREVIEW_SIZE,
        }
    }

    pub fn export_item(&self, source_path: String) -> ExportIcoItemRequest {
        ExportIcoItemRequest {
            source_path,
            render_config: self.render.clone(),
            upscale_config: self.upscale.clone(),
            brush_strokes: self.brush_strokes.clone(),
            wand_strokes: self.wand_strokes.clone(),
            eraser_strokes: self.eraser_strokes.clone(),
        }
    }
}

/// 一次待确认的魔棒选区（蚂蚁线画的就是它）。
#[derive(Clone, Debug, PartialEq)]
pub struct WandSelection {
    pub x: f32,
    pub y: f32,
    pub tolerance: f32,
    pub mask_png_base64: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreviewState {
    /// 已经带 `data:image/png;base64,` 前缀，可直接进 `<img src>`。
    pub data_url: Option<String>,
    pub processed: Option<(u32, u32)>,
    pub loading: bool,
    pub error: Option<CommandError>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: RenderConfig,
    #[serde(default)]
    pub custom: bool,
}

#[derive(Clone, Debug)]
pub enum ExportResult {
    Ico(ExportIcoResponse),
    Lnk(ApplyToLnkResponse),
}

impl ExportResult {
    pub fn success_count(&self) -> usize {
        match self {
            Self::Ico(r) => r.files.len(),
            Self::Lnk(r) => r.applied.len(),
        }
    }
    pub fn issue_count(&self) -> usize {
        match self {
            Self::Ico(r) => r.warnings.len(),
            Self::Lnk(r) => r.failed.len(),
        }
    }
    pub fn rows(&self) -> Vec<(String, String, bool)> {
        match self {
            Self::Ico(r) => {
                let mut rows: Vec<(String, String, bool)> = r
                    .files
                    .iter()
                    .map(|f| ("文件已保存".to_string(), f.output_path.clone(), false))
                    .collect();
                rows.extend(
                    r.warnings
                        .iter()
                        .map(|w| ("提示".to_string(), w.clone(), true)),
                );
                rows
            }
            Self::Lnk(r) => {
                let mut rows: Vec<(String, String, bool)> = r
                    .applied
                    .iter()
                    .map(|a| ("快捷方式已更新".to_string(), a.lnk_path.clone(), false))
                    .collect();
                rows.extend(r.failed.iter().map(|f| {
                    (
                        format!("失败：{}", f.code),
                        format!("{}\n{}", f.path, f.message),
                        true,
                    )
                }));
                rows
            }
        }
    }
}
