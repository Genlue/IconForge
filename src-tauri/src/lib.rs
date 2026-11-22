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
            commands::pick_import_files,
            commands::import_paths,
            commands::render_preview,
            commands::compute_wand_selection,
            commands::render_source_image,
            commands::export_ico,
            commands::export_png,
            commands::apply_to_lnk,
        ])
        // GlassDesign 的宿主胶水：窗口壳层与壁纸接线。
        // 当前是安全空操作，保留它是为了让「上游升级只改 tag」这条约定成立。
        .setup(|app| {
            if let Err(err) = glass_design_host::install(app.handle()) {
                eprintln!("[shell] glass-design-host install failed: {err}");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running IconForge");
}

#[cfg(target_os = "windows")]
pub use windows::*;
