use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use image::RgbaImage;
use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::error::app_error::AppError;

pub enum IconSelector {
    Largest,
    Index(u32),
    ResourceId(u16),
}

pub fn extract_pe_icon(path: &Path, selector: IconSelector) -> Result<RgbaImage, AppError> {
    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: LoadLibraryExW with LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE
    // does not execute any code from the module
    let module = unsafe {
        LoadLibraryExW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            None,
            LOAD_LIBRARY_AS_DATAFILE | LOAD_LIBRARY_AS_IMAGE_RESOURCE,
        )
    }
    .map_err(|_| {
        AppError::IconExtractionFailed(
            "LoadLibraryExW failed - not a valid PE".into(),
            path.to_path_buf(),
        )
    })?;

    if module.is_invalid() {
        return Err(AppError::IconExtractionFailed(
            "LoadLibraryExW returned invalid module".into(),
            path.to_path_buf(),
        ));
    }

    let _module_guard = ModuleGuard(module);

    // Enumerate RT_GROUP_ICON resources
    let group_icons = enumerate_group_icons(module)?;
    if group_icons.is_empty() {
        return Err(AppError::IconExtractionFailed(
            "no RT_GROUP_ICON resources found".into(),
            path.to_path_buf(),
        ));
    }

    let group_name = select_group_icon(module, &group_icons, selector).ok_or_else(|| {
        AppError::IconExtractionFailed(
            "requested icon group was not found".into(),
            path.to_path_buf(),
        )
    })?;

    // Load GRPICONDIR
    let group_data = load_resource_bytes(module, RT_GROUP_ICON, group_name)?;
    if group_data.len() < 6 {
        return Err(AppError::IconExtractionFailed(
            "GRPICONDIR too short".into(),
            path.to_path_buf(),
        ));
    }

    let id_reserved = u16::from_le_bytes([group_data[0], group_data[1]]);
    let id_type = u16::from_le_bytes([group_data[2], group_data[3]]);
    let id_count = u16::from_le_bytes([group_data[4], group_data[5]]);

    if id_reserved != 0 || id_type != 1 || id_count == 0 {
        return Err(AppError::IconExtractionFailed(
            "invalid GRPICONDIR header".into(),
            path.to_path_buf(),
        ));
    }

    // Parse entries and select the largest one
    struct DirEntry {
        width: u8,
        height: u8,
        bit_count: u16,
        nid: u16,
    }

    let entry_size: usize = 14;
    let mut entries = Vec::new();
    for i in 0..id_count as usize {
        let offset = 6 + i * entry_size;
        if offset + entry_size > group_data.len() {
            break;
        }
        let width = group_data[offset];
        let height = group_data[offset + 1];
        let _palette = group_data[offset + 2];
        let _reserved = group_data[offset + 3];
        let _planes = u16::from_le_bytes([group_data[offset + 4], group_data[offset + 5]]);
        let bit_count = u16::from_le_bytes([group_data[offset + 6], group_data[offset + 7]]);
        let _bytes_in_res = u32::from_le_bytes([
            group_data[offset + 8],
            group_data[offset + 9],
            group_data[offset + 10],
            group_data[offset + 11],
        ]);
        let nid = u16::from_le_bytes([group_data[offset + 12], group_data[offset + 13]]);
        entries.push(DirEntry {
            width,
            height,
            bit_count,
            nid,
        });
    }

    // Select: largest area -> highest bit depth -> smallest resource ID
    entries.sort_by(|a, b| {
        let area_a = (if a.width == 0 { 256u32 } else { a.width as u32 })
            * (if a.height == 0 {
                256u32
            } else {
                a.height as u32
            });
        let area_b = (if b.width == 0 { 256u32 } else { b.width as u32 })
            * (if b.height == 0 {
                256u32
            } else {
                b.height as u32
            });
        area_b
            .cmp(&area_a)
            .then_with(|| b.bit_count.cmp(&a.bit_count))
            .then_with(|| a.nid.cmp(&b.nid))
    });

    // Some producers include a corrupt or non-decodable 256px resource while
    // still providing a valid 128/64px frame. Try all entries in quality order
    // instead of falling back to the shell (which often returns only 32px).
    for candidate in entries {
        let resource_name = crate::windows::resource::ResourceName::Id(candidate.nid);
        let Ok(icon_data) = load_resource_bytes(module, RT_ICON, &resource_name) else {
            continue;
        };
        let mut ico_bytes = Vec::with_capacity(22 + icon_data.len());
        ico_bytes.extend_from_slice(&[0u8, 0, 1, 0, 1, 0]);
        ico_bytes.push(candidate.width);
        ico_bytes.push(candidate.height);
        ico_bytes.extend_from_slice(&[0, 0]);
        ico_bytes.extend_from_slice(&1u16.to_le_bytes());
        ico_bytes.extend_from_slice(&candidate.bit_count.to_le_bytes());
        ico_bytes.extend_from_slice(&(icon_data.len() as u32).to_le_bytes());
        ico_bytes.extend_from_slice(&22u32.to_le_bytes());
        ico_bytes.extend_from_slice(&icon_data);

        let Ok(icon_dir) = ico::IconDir::read(std::io::Cursor::new(&ico_bytes)) else {
            continue;
        };
        let Some(entry) = icon_dir.entries().first() else {
            continue;
        };
        let Ok(decoded) = entry.decode() else {
            continue;
        };
        if let Some(image) = image::RgbaImage::from_raw(
            decoded.width(),
            decoded.height(),
            decoded.rgba_data().to_vec(),
        ) {
            return Ok(image);
        }
    }

    Err(AppError::IconExtractionFailed(
        "no decodable icon frame in selected PE group".into(),
        path.to_path_buf(),
    ))
}

fn select_group_icon(
    module: HMODULE,
    group_icons: &[crate::windows::resource::ResourceName],
    selector: IconSelector,
) -> Option<&crate::windows::resource::ResourceName> {
    match selector {
        IconSelector::Largest => group_icons.iter().max_by_key(|name| {
            load_resource_bytes(module, RT_GROUP_ICON, name)
                .ok()
                .and_then(|data| best_group_score(&data))
                .unwrap_or((0, 0))
        }),
        IconSelector::Index(index) => group_icons.get(index as usize),
        IconSelector::ResourceId(id) => group_icons.iter().find(|name| {
            matches!(name, crate::windows::resource::ResourceName::Id(value) if *value == id)
        }),
    }
}

fn best_group_score(data: &[u8]) -> Option<(u32, u16)> {
    if data.len() < 6
        || u16::from_le_bytes([data[0], data[1]]) != 0
        || u16::from_le_bytes([data[2], data[3]]) != 1
    {
        return None;
    }
    let count = u16::from_le_bytes([data[4], data[5]]) as usize;
    let mut best = (0u32, 0u16);
    for index in 0..count {
        let offset = 6 + index * 14;
        if offset + 14 > data.len() {
            break;
        }
        let width = if data[offset] == 0 {
            256u32
        } else {
            data[offset] as u32
        };
        let height = if data[offset + 1] == 0 {
            256u32
        } else {
            data[offset + 1] as u32
        };
        let bit_count = u16::from_le_bytes([data[offset + 6], data[offset + 7]]);
        best = best.max((width * height, bit_count));
    }
    Some(best)
}

pub fn enumerate_group_icons(
    module: HMODULE,
) -> Result<Vec<crate::windows::resource::ResourceName>, AppError> {
    crate::windows::resource::enumerate_group_icons(module)
}

pub fn load_resource_bytes(
    module: HMODULE,
    resource_type: PCWSTR,
    name: &crate::windows::resource::ResourceName,
) -> Result<Vec<u8>, AppError> {
    crate::windows::resource::load_resource_bytes(module, resource_type, name)
}

struct ModuleGuard(HMODULE);
impl Drop for ModuleGuard {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = FreeLibrary(self.0);
            }
        }
    }
}
