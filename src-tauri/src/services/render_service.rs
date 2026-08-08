use image::ImageEncoder;
use tauri::AppHandle;

use crate::domain::request::{RenderPreviewRequest, SourceImageRequest, WandSelectionRequest};
use crate::domain::response::{RenderPreviewResponse, SourceImageResponse, WandSelectionResponse};
use crate::error::app_error::AppError;
use crate::renderer::{composite_brush_strokes, render_master_with_source_edits, validate_config};
use crate::state::AppState;

/// Returns the current source image (after upscale) with all source-editing
/// strokes applied — the reference view for the original-image editor.
pub fn render_source_image(
    app: &AppHandle,
    request: SourceImageRequest,
    state: &AppState,
) -> Result<SourceImageResponse, AppError> {
    let loaded = super::source_service::load_processed_source(
        app,
        std::path::Path::new(&request.source_path),
        &request.upscale_config,
        state,
    )?;
    let mut source = (*loaded).clone();
    crate::renderer::eraser::apply_eraser_strokes(&mut source, &request.eraser_strokes);
    crate::renderer::wand::apply_erasures(&mut source, &request.wand_strokes);

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            source.as_raw(),
            source.width(),
            source.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| AppError::Internal(format!("source PNG encode failed: {}", e)))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
    Ok(SourceImageResponse {
        png_base64: b64,
        width: source.width(),
        height: source.height(),
    })
}

pub fn compute_wand_selection(
    app: &AppHandle,
    request: WandSelectionRequest,
    state: &AppState,
) -> Result<WandSelectionResponse, AppError> {
    let loaded = super::source_service::load_processed_source(
        app,
        std::path::Path::new(&request.source_path),
        &request.upscale_config,
        state,
    )?;
    let mut source = (*loaded).clone();
    // Selection is computed on the current state of the original image, i.e.
    // after previously applied eraser strokes and wand deletions.
    crate::renderer::eraser::apply_eraser_strokes(&mut source, &request.eraser_strokes);
    crate::renderer::wand::apply_erasures(&mut source, &request.wand_strokes);

    let mask = crate::renderer::wand::selection_mask_256(
        &source,
        request.point.x,
        request.point.y,
        request.tolerance,
    )
    .ok_or_else(|| AppError::InvalidArgument("selection point outside the image".into()))?;

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(&mask, 256, 256, image::ExtendedColorType::L8)
        .map_err(|e| AppError::Internal(format!("selection mask encode failed: {}", e)))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);
    Ok(WandSelectionResponse {
        mask_png_base64: b64,
    })
}

pub fn render_preview(
    app: &AppHandle,
    request: RenderPreviewRequest,
    state: &AppState,
) -> Result<RenderPreviewResponse, AppError> {
    // Validate preview size
    if request.preview_size < 64 || request.preview_size > 1024 {
        return Err(AppError::InvalidArgument(
            "previewSize must be 64..=1024".into(),
        ));
    }

    let validated = validate_config(&request.render_config)?;

    let source = super::source_service::load_processed_source(
        app,
        std::path::Path::new(&request.source_path),
        &request.upscale_config,
        state,
    )?;

    let master = composite_brush_strokes(
        render_master_with_source_edits(
            &source,
            &validated,
            &request.wand_strokes,
            &request.eraser_strokes,
        )?,
        &request.brush_strokes,
    )?;

    // Resize to preview size
    let preview = if request.preview_size == 256 {
        master
    } else {
        crate::renderer::resize::resize_from_master(&master, request.preview_size)?
    };

    // Encode as PNG base64
    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            preview.as_raw(),
            preview.width(),
            preview.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| AppError::Internal(format!("preview PNG encode failed: {}", e)))?;

    let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &png_bytes);

    Ok(RenderPreviewResponse {
        png_base64: b64,
        width: preview.width(),
        height: preview.height(),
        processed_source_width: source.width(),
        processed_source_height: source.height(),
    })
}
