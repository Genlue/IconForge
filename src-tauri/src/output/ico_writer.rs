use std::io::Cursor;
use std::path::Path;

use image::RgbaImage;

use crate::error::app_error::AppError;

pub fn encode_ico(images: &[(u32, RgbaImage)]) -> Result<Vec<u8>, AppError> {
    let mut dir = ico::IconDir::new(ico::ResourceType::Icon);

    for (size, image) in images {
        let icon = ico::IconImage::from_rgba_data(*size, *size, image.clone().into_raw());
        // Windows shell consumers are most compatible with DIB data for the
        // classic small icon sizes. Keep PNG for larger, detail-heavy frames.
        let entry = if *size <= 64 {
            ico::IconDirEntry::encode_as_bmp(&icon)
        } else {
            ico::IconDirEntry::encode_as_png(&icon)
        }
        .map_err(AppError::ico_encode)?;
        dir.add_entry(entry);
    }
    let mut cursor = Cursor::new(Vec::new());
    dir.write(&mut cursor).map_err(AppError::ico_encode)?;
    Ok(cursor.into_inner())
}

pub fn write_ico_atomic(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    super::atomic_file::write_bytes_atomic(path, bytes, false)
}
