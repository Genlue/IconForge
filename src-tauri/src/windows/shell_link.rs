use std::path::{Path, PathBuf};
use std::ptr;

use windows::core::{Interface, PCWSTR};
use windows::Win32::System::Com::*;
use windows::Win32::UI::Shell::*;

use super::wide_string::{expand_environment_path, to_wide_null};
use crate::error::app_error::AppError;

pub struct ShortcutInfo {
    pub target_path: Option<PathBuf>,
    pub icon_location: Option<PathBuf>,
    pub icon_index: i32,
}

pub fn read_shortcut(path: &Path) -> Result<ShortcutInfo, AppError> {
    let wide_path = to_wide_null(path.as_os_str());

    // SAFETY: CoCreateInstance for ShellLink
    let shell_link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.map_err(|hr| {
            AppError::ShortcutReadFailed(
                format!("CoCreateInstance failed: {:?}", hr),
                path.to_path_buf(),
            )
        })?;

    // SAFETY: QueryInterface for IPersistFile
    let persist: IPersistFile = shell_link.cast().map_err(|hr| {
        AppError::ShortcutReadFailed(
            format!("IPersistFile cast failed: {:?}", hr),
            path.to_path_buf(),
        )
    })?;

    // SAFETY: IPersistFile::Load reads the shortcut file
    unsafe {
        persist
            .Load(PCWSTR::from_raw(wide_path.as_ptr()), STGM_READ)
            .map_err(|hr| {
                AppError::ShortcutReadFailed(
                    format!("IPersistFile::Load failed: {:?}", hr),
                    path.to_path_buf(),
                )
            })?;
    }

    // Get icon location
    let mut icon_buf = vec![0u16; 32768];
    let mut icon_index: i32 = 0;

    // SAFETY: IShellLinkW::GetIconLocation fills the buffer
    let has_icon_location = unsafe {
        shell_link
            .GetIconLocation(&mut icon_buf, &mut icon_index)
            .is_ok()
    };

    let icon_location = if has_icon_location {
        let icon_str = super::wide_string::from_wide_null_terminated(&icon_buf);
        if icon_str.len() > 0 {
            let expanded = expand_environment_path(icon_str.as_os_str()).ok();
            Some(expanded.unwrap_or_else(|| PathBuf::from(icon_str.to_string_lossy().as_ref())))
        } else {
            None
        }
    } else {
        None
    };

    // Get target path
    let mut target_buf = vec![0u16; 32768];

    // SAFETY: IShellLinkW::GetPath fills the buffer
    let has_target = unsafe {
        shell_link
            .GetPath(&mut target_buf, ptr::null_mut(), SLGP_RAWPATH.0 as u32)
            .is_ok()
    };

    let target_path = if has_target {
        let target_str = super::wide_string::from_wide_null_terminated(&target_buf);
        if target_str.len() > 0 {
            Some(PathBuf::from(target_str.to_string_lossy().as_ref()))
        } else {
            None
        }
    } else {
        None
    };

    Ok(ShortcutInfo {
        target_path,
        icon_location,
        icon_index,
    })
}

pub fn rewrite_shortcut_icon_atomic(
    lnk_path: &Path,
    icon_path: &Path,
    icon_index: i32,
) -> Result<Option<PathBuf>, AppError> {
    let parent = lnk_path
        .parent()
        .ok_or_else(|| AppError::InvalidArgument("LNK path has no parent".into()))?;

    // Generate temp LNK path
    let mut temp_path = None;
    for attempt in 0..10 {
        let uuid = uuid::Uuid::new_v4();
        let stem = lnk_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("shortcut");
        let candidate = parent.join(format!("{}.iconforge-{}.tmp.lnk", stem, uuid));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(_) => {
                std::fs::remove_file(&candidate).ok();
                temp_path = Some(candidate);
                break;
            }
            Err(_) if attempt < 9 => continue,
            Err(e) => {
                return Err(AppError::ShortcutWriteFailed(
                    format!("failed to create temp lnk file: {}", e),
                    lnk_path.to_path_buf(),
                ));
            }
        }
    }

    let temp_path = temp_path.ok_or_else(|| {
        AppError::ShortcutWriteFailed(
            "could not create unique temp lnk path after 10 attempts".into(),
            lnk_path.to_path_buf(),
        )
    })?;

    let wide_lnk = to_wide_null(lnk_path.as_os_str());
    let wide_temp = to_wide_null(temp_path.as_os_str());
    let wide_icon = to_wide_null(icon_path.as_os_str());

    // SAFETY: COM operations for rewriting the shortcut icon
    let result = unsafe {
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)
            .map_err(|hr| {
                AppError::ShortcutWriteFailed(
                    format!("CoCreateInstance failed: {:?}", hr),
                    lnk_path.to_path_buf(),
                )
            })?;

        let persist: IPersistFile = shell_link.cast().map_err(|hr| {
            AppError::ShortcutWriteFailed(
                format!("IPersistFile cast failed: {:?}", hr),
                lnk_path.to_path_buf(),
            )
        })?;

        // Load original
        persist
            .Load(PCWSTR::from_raw(wide_lnk.as_ptr()), STGM_READ)
            .map_err(|hr| {
                AppError::ShortcutWriteFailed(
                    format!("IPersistFile::Load failed: {:?}", hr),
                    lnk_path.to_path_buf(),
                )
            })?;

        // Set icon location
        shell_link
            .SetIconLocation(PCWSTR::from_raw(wide_icon.as_ptr()), icon_index)
            .map_err(|hr| {
                AppError::ShortcutWriteFailed(
                    format!("SetIconLocation failed: {:?}", hr),
                    lnk_path.to_path_buf(),
                )
            })?;

        // Save to temp
        persist
            .Save(PCWSTR::from_raw(wide_temp.as_ptr()), false)
            .map_err(|hr| {
                AppError::ShortcutWriteFailed(
                    format!("IPersistFile::Save failed: {:?}", hr),
                    lnk_path.to_path_buf(),
                )
            })?;

        persist
            .SaveCompleted(PCWSTR::from_raw(wide_temp.as_ptr()))
            .map_err(|hr| {
                AppError::ShortcutWriteFailed(
                    format!("IPersistFile::SaveCompleted failed: {:?}", hr),
                    lnk_path.to_path_buf(),
                )
            })?;

        // Verify temp LNK
        let mut verify_buf = [0u16; 32768];
        let mut verify_idx: i32 = 0;
        let _ = shell_link.GetIconLocation(&mut verify_buf, &mut verify_idx);

        // Create backup path
        let backup_path = create_backup_path(lnk_path)?;

        // Create backup copy
        std::fs::copy(lnk_path, &backup_path).map_err(|e| {
            AppError::ShortcutWriteFailed(
                format!("backup copy failed: {}", e),
                lnk_path.to_path_buf(),
            )
        })?;

        // Rename temp to target (atomic on same filesystem)
        std::fs::rename(&temp_path, lnk_path).map_err(|e| {
            AppError::ShortcutWriteFailed(format!("rename failed: {}", e), lnk_path.to_path_buf())
        })?;

        // Clean up temp (already moved)
        let _ = std::fs::remove_file(&temp_path);

        Ok::<_, AppError>(Some(backup_path))
    };

    // SHChangeNotify
    crate::windows::shell_notify::notify_icon_changed(lnk_path, icon_path);

    result
}

fn create_backup_path(lnk_path: &Path) -> Result<PathBuf, AppError> {
    let parent = lnk_path.parent().unwrap_or(Path::new("."));
    let stem = lnk_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("shortcut");
    let mut path = parent.join(format!("{}.iconforge.bak", stem));
    if !path.exists() {
        return Ok(path);
    }
    for i in 2.. {
        path = parent.join(format!("{}.iconforge.bak.{}", stem, i));
        if !path.exists() {
            return Ok(path);
        }
    }
    Err(AppError::ShortcutWriteFailed(
        "could not create backup path".into(),
        lnk_path.to_path_buf(),
    ))
}
