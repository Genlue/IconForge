pub mod com;
pub mod resource;
pub mod shell_link;
pub mod shell_notify;
pub mod wide_string;

#[cfg(target_os = "windows")]
pub use self::{
    com::ComStaWorker,
    shell_link::{read_shortcut, rewrite_shortcut_icon_atomic, ShortcutInfo},
    shell_notify::{notify_associations_changed, notify_icon_changed},
    wide_string::{expand_environment_path, from_wide_null_terminated, to_wide_null},
};

#[cfg(not(target_os = "windows"))]
pub mod platform_stubs {
    use crate::error::app_error::AppError;
    use std::path::{Path, PathBuf};

    pub struct ComStaWorker;
    impl ComStaWorker {
        pub fn spawn() -> Result<Self, AppError> {
            Err(AppError::PlatformUnsupported)
        }
        pub fn execute<T, F>(&self, _task: F) -> Result<T, AppError>
        where
            T: Send + 'static,
            F: FnOnce() -> Result<T, AppError> + Send + 'static,
        {
            Err(AppError::PlatformUnsupported)
        }
        pub fn shutdown(&self) -> Result<(), AppError> {
            Err(AppError::PlatformUnsupported)
        }
    }

    pub struct ShortcutInfo {
        pub target_path: Option<PathBuf>,
        pub icon_location: Option<PathBuf>,
        pub icon_index: i32,
    }

    pub fn read_shortcut(_path: &Path) -> Result<ShortcutInfo, AppError> {
        Err(AppError::PlatformUnsupported)
    }
    pub fn rewrite_shortcut_icon_atomic(
        _lnk_path: &Path,
        _icon_path: &Path,
        _icon_index: i32,
    ) -> Result<Option<PathBuf>, AppError> {
        Err(AppError::PlatformUnsupported)
    }
    pub fn notify_icon_changed(_lnk_path: &Path, _icon_path: &Path) {}
    pub fn notify_associations_changed() {}
    pub fn to_wide_null(value: &std::ffi::OsStr) -> Vec<u16> {
        value.encode_wide().chain(std::iter::once(0)).collect()
    }
    pub fn from_wide_null_terminated(value: &[u16]) -> std::ffi::OsString {
        let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
        String::from_utf16_lossy(&value[..end]).into()
    }
    pub fn expand_environment_path(
        _value: &std::ffi::OsStr,
    ) -> Result<std::path::PathBuf, AppError> {
        Err(AppError::PlatformUnsupported)
    }
}
