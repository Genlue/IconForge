use std::ptr;

use image::RgbaImage;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::error::app_error::AppError;

pub fn hicon_to_rgba(hicon: HICON) -> Result<RgbaImage, AppError> {
    if hicon.is_invalid() {
        return Err(AppError::Internal("invalid HICON handle".into()));
    }

    // GetIconInfo returns Result<()> via out parameter
    let mut info = ICONINFO::default();
    unsafe { GetIconInfo(hicon, &mut info as *mut ICONINFO) }
        .map_err(|_| AppError::Internal("GetIconInfo failed".into()))?;

    // Guard color and mask bitmaps
    let _color_guard = if info.hbmColor != HBITMAP::default() {
        Some(GdiGuard(HGDIOBJ(info.hbmColor.0)))
    } else {
        None
    };
    let _mask_guard = if info.hbmMask != HBITMAP::default() {
        Some(GdiGuard(HGDIOBJ(info.hbmMask.0)))
    } else {
        None
    };

    let has_color = info.hbmColor != HBITMAP::default();
    let (width, height) = {
        let mut bmp = BITMAP::default();
        let bmp_size = std::mem::size_of::<BITMAP>() as i32;
        if has_color {
            unsafe {
                GetObjectW(
                    HGDIOBJ(info.hbmColor.0),
                    bmp_size,
                    Some(&mut bmp as *mut _ as *mut _),
                );
            }
        } else {
            unsafe {
                GetObjectW(
                    HGDIOBJ(info.hbmMask.0),
                    bmp_size,
                    Some(&mut bmp as *mut _ as *mut _),
                );
            }
        }
        if has_color {
            (bmp.bmWidth as u32, bmp.bmHeight as u32)
        } else {
            (bmp.bmWidth as u32, bmp.bmHeight as u32 / 2)
        }
    };

    if width == 0 || height == 0 {
        return Err(AppError::Internal("icon has zero dimensions".into()));
    }

    // Create DC
    let dc = unsafe { CreateCompatibleDC(None) };
    if dc.is_invalid() {
        return Err(AppError::Internal("CreateCompatibleDC failed".into()));
    }
    let _dc_guard = DcGuard(dc);

    // Create DIB section
    let mut bmi: BITMAPINFO = unsafe { std::mem::zeroed() };
    bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = width as i32;
    bmi.bmiHeader.biHeight = -(height as i32);
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB.0;

    let mut dib_bits: *mut std::ffi::c_void = ptr::null_mut();
    let dib = unsafe { CreateDIBSection(Some(dc), &bmi, DIB_RGB_COLORS, &mut dib_bits, None, 0) }
        .map_err(|_| AppError::Internal("CreateDIBSection failed".into()))?;

    let _dib_guard = GdiGuard(HGDIOBJ(dib.0));

    // Select DIB into DC
    let old_obj = unsafe { SelectObject(dc, HGDIOBJ(dib.0)) };
    let _old_guard = GdiGuard(old_obj);

    unsafe {
        ptr::write_bytes(dib_bits, 0, (width * height * 4) as usize);
    }

    // Draw icon
    unsafe {
        let _ = DrawIconEx(
            dc,
            0,
            0,
            hicon,
            width as i32,
            height as i32,
            0,
            None,
            DI_NORMAL,
        );
    }

    // Copy pixel data (BGRA -> RGBA)
    let dib_slice =
        unsafe { std::slice::from_raw_parts(dib_bits as *const u8, (width * height * 4) as usize) };

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for chunk in dib_slice.chunks(4) {
        let b = chunk[0];
        let g = chunk[1];
        let r = chunk[2];
        let a = chunk[3];
        if a > 0 {
            let r_u = ((r as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            let g_u = ((g as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            let b_u = ((b as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8;
            rgba.extend_from_slice(&[r_u, g_u, b_u, a]);
        } else {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        }
    }

    RgbaImage::from_raw(width, height, rgba)
        .ok_or_else(|| AppError::Internal("failed to create RgbaImage".into()))
}

struct GdiGuard(HGDIOBJ);
impl Drop for GdiGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteObject(self.0);
            }
        }
    }
}

struct DcGuard(HDC);
impl Drop for DcGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = DeleteDC(self.0);
            }
        }
    }
}
