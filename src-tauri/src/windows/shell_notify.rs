use std::path::Path;

use windows::Win32::UI::Shell::*;

use super::wide_string::to_wide_null;

pub fn notify_icon_changed(lnk_path: &Path, icon_path: &Path) {
    notify_path(lnk_path, SHCNE_UPDATEITEM);
    notify_path(icon_path, SHCNE_UPDATEITEM);
}

fn notify_path(path: &Path, event: SHCNE_ID) {
    let wide = to_wide_null(path.as_os_str());
    // SAFETY: SHChangeNotify with a stable pointer to the wide string
    unsafe {
        SHChangeNotify(
            event,
            SHCNF_FLAGS(SHCNF_PATHW.0 | SHCNF_FLUSHNOWAIT.0),
            Some(wide.as_ptr() as *const core::ffi::c_void),
            None,
        );
    }
}

pub fn notify_associations_changed() {
    // SAFETY: SHCNE_ASSOCCHANGED with IDLIST takes no items
    unsafe {
        SHChangeNotify(
            SHCNE_ASSOCCHANGED,
            SHCNF_FLAGS(SHCNF_IDLIST.0 | SHCNF_FLUSHNOWAIT.0),
            None,
            None,
        );
    }
}
