use std::path::{Path, PathBuf};

use tempfile::NamedTempFile;

use crate::error::app_error::AppError;

pub fn write_bytes_atomic(path: &Path, bytes: &[u8], overwrite: bool) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("path has no parent directory".into()))?;

    let tmp = NamedTempFile::new_in(parent).map_err(|e| {
        AppError::io_error(
            format!("failed to create temp file: {}", e),
            Some(parent.to_path_buf()),
        )
    })?;

    tmp.as_file()
        .write_all(bytes)
        .and_then(|_| tmp.as_file().flush())
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|e| {
            AppError::io_error(
                format!("failed to write temp file: {}", e),
                Some(path.to_path_buf()),
            )
        })?;

    if overwrite {
        tmp.persist(path).map_err(|e| {
            AppError::io_error(
                format!("failed to persist file: {}", e),
                Some(path.to_path_buf()),
            )
        })?;
    } else {
        tmp.persist_noclobber(path).map_err(|_| {
            AppError::io_error(
                format!("file already exists: {}", path.display()),
                Some(path.to_path_buf()),
            )
        })?;
    }

    Ok(())
}

pub fn next_available_path(
    directory: &Path,
    stem: &str,
    extension: &str,
) -> Result<PathBuf, AppError> {
    let mut path = directory.join(format!("{}.{}", stem, extension));
    if !path.exists() {
        return Ok(path);
    }
    for i in 2..10000 {
        path = directory.join(format!("{} ({}).{}", stem, i, extension));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(AppError::Internal(
        "could not find available path after 10000 attempts".into(),
    ))
}

// We need WriteAll trait
use std::io::Write;
