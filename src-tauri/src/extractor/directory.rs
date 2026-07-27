use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::domain::input::InputFileType;
use crate::error::app_error::AppError;
use crate::extractor::shell_icon;

use super::ExtractedIcon;

#[derive(Debug, Clone, Copy)]
pub struct DirectoryLimits {
    pub max_depth: usize,
    pub max_files: usize,
}

impl Default for DirectoryLimits {
    fn default() -> Self {
        Self {
            max_depth: 32,
            max_files: 10_000,
        }
    }
}

pub struct ExtractedImport {
    pub source_path: PathBuf,
    pub file_type: InputFileType,
    pub icon: ExtractedIcon,
    pub parent_directory_path: Option<PathBuf>,
}

pub struct DirectoryExpansion {
    pub root: ExtractedImport,
    pub children: Vec<Result<ExtractedImport, AppError>>,
    pub warnings: Vec<String>,
}

pub fn expand_directory(
    path: &Path,
    limits: DirectoryLimits,
) -> Result<DirectoryExpansion, AppError> {
    let root_icon = extract_directory_icon(path)?;
    let root = ExtractedImport {
        source_path: path.to_path_buf(),
        file_type: InputFileType::Directory,
        icon: root_icon,
        parent_directory_path: None,
    };

    let mut children: Vec<Result<ExtractedImport, AppError>> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let mut file_count = 0;

    let mut entries: Vec<walkdir::DirEntry> = WalkDir::new(path)
        .follow_links(false)
        .max_depth(limits.max_depth as usize)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            let p = e.path();
            !is_hidden_system_dir(p)
        })
        .collect();

    entries.sort_by(|a, b| {
        let a_path = a.path().to_string_lossy().to_lowercase();
        let b_path = b.path().to_string_lossy().to_lowercase();
        a_path.cmp(&b_path)
    });

    for entry in entries {
        if file_count >= limits.max_files {
            warnings.push(format!(
                "directory scan limit reached: {} files",
                limits.max_files
            ));
            break;
        }

        let child_path = entry.path().to_path_buf();
        let result = process_child_file(&child_path, path);
        if result.is_ok() {
            file_count += 1;
        }
        children.push(result);
    }

    Ok(DirectoryExpansion {
        root,
        children,
        warnings,
    })
}

fn is_hidden_system_dir(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_lowercase();
    lower.contains("$recycle.bin") || lower.contains("system volume information")
}

fn process_child_file(path: &Path, parent: &Path) -> Result<ExtractedImport, AppError> {
    use crate::extractor::detect;
    use crate::extractor::image_file;

    let file_type = detect::classify(path)?;
    let icon = match file_type {
        InputFileType::Image => {
            match image_file::decode_image(path) {
                Ok(pixels) => ExtractedIcon {
                    pixels,
                    resolved_target_path: None,
                    warnings: vec![],
                },
                Err(_) => {
                    // Fallback to shell icon for failed image decode
                    shell_icon::extract_shell_icon(path, 256).map(|pixels| ExtractedIcon {
                        pixels,
                        resolved_target_path: None,
                        warnings: vec![],
                    })?
                }
            }
        }
        InputFileType::Exe => {
            shell_icon::extract_shell_icon(path, 256).map(|pixels| ExtractedIcon {
                pixels,
                resolved_target_path: None,
                warnings: vec![],
            })?
        }
        InputFileType::Lnk => {
            // For dir children, just get shell icon
            shell_icon::extract_shell_icon(path, 256).map(|pixels| ExtractedIcon {
                pixels,
                resolved_target_path: None,
                warnings: vec![],
            })?
        }
        InputFileType::Directory => {
            return Err(AppError::InvalidArgument(
                "directories are not listed as children".into(),
            ));
        }
    };

    Ok(ExtractedImport {
        source_path: path.to_path_buf(),
        file_type,
        icon,
        parent_directory_path: Some(parent.to_path_buf()),
    })
}

pub fn extract_directory_icon(path: &Path) -> Result<ExtractedIcon, AppError> {
    shell_icon::extract_shell_icon(path, 256).map(|pixels| ExtractedIcon {
        pixels,
        resolved_target_path: None,
        warnings: vec![],
    })
}
