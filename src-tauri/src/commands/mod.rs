use tauri::{AppHandle, State};

use crate::domain::request::{
    ApplyToLnkRequest, ExportIcoRequest, ImportPathsRequest, RenderPreviewRequest,
};
use crate::domain::response::{
    ApplyToLnkResponse, CommandError, ExportIcoResponse, ImportPathsResponse, RenderPreviewResponse,
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
    request: RenderPreviewRequest,
    state: State<'_, AppState>,
) -> Result<RenderPreviewResponse, CommandError> {
    render_service::render_preview(request, &state).map_err(into_cmd_err)
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
pub async fn apply_to_lnk(
    app: AppHandle,
    request: ApplyToLnkRequest,
    state: State<'_, AppState>,
) -> Result<ApplyToLnkResponse, CommandError> {
    export_service::apply_to_shortcuts(&app, request, &state).map_err(into_cmd_err)
}
