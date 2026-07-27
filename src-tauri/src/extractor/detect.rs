use std::path::Path;

use crate::domain::input::InputFileType;
use crate::error::app_error::AppError;

pub fn classify(path: &Path) -> Result<InputFileType, AppError> {
    let path_str = path.to_string_lossy();
    if path_str.contains('\0') {
        return Err(AppError::InvalidArgument("Path contains NUL byte".into()));
    }
    if !path.is_absolute() {
        return Err(AppError::InvalidArgument(format!(
            "Path must be absolute: {}",
            path_str
        )));
    }

    let metadata =
        std::fs::symlink_metadata(path).map_err(|_| AppError::PathNotFound(path.to_path_buf()))?;

    if metadata.is_dir() {
        return Ok(InputFileType::Directory);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "webp" | "bmp" | "gif" | "tif" | "tiff" | "ico" => {
            Ok(InputFileType::Image)
        }
        "lnk" => Ok(InputFileType::Lnk),
        _ => Ok(InputFileType::Exe), // All other files are Exe bucket
    }
}

pub fn normalize_existing_path(path: &Path) -> Result<std::path::PathBuf, AppError> {
    let canonical =
        std::fs::canonicalize(path).map_err(|_| AppError::PathNotFound(path.to_path_buf()))?;
    Ok(canonical)
}
