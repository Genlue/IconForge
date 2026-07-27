use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager};

use crate::domain::config::RenderConfig;
use crate::error::app_error::AppError;

pub fn persist_for_shortcut(
    app: &AppHandle,
    lnk_path: &Path,
    _source: &image::RgbaImage,
    config: &RenderConfig,
    ico_bytes: &[u8],
) -> Result<PathBuf, AppError> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| AppError::io_error(format!("failed to get app data dir: {}", e), None))?;

    let managed_dir = app_dir.join("managed-icons");
    std::fs::create_dir_all(&managed_dir).map_err(|e| {
        AppError::io_error(
            format!("failed to create managed-icons dir: {}", e),
            Some(managed_dir.clone()),
        )
    })?;

    // Compute hash
    let config_json = serde_json::to_string(config)
        .map_err(|e| AppError::Internal(format!("config serialization failed: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(lnk_path.to_string_lossy().as_bytes());
    hasher.update(&ico_bytes);
    hasher.update(config_json.as_bytes());
    let hash = hex::encode(hasher.finalize());

    let icon_path = managed_dir.join(format!("{}.ico", hash));

    if icon_path.exists() {
        // Verify existing file
        let verify = std::fs::read(&icon_path).and_then(|data| {
            let mut cursor = std::io::Cursor::new(&data[..]);
            ico::IconDir::read(&mut cursor).map(|_| ())
        });
        if verify.is_ok() {
            return Ok(icon_path);
        }
    }

    // Atomic write
    super::atomic_file::write_bytes_atomic(&icon_path, ico_bytes, true)?;

    Ok(icon_path)
}
