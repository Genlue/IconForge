use std::collections::HashSet;
use std::path::{Path, PathBuf};

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::domain::config::{BrushStroke, RenderConfig, UpscaleConfig};
use crate::domain::input::ExportMode;
use crate::domain::request::{
    ApplyToLnkRequest, ExportIcoItemRequest, ExportIcoRequest, ExportPngRequest,
};
use crate::domain::response::{
    AppliedShortcut, ApplyToLnkResponse, ExportIcoResponse, ExportedFile, RejectedPath,
};
use crate::error::app_error::AppError;
use crate::output::ico_writer::{self, encode_ico};
use crate::output::managed_icon;
use crate::renderer;
use crate::state::AppState;

pub fn prepare_ico(
    app: &AppHandle,
    source_path: &Path,
    config: &RenderConfig,
    upscale_config: &UpscaleConfig,
    brush_strokes: &[BrushStroke],
    wand_strokes: &[crate::domain::config::WandStroke],
    eraser_strokes: &[crate::domain::config::EraserStroke],
    state: &AppState,
) -> Result<Vec<u8>, AppError> {
    let validated = renderer::validate_config(config)?;
    let source =
        super::source_service::load_processed_source(app, source_path, upscale_config, state)?;
    let icon_set = renderer::render_icon_set_with_brushes(
        &source,
        &validated,
        brush_strokes,
        wand_strokes,
        eraser_strokes,
    )?;
    encode_ico(&icon_set)
}

pub fn export_ico_batch(
    app: &AppHandle,
    request: ExportIcoRequest,
    state: &AppState,
) -> Result<ExportIcoResponse, AppError> {
    if request.export_mode != ExportMode::ExportAsIco {
        return Err(AppError::InvalidArgument(
            "export_ico requires ExportAsIco mode".into(),
        ));
    }

    if request.items.is_empty() {
        return Err(AppError::InvalidArgument("items must not be empty".into()));
    }

    let mut seen = HashSet::new();
    let jobs: Vec<(PathBuf, ExportIcoItemRequest)> = request
        .items
        .into_iter()
        .filter_map(|item| {
            let path = std::fs::canonicalize(Path::new(&item.source_path)).ok()?;
            seen.insert(path.clone()).then_some((path, item))
        })
        .collect();

    if jobs.is_empty() {
        return Err(AppError::InvalidArgument("no valid source paths".into()));
    }

    if jobs.len() == 1 {
        let (path, item) = &jobs[0];
        let stem = sanitize_filename(path.file_stem().and_then(|s| s.to_str()).unwrap_or("icon"));
        let default_name = format!("{}.ico", stem);

        let dialog = app
            .dialog()
            .file()
            .add_filter("Windows Icon", &["ico"])
            .set_file_name(&default_name);

        match dialog.blocking_save_file() {
            Some(output_path) => {
                let requested_path = output_path.into_path().map_err(|e| {
                    AppError::IoFailed(format!("invalid path from dialog: {}", e), None)
                })?;
                let output_path = available_requested_path(&requested_path)?;
                let ico_bytes = prepare_ico(
                    app,
                    path,
                    &item.render_config,
                    &item.upscale_config,
                    &item.brush_strokes,
                &item.wand_strokes,
                &item.eraser_strokes,
                state,
                )?;
                ico_writer::write_ico_atomic(&output_path, &ico_bytes)?;
                Ok(ExportIcoResponse {
                    cancelled: false,
                    files: vec![ExportedFile {
                        source_path: path.to_string_lossy().to_string(),
                        output_path: output_path.to_string_lossy().to_string(),
                    }],
                    warnings: vec![],
                })
            }
            None => Ok(ExportIcoResponse {
                cancelled: true,
                files: vec![],
                warnings: vec![],
            }),
        }
    } else {
        let dir = app.dialog().file().blocking_pick_folder();
        let dir_path = match dir {
            Some(d) => d.into_path().map_err(|e| {
                AppError::IoFailed(format!("invalid path from dialog: {}", e), None)
            })?,
            None => {
                return Ok(ExportIcoResponse {
                    cancelled: true,
                    files: vec![],
                    warnings: vec![],
                })
            }
        };

        let mut files = Vec::new();
        let mut warnings = Vec::new();

        for (path, item) in &jobs {
            let stem =
                sanitize_filename(path.file_stem().and_then(|s| s.to_str()).unwrap_or("icon"));
            let output_path =
                crate::output::atomic_file::next_available_path(&dir_path, &stem, "ico")
                    .unwrap_or_else(|_| dir_path.join(format!("{}.ico", stem)));

            match prepare_ico(
                app,
                path,
                &item.render_config,
                &item.upscale_config,
                &item.brush_strokes,
                &item.wand_strokes,
                &item.eraser_strokes,
                state,
            ) {
                Ok(ico_bytes) => match ico_writer::write_ico_atomic(&output_path, &ico_bytes) {
                    Ok(()) => {
                        files.push(ExportedFile {
                            source_path: path.to_string_lossy().to_string(),
                            output_path: output_path.to_string_lossy().to_string(),
                        });
                    }
                    Err(e) => {
                        warnings.push(format!("failed to write {}: {}", output_path.display(), e));
                    }
                },
                Err(e) => {
                    warnings.push(format!("failed to process {}: {}", path.display(), e));
                }
            }
        }

        Ok(ExportIcoResponse {
            cancelled: false,
            files,
            warnings,
        })
    }
}

pub fn export_png_batch(
    app: &AppHandle,
    request: ExportPngRequest,
    state: &AppState,
) -> Result<ExportIcoResponse, AppError> {
    if request.items.is_empty() {
        return Err(AppError::InvalidArgument("items must not be empty".into()));
    }
    let mut jobs = Vec::new();
    let mut seen = HashSet::new();
    for item in request.items {
        if let Ok(path) = std::fs::canonicalize(Path::new(&item.source_path)) {
            if seen.insert(path.clone()) {
                jobs.push((path, item));
            }
        }
    }
    if jobs.is_empty() {
        return Err(AppError::InvalidArgument("no valid source paths".into()));
    }

    let output_dir = if jobs.len() == 1 {
        None
    } else {
        match app.dialog().file().blocking_pick_folder() {
            Some(path) => Some(
                path.into_path()
                    .map_err(|e| AppError::IoFailed(e.to_string(), None))?,
            ),
            None => {
                return Ok(ExportIcoResponse {
                    cancelled: true,
                    files: vec![],
                    warnings: vec![],
                })
            }
        }
    };
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    for (path, item) in jobs {
        let stem = sanitize_filename(path.file_stem().and_then(|s| s.to_str()).unwrap_or("icon"));
        let output_path = if let Some(dir) = &output_dir {
            crate::output::atomic_file::next_available_path(dir, &stem, "png")?
        } else {
            let Some(selected) = app
                .dialog()
                .file()
                .add_filter("PNG Image", &["png"])
                .set_file_name(format!("{}.png", stem))
                .blocking_save_file()
            else {
                return Ok(ExportIcoResponse {
                    cancelled: true,
                    files: vec![],
                    warnings: vec![],
                });
            };
            available_requested_path(
                &selected
                    .into_path()
                    .map_err(|e| AppError::IoFailed(e.to_string(), None))?,
            )?
        };

        let processed = (|| -> Result<image::RgbaImage, AppError> {
            if request.raw_source {
                let extractor = crate::extractor::IconExtractor::new(&state.com_sta_worker);
                return Ok(extractor.extract(&path)?.pixels);
            }
            let source = super::source_service::load_processed_source(
                app,
                &path,
                &item.upscale_config,
                state,
            )?;
            let validated = renderer::validate_config(&item.render_config)?;
            renderer::composite_brush_strokes(
                renderer::render_master_with_source_edits(
                    &source,
                    &validated,
                    &item.wand_strokes,
                    &item.eraser_strokes,
                )?,
                &item.brush_strokes,
            )
        })();
        match processed.and_then(|image| write_png_atomic(&output_path, &image)) {
            Ok(()) => files.push(ExportedFile {
                source_path: path.to_string_lossy().to_string(),
                output_path: output_path.to_string_lossy().to_string(),
            }),
            Err(error) => warnings.push(format!("failed to export {}: {}", path.display(), error)),
        }
    }
    Ok(ExportIcoResponse {
        cancelled: false,
        files,
        warnings,
    })
}

fn write_png_atomic(path: &Path, image: &image::RgbaImage) -> Result<(), AppError> {
    use image::ImageEncoder;
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| AppError::RenderFailed(format!("PNG encode failed: {}", e)))?;
    crate::output::atomic_file::write_bytes_atomic(path, &bytes, false)
}

fn available_requested_path(path: &Path) -> Result<PathBuf, AppError> {
    if !path.exists() {
        return Ok(path.to_path_buf());
    }
    let directory = path
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("path has no parent".into()))?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("icon");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("ico");
    crate::output::atomic_file::next_available_path(directory, stem, extension)
}

pub fn apply_to_shortcuts(
    app: &AppHandle,
    request: ApplyToLnkRequest,
    state: &AppState,
) -> Result<ApplyToLnkResponse, AppError> {
    if request.export_mode != ExportMode::ApplyToLnk {
        return Err(AppError::InvalidArgument(
            "apply_to_lnk requires ApplyToLnk mode".into(),
        ));
    }

    let mut applied = Vec::new();
    let mut failed = Vec::new();

    for item in &request.items {
        let lnk_path = Path::new(&item.lnk_path);

        let ext = lnk_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        if ext != "lnk" {
            failed.push(RejectedPath {
                path: item.lnk_path.clone(),
                code: "InvalidArgument".into(),
                message: "only .lnk files can be modified".into(),
            });
            continue;
        }

        let validated = match renderer::validate_config(&item.render_config) {
            Ok(config) => config,
            Err(e) => {
                failed.push(RejectedPath {
                    path: item.lnk_path.clone(),
                    code: e.code().to_string(),
                    message: e.to_string(),
                });
                continue;
            }
        };

        match process_single_shortcut(
            app,
            lnk_path,
            &item.render_config,
            &item.upscale_config,
            &item.brush_strokes,
            &item.wand_strokes,
            &item.eraser_strokes,
            &validated,
            state,
        ) {
            Ok(result) => applied.push(result),
            Err(e) => {
                failed.push(RejectedPath {
                    path: item.lnk_path.clone(),
                    code: e.code().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    crate::windows::shell_notify::notify_associations_changed();

    Ok(ApplyToLnkResponse { applied, failed })
}

fn process_single_shortcut(
    app: &AppHandle,
    lnk_path: &Path,
    raw_config: &RenderConfig,
    upscale_config: &UpscaleConfig,
    brush_strokes: &[crate::domain::config::BrushStroke],
    wand_strokes: &[crate::domain::config::WandStroke],
    eraser_strokes: &[crate::domain::config::EraserStroke],
    config: &renderer::ValidatedRenderConfig,
    state: &AppState,
) -> Result<AppliedShortcut, AppError> {
    let source =
        super::source_service::load_processed_source(app, lnk_path, upscale_config, state)?;
    let icon_set = renderer::render_icon_set_with_brushes(
        &source,
        config,
        brush_strokes,
        wand_strokes,
        eraser_strokes,
    )?;
    let ico_bytes = encode_ico(&icon_set)?;

    let managed_path =
        managed_icon::persist_for_shortcut(app, lnk_path, &source, raw_config, &ico_bytes)?;

    let backup =
        crate::windows::shell_link::rewrite_shortcut_icon_atomic(lnk_path, &managed_path, 0)?;

    Ok(AppliedShortcut {
        lnk_path: lnk_path.to_string_lossy().to_string(),
        managed_icon_path: managed_path.to_string_lossy().to_string(),
        backup_path: backup.map(|p| p.to_string_lossy().to_string()),
    })
}

fn sanitize_filename(name: &str) -> String {
    let invalid = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let sanitized: String = name
        .chars()
        .map(|c| {
            if invalid.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .collect();
    let trimmed = sanitized.trim_end_matches(|c| c == '.' || c == ' ');
    if trimmed.is_empty() {
        return "icon".into();
    }
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&trimmed.to_uppercase().as_str()) {
        format!("_{}", trimmed)
    } else {
        trimmed.to_string()
    }
}
