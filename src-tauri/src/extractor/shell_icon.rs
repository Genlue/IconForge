use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use image::RgbaImage;
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use super::hicon::hicon_to_rgba;
use crate::error::app_error::AppError;

pub fn extract_shell_icon(path: &Path, _preferred_size: u32) -> Result<RgbaImage, AppError> {
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // Try ExtractIconExW first
    let mut large: HICON = HICON::default();
    let mut small: HICON = HICON::default();

    let count = unsafe {
        ExtractIconExW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            0,
            Some(&mut large as *mut HICON),
            Some(&mut small as *mut HICON),
            1,
        )
    };

    if count > 0 {
        if !large.is_invalid() {
            let result = hicon_to_rgba(large);
            unsafe {
                let _ = DestroyIcon(large);
            }
            if !small.is_invalid() {
                unsafe {
                    let _ = DestroyIcon(small);
                }
            }
            if result.is_ok() {
                return result;
            }
        }
        if !small.is_invalid() {
            let result = hicon_to_rgba(small);
            unsafe {
                let _ = DestroyIcon(small);
            }
            if !large.is_invalid() {
                unsafe {
                    let _ = DestroyIcon(large);
                }
            }
            if result.is_ok() {
                return result;
            }
        }
    }

    // Clean up unused handles
    if !large.is_invalid() {
        unsafe {
            let _ = DestroyIcon(large);
        }
    }
    if !small.is_invalid() {
        unsafe {
            let _ = DestroyIcon(small);
        }
    }

    // Try SHGetFileInfoW as fallback
    let mut sfi = SHFILEINFOW::default();

    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            FILE_ATTRIBUTE_NORMAL,
            Some(&mut sfi as *mut SHFILEINFOW),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_SYSICONINDEX,
        )
    };

    if result != 0 {
        // Try to get icon via system image list using HICON from SHGetFileInfo
        let mut sfi_icon = SHFILEINFOW::default();
        let icon_result = unsafe {
            SHGetFileInfoW(
                PCWSTR::from_raw(wide_path.as_ptr()),
                FILE_ATTRIBUTE_NORMAL,
                Some(&mut sfi_icon as *mut SHFILEINFOW),
                std::mem::size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            )
        };
        if icon_result != 0 {
            let hicon = sfi_icon.hIcon;
            if !hicon.is_invalid() {
                let img_result = hicon_to_rgba(hicon);
                unsafe {
                    let _ = DestroyIcon(hicon);
                }
                if img_result.is_ok() {
                    return img_result;
                }
            }
        }
    }

    Err(AppError::IconExtractionFailed(
        "all shell icon extraction methods failed".into(),
        path.to_path_buf(),
    ))
}
