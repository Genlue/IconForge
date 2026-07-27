pub mod detect;
pub mod directory;
pub mod hicon;
pub mod image_file;
pub mod pe_resource;
pub mod shell_icon;
pub mod shortcut;

use std::path::Path;

use image::RgbaImage;

use crate::error::app_error::AppError;
use crate::windows::com::ComStaWorker;

use self::detect::classify;

pub struct ExtractedIcon {
    pub pixels: RgbaImage,
    pub resolved_target_path: Option<std::path::PathBuf>,
    pub warnings: Vec<String>,
}

pub struct IconExtractor<'a> {
    #[allow(dead_code)]
    com_sta_worker: &'a ComStaWorker,
}

impl<'a> IconExtractor<'a> {
    pub fn new(com_sta_worker: &'a ComStaWorker) -> Self {
        Self { com_sta_worker }
    }

    pub fn classify(path: &Path) -> Result<crate::domain::input::InputFileType, AppError> {
        classify(path)
    }

    pub fn extract(&self, path: &Path) -> Result<ExtractedIcon, AppError> {
        let file_type = classify(path)?;
        match file_type {
            crate::domain::input::InputFileType::Image => {
                let img = image_file::decode_image(path)?;
                Ok(ExtractedIcon {
                    pixels: img,
                    resolved_target_path: None,
                    warnings: vec![],
                })
            }
            crate::domain::input::InputFileType::Exe => extract_exe_icon(path),
            crate::domain::input::InputFileType::Lnk => shortcut::extract_shortcut_icon(path, 8),
            crate::domain::input::InputFileType::Directory => {
                directory::extract_directory_icon(path)
            }
        }
    }
}

fn extract_exe_icon(path: &Path) -> Result<ExtractedIcon, AppError> {
    let mut warnings = vec![];
    // Try PE resource extraction first
    let result = pe_resource::extract_pe_icon(path, pe_resource::IconSelector::Largest);
    match result {
        Ok(pixels) => {
            return Ok(ExtractedIcon {
                pixels,
                resolved_target_path: None,
                warnings,
            });
        }
        Err(e) => {
            warnings.push(format!(
                "PE resource extraction failed: {}, falling back to Shell icon",
                e
            ));
        }
    }
    // Fallback to Shell icon
    match shell_icon::extract_shell_icon(path, 256) {
        Ok(pixels) => Ok(ExtractedIcon {
            pixels,
            resolved_target_path: None,
            warnings,
        }),
        Err(e) => Err(AppError::IconExtractionFailed(
            format!("all extraction methods failed: {}", e),
            path.to_path_buf(),
        )),
    }
}

#[cfg(not(target_os = "windows"))]
pub use platform_stubs::*;

#[cfg(not(target_os = "windows"))]
mod platform_stubs {
    use super::*;
    // Non-Windows stubs would go here
}
