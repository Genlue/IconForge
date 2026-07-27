use std::path::Path;

use image::ImageEncoder;
use uuid::Uuid;

use crate::domain::input::{ExportMode, InputFileType, InputItem};
use crate::domain::request::ImportPathsRequest;
use crate::domain::response::{ImportPathsResponse, RejectedPath};
use crate::error::app_error::AppError;
use crate::extractor::{ExtractedIcon, IconExtractor};
use crate::state::AppState;

pub fn import_paths(
    request: ImportPathsRequest,
    state: &AppState,
) -> Result<ImportPathsResponse, AppError> {
    let extractor = IconExtractor::new(&state.com_sta_worker);
    let mut items = Vec::new();
    let mut rejected_paths = Vec::new();
    let warnings = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    for raw_path in &request.paths {
        if raw_path.is_empty() {
            continue;
        }
        let path = Path::new(raw_path);

        match process_single_path(path, &extractor, &mut seen_paths) {
            Ok(mut new_items) => {
                for item in &mut new_items {
                    seen_paths.insert(item.source_path.to_lowercase());
                }
                // Separate directory children
                for item in new_items.drain(..) {
                    if item.file_type == InputFileType::Directory {
                        // Expand directory
                        match expand_directory_item(item, &extractor, &mut seen_paths) {
                            Ok(expanded) => {
                                items.push(expanded.0);
                                items.extend(expanded.1);
                            }
                            Err(e) => {
                                rejected_paths.push(RejectedPath {
                                    path: raw_path.clone(),
                                    code: e.code().to_string(),
                                    message: e.to_string(),
                                });
                            }
                        }
                    } else {
                        items.push(item);
                    }
                }
            }
            Err(e) => {
                rejected_paths.push(RejectedPath {
                    path: raw_path.clone(),
                    code: e.code().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    Ok(ImportPathsResponse {
        items,
        rejected_paths,
        warnings,
    })
}

fn process_single_path(
    path: &Path,
    extractor: &IconExtractor,
    seen: &mut std::collections::HashSet<String>,
) -> Result<Vec<InputItem>, AppError> {
    let normalized =
        std::fs::canonicalize(path).map_err(|_| AppError::PathNotFound(path.to_path_buf()))?;

    let key = normalized.to_string_lossy().to_lowercase();
    if seen.contains(&key) {
        return Ok(Vec::new());
    }

    let file_type = IconExtractor::classify(path)?;

    match file_type {
        InputFileType::Directory => {
            let item = create_directory_item(&normalized, &extractor)?;
            Ok(vec![item])
        }
        _ => {
            let extracted = extractor.extract(path)?;
            let item = create_item_from_extracted(&normalized, file_type, extracted)?;
            Ok(vec![item])
        }
    }
}

fn expand_directory_item(
    dir_item: InputItem,
    _extractor: &IconExtractor,
    seen: &mut std::collections::HashSet<String>,
) -> Result<(InputItem, Vec<InputItem>), AppError> {
    use crate::extractor::directory::{expand_directory, DirectoryLimits};

    let path = Path::new(&dir_item.source_path);
    let expansion = expand_directory(path, DirectoryLimits::default())?;

    let mut children = Vec::new();
    for child_result in expansion.children {
        match child_result {
            Ok(extracted) => {
                let child_path_str = extracted.source_path.to_string_lossy().to_lowercase();
                if !seen.contains(&child_path_str) {
                    match create_item_from_extracted(
                        &extracted.source_path,
                        extracted.file_type,
                        extracted.icon,
                    ) {
                        Ok(item) => {
                            seen.insert(child_path_str);
                            children.push(item);
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {}
        }
    }

    Ok((dir_item, children))
}

fn create_directory_item(path: &Path, _extractor: &IconExtractor) -> Result<InputItem, AppError> {
    let extracted = crate::extractor::directory::extract_directory_icon(path)?;
    create_item_from_extracted(path, InputFileType::Directory, extracted)
}

fn create_item_from_extracted(
    path: &Path,
    file_type: InputFileType,
    extracted: ExtractedIcon,
) -> Result<InputItem, AppError> {
    let display_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Generate thumbnail (128px max)
    let thumbnail = generate_thumbnail(&extracted.pixels)?;

    let id = Uuid::new_v4().to_string();
    let resolved_target_path = extracted
        .resolved_target_path
        .map(|p| p.to_string_lossy().to_string());
    let supported = match file_type {
        InputFileType::Lnk => vec![ExportMode::ExportAsIco, ExportMode::ApplyToLnk],
        _ => vec![ExportMode::ExportAsIco],
    };

    Ok(InputItem {
        id,
        source_path: path.to_string_lossy().to_string(),
        display_name,
        file_type,
        parent_directory_path: path.parent().map(|p| p.to_string_lossy().to_string()),
        resolved_target_path,
        source_width: extracted.pixels.width(),
        source_height: extracted.pixels.height(),
        thumbnail_png_base64: thumbnail,
        supported_export_modes: supported,
        warnings: extracted.warnings,
    })
}

fn generate_thumbnail(image: &image::RgbaImage) -> Result<String, AppError> {
    let max_size = 128u32;
    let (w, h) = if image.width() > max_size || image.height() > max_size {
        let scale = max_size as f32 / image.width().max(image.height()) as f32;
        (
            (image.width() as f32 * scale) as u32,
            (image.height() as f32 * scale) as u32,
        )
    } else {
        (image.width(), image.height())
    };

    let thumbnail = image::imageops::resize(image, w, h, image::imageops::FilterType::Lanczos3);

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            thumbnail.as_raw(),
            thumbnail.width(),
            thumbnail.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| AppError::Internal(format!("thumbnail PNG encode failed: {}", e)))?;

    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &png_bytes,
    ))
}
