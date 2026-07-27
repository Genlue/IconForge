use std::path::{Path, PathBuf};

use thiserror::Error;

use crate::domain::response::CommandError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Path not found: {0}")]
    PathNotFound(PathBuf),

    #[error("Unsupported image: {0}")]
    UnsupportedImage(String, PathBuf),

    #[error("Icon extraction failed: {0}")]
    IconExtractionFailed(String, PathBuf),

    #[error("Shortcut read failed: {0}")]
    ShortcutReadFailed(String, PathBuf),

    #[error("Shortcut write failed: {0}")]
    ShortcutWriteFailed(String, PathBuf),

    #[error("Render failed: {0}")]
    RenderFailed(String),

    #[error("ICO encode failed: {0}")]
    IcoEncodeFailed(String),

    #[error("IO error: {0}")]
    IoFailed(String, Option<PathBuf>),

    #[error("Platform unsupported")]
    PlatformUnsupported,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidArgument(_) => "InvalidArgument",
            Self::PathNotFound(_) => "PathNotFound",
            Self::UnsupportedImage(_, _) => "UnsupportedImage",
            Self::IconExtractionFailed(_, _) => "IconExtractionFailed",
            Self::ShortcutReadFailed(_, _) => "ShortcutReadFailed",
            Self::ShortcutWriteFailed(_, _) => "ShortcutWriteFailed",
            Self::RenderFailed(_) => "RenderFailed",
            Self::IcoEncodeFailed(_) => "IcoEncodeFailed",
            Self::IoFailed(_, _) => "IoFailed",
            Self::PlatformUnsupported => "PlatformUnsupported",
            Self::Internal(_) => "Internal",
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::InvalidArgument(_)
            | Self::RenderFailed(_)
            | Self::IcoEncodeFailed(_)
            | Self::Internal(_)
            | Self::PlatformUnsupported => None,
            Self::PathNotFound(p)
            | Self::UnsupportedImage(_, p)
            | Self::IconExtractionFailed(_, p)
            | Self::ShortcutReadFailed(_, p)
            | Self::ShortcutWriteFailed(_, p) => Some(p.as_path()),
            Self::IoFailed(_, p) => p.as_deref(),
        }
    }

    pub fn into_command_error(self) -> CommandError {
        let path = self.path().map(|p| p.to_string_lossy().to_string());
        let details = if cfg!(debug_assertions) {
            Some(format!("{:?}", &self))
        } else {
            None
        };
        CommandError {
            code: self.code().to_string(),
            message: self.to_string(),
            path,
            details,
        }
    }

    pub fn io_error(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::IoFailed(message.into(), path)
    }

    pub fn io_for(path: &Path) -> impl FnOnce(std::io::Error) -> Self + '_ {
        |e| Self::IoFailed(e.to_string(), Some(path.to_path_buf()))
    }

    pub fn unsupported_image_for(path: &Path) -> impl FnOnce(image::ImageError) -> Self + '_ {
        |e| Self::UnsupportedImage(e.to_string(), path.to_path_buf())
    }

    pub fn ico_encode(e: std::io::Error) -> Self {
        Self::IcoEncodeFailed(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Internal(format!("JSON error: {}", e))
    }
}

impl From<AppError> for CommandError {
    fn from(e: AppError) -> Self {
        e.into_command_error()
    }
}
