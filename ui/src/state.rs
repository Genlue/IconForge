//! 应用状态与全部动作。
//!
//! 状态是「一组 `RwSignal` 组成的一个 `Copy` 句柄」，不是单一大 struct 的
//! 深度嵌套：组件只订阅自己读到的信号，改一个滑杆不会重渲染整个检查器。
//!
//! 所有 IPC 都从这一层发起，组件只调用方法、不自己拼请求 —— 这样
//! 「预览、导出、应用快捷方式」三条路径用的是同一份配置快照。

use leptos::prelude::*;
use leptos::task::spawn_local;

use glass_design::prelude::{toast, toast_with, Tone};

use crate::api;
use crate::defaults;
use crate::storage;
use crate::types::*;
use crate::util;

const CUSTOM_PRESETS_KEY: &str = "iconforge.customPresets.v1";
const SETTINGS_KEY: &str = "iconforge.settings.v1";

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct PersistedSettings {
    theme: ThemePreference,
}

#[derive(Clone, Copy)]
pub struct App {
    pub items: RwSignal<Vec<InputItem>>,
    pub selected: RwSignal<Option<String>>,
    pub configs: RwSignal<Vec<(String, ItemConfig)>>,

    pub tool: RwSignal<Tool>,
    pub color_target: RwSignal<Option<ColorTarget>>,
    pub wand_selection: RwSignal<Option<WandSelection>>,
    pub preview: RwSignal<PreviewState>,

    pub importing: RwSignal<bool>,
    pub exporting: RwSignal<bool>,
    pub drag_active: RwSignal<bool>,
    pub error: RwSignal<Option<CommandError>>,
    pub export_result: RwSignal<Option<ExportResult>>,
    pub custom_presets: RwSignal<Vec<Preset>>,

    pub source_editor_open: RwSignal<bool>,
    pub settings_open: RwSignal<bool>,
    pub theme_pref: RwSignal<ThemePreference>,
    pub inspector_tab: RwSignal<InspectorTab>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let settings: PersistedSettings =
            storage::read_json(SETTINGS_KEY).unwrap_or_default();
        Self {
            items: RwSignal::new(Vec::new()),
            selected: RwSignal::new(None),
            configs: RwSignal::new(Vec::new()),
            tool: RwSignal::new(Tool::None),
            color_target: RwSignal::new(None),
            wand_selection: RwSignal::new(None),
            preview: RwSignal::new(PreviewState::default()),
            importing: RwSignal::new(false),
            exporting: RwSignal::new(false),
            drag_active: RwSignal::new(false),
            error: RwSignal::new(None),
            export_result: RwSignal::new(None),
            custom_presets: RwSignal::new(storage::read_json_array(CUSTOM_PRESETS_KEY)),
            source_editor_open: RwSignal::new(false),
            settings_open: RwSignal::new(false),
            theme_pref: RwSignal::new(settings.theme),
            inspector_tab: RwSignal::new(InspectorTab::Style),
        }
    }

    pub fn persist_settings(&self) {
        storage::write_json(
            SETTINGS_KEY,
            &PersistedSettings {
                theme: self.theme_pref.get_untracked(),
            },
        );
    }

    // ---------------------------------------------------------------- 读取

    pub fn current_item(&self) -> Option<InputItem> {
        let selected = self.selected.get();
        self.items.with(|items| match selected {
            Some(id) => items.iter().find(|item| item.id == id).cloned(),
            None => items.first().cloned(),
        })
    }

    pub fn current_config(&self) -> Option<ItemConfig> {
        let selected = self.selected.get()?;
        self.configs
            .with(|list| list.iter().find(|(id, _)| *id == selected).map(|(_, cfg)| cfg.clone()))
    }

    pub fn item_count(&self) -> usize {
        self.items.with(Vec::len)
    }

    pub fn has_lnk(&self) -> bool {
        self.items
            .with(|items| items.iter().any(|item| item.file_type == InputFileType::Lnk))
    }

    pub fn has_ico_source(&self) -> bool {
        self.items.with(|items| {
            items
                .iter()
                .any(|item| item.source_path.to_lowercase().ends_with(".ico"))
        })
    }

    pub fn all_presets(&self) -> Vec<Preset> {
        let mut presets = defaults::built_in_presets();
        presets.extend(self.custom_presets.get());
        presets
    }

    pub fn hq_active(&self) -> bool {
        self.current_config()
            .map(|cfg| cfg.render.hq_render.enabled)
            .unwrap_or(false)
    }

    // ------------------------------------------------------- 推导值（给控件）

    /// 从当前图标的渲染配置里取一个字段，包成信号。
    ///
    /// 检查器分组的函数体只跑一次，裸值会被钉死在创建那一刻；把「取字段」
    /// 做成信号，控件才知道自己该跟着哪次更新重画。
    pub fn render_value<T>(
        &self,
        extract: impl Fn(&RenderConfig) -> T + Send + Sync + 'static,
    ) -> Signal<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let app = *self;
        Signal::derive(move || {
            let render = app
                .current_config()
                .map(|config| config.render)
                .unwrap_or_else(defaults::default_render_config);
            extract(&render)
        })
    }

    pub fn upscale_value<T>(
        &self,
        extract: impl Fn(&UpscaleConfig) -> T + Send + Sync + 'static,
    ) -> Signal<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let app = *self;
        Signal::derive(move || {
            let upscale = app
                .current_config()
                .map(|config| config.upscale)
                .unwrap_or_else(defaults::default_upscale_config);
            extract(&upscale)
        })
    }

    pub fn config_value<T>(
        &self,
        extract: impl Fn(&ItemConfig) -> T + Send + Sync + 'static,
    ) -> Signal<T>
    where
        T: Send + Sync + Clone + 'static,
    {
        let app = *self;
        Signal::derive(move || {
            let config = app.current_config().unwrap_or_else(defaults::default_item_config);
            extract(&config)
        })
    }

    // ---------------------------------------------------------------- 导入

    pub fn import_paths(&self, paths: Vec<String>) {
        if paths.is_empty() {
            return;
        }
        self.importing.set(true);
        self.error.set(None);
        let app = *self;
        spawn_local(async move {
            match api::import_paths(paths).await {
                Ok(response) => app.ingest(response),
                Err(err) => app.error.set(Some(err)),
            }
            app.importing.set(false);
        });
    }

    pub fn pick_and_import(&self) {
        let app = *self;
        spawn_local(async move {
            let paths = util::pick_paths().await;
            app.import_paths(paths);
        });
    }

    fn ingest(&self, response: ImportPathsResponse) {
        let existing: Vec<String> = self
            .items
            .with_untracked(|items| items.iter().map(|i| i.source_path.to_lowercase()).collect());
        let template = self.current_config();
        let mut added_items = Vec::new();
        let mut added_configs = Vec::new();

        for item in response.items {
            if existing.contains(&item.source_path.to_lowercase()) {
                continue;
            }
            let config = template.clone().unwrap_or_else(defaults::default_item_config);
            added_configs.push((item.id.clone(), config));
            added_items.push(item);
        }

        if !added_items.is_empty() {
            let first_id = added_items[0].id.clone();
            self.items.update(|items| items.extend(added_items));
            self.configs.update(|list| list.extend(added_configs));
            if self.selected.get_untracked().is_none() {
                self.selected.set(Some(first_id));
            }
        }

        if !response.rejected_paths.is_empty() {
            let detail = response
                .rejected_paths
                .iter()
                .take(3)
                .map(|rejected| format!("{}：{}", util::file_name(&rejected.path), rejected.message))
                .collect::<Vec<_>>()
                .join("\n");
            toast_with(
                format!("{} 个文件无法导入\n{detail}", response.rejected_paths.len()),
                Tone::Warning,
                None,
            );
        }
        if !response.warnings.is_empty() {
            toast(response.warnings.join("\n"));
        }
    }

    pub fn remove_item(&self, id: String) {
        let removed_index = self
            .items
            .with_untracked(|items| items.iter().position(|item| item.id == id));
        let Some(index) = removed_index else {
            return;
        };
        self.items.update(|items| items.retain(|item| item.id != id));
        self.configs.update(|list| list.retain(|(key, _)| *key != id));
        if self.selected.get_untracked().as_deref() == Some(id.as_str()) {
            let next = self.items.with_untracked(|items| {
                items
                    .get(index)
                    .or_else(|| items.last())
                    .map(|item| item.id.clone())
            });
            self.selected.set(next);
        }
    }

    pub fn clear_items(&self) {
        self.items.set(Vec::new());
        self.configs.set(Vec::new());
        self.selected.set(None);
        self.preview.set(PreviewState::default());
    }

    pub fn select_item(&self, id: String) {
        if self.selected.get_untracked().as_deref() == Some(id.as_str()) {
            return;
        }
        self.selected.set(Some(id));
        self.wand_selection.set(None);
    }

    // ---------------------------------------------------------------- 配置

    pub fn update_config(&self, update: impl FnOnce(&mut ItemConfig)) {
        let Some(id) = self.selected.get_untracked() else {
            return;
        };
        self.configs.update(|list| {
            if let Some((_, config)) = list.iter_mut().find(|(key, _)| *key == id) {
                update(config);
            }
        });
    }

    pub fn update_render(&self, update: impl FnOnce(&mut RenderConfig)) {
        self.update_config(|config| {
            update(&mut config.render);
            config.active_preset = None;
        });
    }

    pub fn update_upscale(&self, update: impl FnOnce(&mut UpscaleConfig)) {
        self.update_config(|config| update(&mut config.upscale));
    }

    pub fn apply_preset(&self, preset_id: String) {
        let Some(preset) = self
            .all_presets()
            .into_iter()
            .find(|preset| preset.id == preset_id)
        else {
            return;
        };
        self.update_config(|config| {
            config.render = preset.config.clone();
            config.active_preset = Some(preset.id.clone());
        });
    }

    pub fn apply_config_to_all(&self) {
        let Some(current) = self.current_config() else {
            return;
        };
        let ids: Vec<String> = self.items.with_untracked(|items| {
            items.iter().map(|item| item.id.clone()).collect()
        });
        self.configs.update(|list| {
            for id in ids {
                if let Some((_, config)) = list.iter_mut().find(|(key, _)| *key == id) {
                    *config = current.clone();
                }
            }
        });
        toast("已把当前图标的所有参数应用到全部图标");
    }

    pub fn save_current_as_preset(&self) {
        let Some(current) = self.current_config() else {
            return;
        };
        let mut render = current.render.clone();
        // 原图处理（自动抠图）不随预设走：它描述的是某一张图，不是风格。
        render.auto_cutout.enabled = false;
        render.auto_cutout.tolerance = 20.0;
        render.auto_cutout.feather = 8.0;

        let existing = self.custom_presets.get_untracked();
        let next_number = existing
            .iter()
            .filter_map(|preset| {
                preset
                    .name
                    .strip_prefix("自定义预设 ")
                    .and_then(|rest| rest.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(0)
            + 1;
        let preset = Preset {
            id: format!("custom-{}", js_sys::Date::now() as u64),
            name: format!("自定义预设 {next_number}"),
            description: "保存的自定义设置".to_string(),
            config: render,
            custom: true,
        };
        let id = preset.id.clone();
        self.custom_presets.update(|list| list.push(preset));
        self.persist_presets();
        self.update_config(|config| config.active_preset = Some(id));
        toast("已保存为自定义预设");
    }

    pub fn rename_preset(&self, id: String, name: String) {
        let trimmed = name.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        self.custom_presets.update(|list| {
            if let Some(preset) = list.iter_mut().find(|preset| preset.id == id) {
                preset.name = trimmed;
            }
        });
        self.persist_presets();
    }

    pub fn delete_preset(&self, id: String) {
        self.custom_presets.update(|list| list.retain(|preset| preset.id != id));
        self.persist_presets();
        self.update_config(|config| {
            if config.active_preset.as_deref() == Some(id.as_str()) {
                config.active_preset = None;
            }
        });
    }

    fn persist_presets(&self) {
        storage::write_json(CUSTOM_PRESETS_KEY, &self.custom_presets.get_untracked());
    }

    // ---------------------------------------------------------------- 工具

    pub fn set_tool(&self, tool: Tool) {
        let current = self.tool.get_untracked();
        self.tool.set(if current == tool { Tool::None } else { tool });
        if self.tool.get_untracked() != Tool::Eyedropper {
            self.color_target.set(None);
        }
        if self.tool.get_untracked() != Tool::MagicWand {
            self.wand_selection.set(None);
        }
    }

    pub fn start_color_pick(&self, target: ColorTarget) {
        self.tool.set(Tool::Eyedropper);
        self.color_target.set(Some(target));
    }

    pub fn apply_picked_color(&self, rgb_hex: &str) {
        let Some(target) = self.color_target.get_untracked() else {
            return;
        };
        if target == ColorTarget::Brush {
            self.update_config(|config| {
                config.brush_color = util::replace_rgb(&config.brush_color, rgb_hex);
            });
        } else {
            self.update_render(|render| match target {
                ColorTarget::Brush => {}
                ColorTarget::Backplate => {
                    render.backplate_color = util::replace_rgb(&render.backplate_color, rgb_hex)
                }
                ColorTarget::GradientStart => {
                    render.gradient_start_color =
                        util::replace_rgb(&render.gradient_start_color, rgb_hex)
                }
                ColorTarget::GradientEnd => {
                    render.gradient_end_color =
                        util::replace_rgb(&render.gradient_end_color, rgb_hex)
                }
                ColorTarget::Shadow => {
                    render.outer_shadow.color =
                        util::replace_rgb(&render.outer_shadow.color, rgb_hex)
                }
                ColorTarget::Stroke => {
                    render.stroke.color = util::replace_rgb(&render.stroke.color, rgb_hex)
                }
                ColorTarget::GlossLight => {
                    render.gloss.light_color =
                        util::replace_rgb(&render.gloss.light_color, rgb_hex)
                }
            });
        }
        self.tool.set(Tool::None);
        self.color_target.set(None);
    }

    pub fn add_brush_stroke(&self, stroke: BrushStroke) {
        self.update_config(|config| {
            config.brush_strokes.push(stroke);
            config.active_preset = None;
        });
    }

    pub fn undo_brush(&self) {
        self.update_config(|config| {
            config.brush_strokes.pop();
        });
    }

    pub fn clear_brush(&self) {
        self.update_config(|config| config.brush_strokes.clear());
    }

    pub fn set_wand_selection(&self, selection: Option<WandSelection>) {
        self.wand_selection.set(selection);
    }

    pub fn delete_wand_selection(&self) {
        let Some(selection) = self.wand_selection.get_untracked() else {
            return;
        };
        self.update_config(|config| {
            config.wand_strokes.push(WandStroke {
                points: vec![WandPoint {
                    x: selection.x,
                    y: selection.y,
                }],
                tolerance: selection.tolerance,
            });
        });
        self.wand_selection.set(None);
    }

    pub fn undo_wand(&self) {
        self.update_config(|config| {
            config.wand_strokes.pop();
        });
    }

    pub fn clear_wand(&self) {
        self.update_config(|config| config.wand_strokes.clear());
    }

    pub fn add_eraser_stroke(&self, stroke: EraserStroke) {
        self.update_config(|config| config.eraser_strokes.push(stroke));
    }

    pub fn undo_eraser(&self) {
        self.update_config(|config| {
            config.eraser_strokes.pop();
        });
    }

    pub fn clear_eraser(&self) {
        self.update_config(|config| config.eraser_strokes.clear());
    }

    // ---------------------------------------------------------------- 导出

    pub fn export_selected_ico(&self) {
        let Some(item) = self.current_item() else {
            return;
        };
        let Some(config) = self.current_config() else {
            return;
        };
        self.run_ico(vec![config.export_item(item.source_path)]);
    }

    pub fn export_all_ico(&self) {
        let items = self.export_items();
        if items.is_empty() {
            return;
        }
        self.run_ico(items);
    }

    pub fn export_selected_png(&self) {
        let Some(item) = self.current_item() else {
            return;
        };
        let Some(config) = self.current_config() else {
            return;
        };
        self.run_png(vec![config.export_item(item.source_path)], false);
    }

    pub fn export_all_png(&self) {
        let items = self.export_items();
        if items.is_empty() {
            return;
        }
        self.run_png(items, false);
    }

    pub fn extract_ico_png(&self) {
        let items: Vec<ExportIcoItemRequest> = self
            .items
            .with_untracked(|items| items.clone())
            .into_iter()
            .filter(|item| item.source_path.to_lowercase().ends_with(".ico"))
            .filter_map(|item| {
                self.config_for(&item.id)
                    .map(|config| config.export_item(item.source_path))
            })
            .collect();
        if items.is_empty() {
            return;
        }
        self.run_png(items, true);
    }

    pub fn apply_to_shortcuts(&self) {
        let items: Vec<ApplyToLnkItemRequest> = self
            .items
            .with_untracked(|items| items.clone())
            .into_iter()
            .filter(|item| item.file_type == InputFileType::Lnk)
            .filter_map(|item| {
                self.config_for(&item.id).map(|config| ApplyToLnkItemRequest {
                    lnk_path: item.source_path,
                    render_config: config.render,
                    upscale_config: config.upscale,
                    brush_strokes: config.brush_strokes,
                    wand_strokes: config.wand_strokes,
                    eraser_strokes: config.eraser_strokes,
                })
            })
            .collect();
        if items.is_empty() {
            return;
        }
        self.exporting.set(true);
        self.error.set(None);
        self.export_result.set(None);
        let app = *self;
        spawn_local(async move {
            match api::apply_to_lnk(items).await {
                Ok(result) => app.export_result.set(Some(ExportResult::Lnk(result))),
                Err(err) => app.error.set(Some(err)),
            }
            app.exporting.set(false);
        });
    }

    fn config_for(&self, id: &str) -> Option<ItemConfig> {
        self.configs
            .with_untracked(|list| list.iter().find(|(key, _)| key == id).map(|(_, cfg)| cfg.clone()))
    }

    fn export_items(&self) -> Vec<ExportIcoItemRequest> {
        let items = self.items.with_untracked(|items| items.clone());
        items
            .into_iter()
            .filter_map(|item| {
                self.config_for(&item.id)
                    .map(|config| config.export_item(item.source_path))
            })
            .collect()
    }

    fn run_ico(&self, items: Vec<ExportIcoItemRequest>) {
        self.exporting.set(true);
        self.error.set(None);
        self.export_result.set(None);
        let app = *self;
        spawn_local(async move {
            match api::export_ico(items).await {
                Ok(result) => {
                    if !result.cancelled {
                        app.export_result.set(Some(ExportResult::Ico(result)));
                    }
                }
                Err(err) => app.error.set(Some(err)),
            }
            app.exporting.set(false);
        });
    }

    fn run_png(&self, items: Vec<ExportIcoItemRequest>, raw_source: bool) {
        self.exporting.set(true);
        self.error.set(None);
        self.export_result.set(None);
        let app = *self;
        spawn_local(async move {
            match api::export_png(items, raw_source).await {
                Ok(result) => {
                    if !result.cancelled {
                        app.export_result.set(Some(ExportResult::Ico(result)));
                    }
                }
                Err(err) => app.error.set(Some(err)),
            }
            app.exporting.set(false);
        });
    }

    pub fn clear_error(&self) {
        self.error.set(None);
    }
}
