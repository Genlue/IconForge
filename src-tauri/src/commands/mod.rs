use tauri::{AppHandle, State};

use crate::domain::request::{
    ApplyToLnkRequest, ExportIcoRequest, ExportPngRequest, ImportPathsRequest, RenderPreviewRequest,
    SourceImageRequest, WandSelectionRequest,
};
use crate::domain::response::{
    ApplyToLnkResponse, CommandError, ExportIcoResponse, ImportPathsResponse, RenderPreviewResponse,
    SourceImageResponse, WandSelectionResponse,
};
use crate::services::{export_service, import_service, render_service};
use crate::state::AppState;

fn into_cmd_err(e: crate::error::app_error::AppError) -> CommandError {
    e.into_command_error()
}

#[tauri::command]
pub async fn import_paths(
    request: ImportPathsRequest,
    state: State<'_, AppState>,
) -> Result<ImportPathsResponse, CommandError> {
    import_service::import_paths(request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn render_preview(
    app: AppHandle,
    request: RenderPreviewRequest,
    state: State<'_, AppState>,
) -> Result<RenderPreviewResponse, CommandError> {
    render_service::render_preview(&app, request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn compute_wand_selection(
    app: AppHandle,
    request: WandSelectionRequest,
    state: State<'_, AppState>,
) -> Result<WandSelectionResponse, CommandError> {
    render_service::compute_wand_selection(&app, request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn render_source_image(
    app: AppHandle,
    request: SourceImageRequest,
    state: State<'_, AppState>,
) -> Result<SourceImageResponse, CommandError> {
    render_service::render_source_image(&app, request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn export_ico(
    app: AppHandle,
    request: ExportIcoRequest,
    state: State<'_, AppState>,
) -> Result<ExportIcoResponse, CommandError> {
    export_service::export_ico_batch(&app, request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn export_png(
    app: AppHandle,
    request: ExportPngRequest,
    state: State<'_, AppState>,
) -> Result<ExportIcoResponse, CommandError> {
    export_service::export_png_batch(&app, request, &state).map_err(into_cmd_err)
}

#[tauri::command]
pub async fn apply_to_lnk(
    app: AppHandle,
    request: ApplyToLnkRequest,
    state: State<'_, AppState>,
) -> Result<ApplyToLnkResponse, CommandError> {
    export_service::apply_to_shortcuts(&app, request, &state).map_err(into_cmd_err)
}
