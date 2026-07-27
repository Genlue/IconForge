use std::path::Path;

use crate::error::app_error::AppError;
use crate::windows::shell_link;

use super::detect;
use super::image_file;
use super::shell_icon;
use super::ExtractedIcon;

pub fn extract_shortcut_icon(path: &Path, max_depth: u8) -> Result<ExtractedIcon, AppError> {
    let mut warnings = vec![];
    let mut visited = std::collections::HashSet::new();
    let mut current_path = path.to_path_buf();

    for _depth in 0..max_depth {
        if !visited.insert(current_path.clone()) {
            warnings.push("shortcut cycle detected".into());
            break;
        }

        let info = shell_link::read_shortcut(&current_path)?;

        // Try icon location first
        if let Some(ref icon_path) = info.icon_location {
            if icon_path.exists() {
                let _icon_index = if info.icon_index >= 0 {
                    info.icon_index as u32
                } else {
                    // Absolute value for resource ID
                    info.icon_index.unsigned_abs()
                };

                // Extract from the icon path
                match detect::classify(icon_path) {
                    Ok(crate::domain::input::InputFileType::Image) => {
                        match image_file::decode_image(icon_path) {
                            Ok(pixels) => {
                                return Ok(ExtractedIcon {
                                    pixels,
                                    resolved_target_path: Some(current_path),
                                    warnings,
                                })
                            }
                            Err(e) => warnings.push(format!("icon location decode failed: {}", e)),
                        }
                    }
                    Ok(crate::domain::input::InputFileType::Exe) => {
                        match super::extract_exe_icon(icon_path) {
                            Ok(extracted) => {
                                return Ok(ExtractedIcon {
                                    pixels: extracted.pixels,
                                    resolved_target_path: Some(current_path),
                                    warnings,
                                })
                            }
                            Err(e) => {
                                warnings.push(format!("icon location PE extract failed: {}", e))
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Try target path
        if let Some(ref target) = info.target_path {
            if target.is_file() {
                match detect::classify(target) {
                    Ok(crate::domain::input::InputFileType::Lnk) => {
                        current_path = target.clone();
                        continue;
                    }
                    Ok(crate::domain::input::InputFileType::Image) => {
                        match image_file::decode_image(target) {
                            Ok(pixels) => {
                                return Ok(ExtractedIcon {
                                    pixels,
                                    resolved_target_path: Some(current_path),
                                    warnings,
                                })
                            }
                            Err(e) => warnings.push(format!("target decode failed: {}", e)),
                        }
                    }
                    Ok(crate::domain::input::InputFileType::Exe) => {
                        match super::extract_exe_icon(target) {
                            Ok(extracted) => {
                                return Ok(ExtractedIcon {
                                    pixels: extracted.pixels,
                                    resolved_target_path: Some(current_path),
                                    warnings,
                                })
                            }
                            Err(e) => warnings.push(format!("target EXE extract failed: {}", e)),
                        }
                    }
                    _ => {}
                }
            }
        }

        break;
    }

    // Final fallback: extract shell icon from the LNK file itself
    match shell_icon::extract_shell_icon(path, 256) {
        Ok(pixels) => Ok(ExtractedIcon {
            pixels,
            resolved_target_path: None,
            warnings,
        }),
        Err(e) => Err(AppError::ShortcutReadFailed(
            format!("all shortcut extraction methods failed: {}", e),
            path.to_path_buf(),
        )),
    }
}
