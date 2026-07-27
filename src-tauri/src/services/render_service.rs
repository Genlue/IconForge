use image::ImageEncoder;
use tauri::AppHandle;

use crate::domain::request::RenderPreviewRequest;
use crate::domain::response::RenderPreviewResponse;
use crate::error::app_error::AppError;
use crate::renderer::{render_master, validate_config};
use crate::state::AppState;

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

    let master = render_master(&source, &validated)?;

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
