use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;

use windows::Win32::System::Environment::ExpandEnvironmentStringsW;

use crate::error::app_error::AppError;

pub fn to_wide_null(value: &OsStr) -> Vec<u16> {
    value.encode_wide().chain(std::iter::once(0)).collect()
}

pub fn from_wide_null_terminated(value: &[u16]) -> OsString {
    let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
    let slice = &value[..end];
    OsString::from_wide(slice)
}

pub fn expand_environment_path(value: &OsStr) -> Result<PathBuf, AppError> {
    let wide = to_wide_null(value);
    let mut result = vec![0u16; 32768];

    // SAFETY: ExpandEnvironmentStringsW with properly sized buffers
    let len = unsafe {
        ExpandEnvironmentStringsW(
            windows::core::PCWSTR::from_raw(wide.as_ptr()),
            Some(&mut result),
        )
    };

    if len == 0 {
        return Ok(PathBuf::from(value));
    }

    let result_str = from_wide_null_terminated(&result);
    Ok(PathBuf::from(result_str.to_string_lossy().as_ref()))
}
