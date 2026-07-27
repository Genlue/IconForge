use windows::core::{BOOL as WinBOOL, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::*;
use windows::Win32::UI::WindowsAndMessaging::RT_GROUP_ICON;

use crate::error::app_error::AppError;

pub enum ResourceName {
    Id(u16),
    Name(Vec<u16>),
}

pub fn enumerate_group_icons(module: HMODULE) -> Result<Vec<ResourceName>, AppError> {
    let mut names = Vec::new();

    // SAFETY: EnumResourceNamesW with a callback that collects resource names
    extern "system" fn enum_callback(
        _hmodule: HMODULE,
        _lptype: PCWSTR,
        lpname: PCWSTR,
        lparam: isize,
    ) -> WinBOOL {
        let names = unsafe { &mut *(lparam as *mut Vec<ResourceName>) };
        let ptr = lpname.as_ptr();
        if (ptr as usize) >> 16 == 0 {
            let id = (ptr as u16).to_be();
            names.push(ResourceName::Id(id));
        } else {
            let mut len = 0;
            unsafe {
                while *ptr.add(len) != 0 {
                    len += 1;
                }
            }
            let chars = unsafe { std::slice::from_raw_parts(ptr, len) };
            names.push(ResourceName::Name(chars.to_vec()));
        }
        WinBOOL(1)
    }

    // SAFETY: EnumResourceNamesW callback
    let result = unsafe {
        EnumResourceNamesW(
            Some(module),
            RT_GROUP_ICON,
            Some(enum_callback),
            isize::from(&mut names as *mut _ as isize),
        )
    };

    if !result.as_bool() {
        return Ok(Vec::new());
    }

    Ok(names)
}

pub fn load_resource_bytes(
    module: HMODULE,
    resource_type: PCWSTR,
    name: &ResourceName,
) -> Result<Vec<u8>, AppError> {
    let name_ptr = match name {
        ResourceName::Id(id) => PCWSTR::from_raw(id.to_be() as usize as *const u16),
        ResourceName::Name(chars) => PCWSTR::from_raw(chars.as_ptr()),
    };

    // SAFETY: FindResourceW locates the resource
    let res_info = unsafe { FindResourceW(Some(module), name_ptr, resource_type) };

    if res_info.is_invalid() {
        return Err(AppError::IconExtractionFailed(
            "FindResourceW failed".into(),
            std::path::PathBuf::new(),
        ));
    }

    // SAFETY: SizeofResource gets the resource size
    let size = unsafe { SizeofResource(Some(module), res_info) };

    if size == 0 {
        return Err(AppError::IconExtractionFailed(
            "SizeofResource returned 0".into(),
            std::path::PathBuf::new(),
        ));
    }

    // SAFETY: LoadResource loads the resource data
    let handle = unsafe { LoadResource(Some(module), res_info) }.map_err(|_| {
        AppError::IconExtractionFailed("LoadResource failed".into(), std::path::PathBuf::new())
    })?;

    // SAFETY: LockResource returns a pointer to the resource data
    let res_data = unsafe { LockResource(handle) };

    if res_data.is_null() {
        return Err(AppError::IconExtractionFailed(
            "LockResource returned null".into(),
            std::path::PathBuf::new(),
        ));
    }

    let bytes = unsafe { std::slice::from_raw_parts(res_data as *const u8, size as usize) };

    Ok(bytes.to_vec())
}
