pub mod commands;
pub mod domain;
pub mod error;
pub mod extractor;
pub mod output;
pub mod renderer;
pub mod services;
pub mod state;
pub mod windows;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .manage(state::AppState::new().expect("failed to initialize app state"))
        .invoke_handler(tauri::generate_handler![
            commands::import_paths,
            commands::render_preview,
            commands::compute_wand_selection,
            commands::render_source_image,
            commands::export_ico,
            commands::export_png,
            commands::apply_to_lnk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running IconForge");
}

#[cfg(target_os = "windows")]
pub use windows::*;
