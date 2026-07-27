use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use image::RgbaImage;
use tauri::{AppHandle, Manager};

use crate::domain::config::UpscaleConfig;
use crate::error::app_error::AppError;
use crate::extractor::IconExtractor;
use crate::state::{AppState, SourceCacheKey, UpscaleCacheKey};

pub fn load_processed_source(
    app: &AppHandle,
    source_path: &Path,
    upscale: &UpscaleConfig,
    state: &AppState,
) -> Result<Arc<RgbaImage>, AppError> {
    let source_key = SourceCacheKey::for_path(&source_path.to_path_buf())?;
    let source = load_original_source(source_path, &source_key, state)?;

    if !upscale.enabled {
        return Ok(source);
    }

    validate_upscale_config(upscale)?;
    let cache_key = UpscaleCacheKey {
        source: source_key,
        config: upscale.clone(),
    };
    if let Some(cached) = state.upscale_cache.write().get(&cache_key).cloned() {
        return Ok(cached);
    }

    // Slider changes can queue several preview requests while enhancement is
    // running. Serialize the external GPU process and recheck the cache so a
    // source/config pair is enhanced only once.
    let _worker = state.upscale_worker.lock();
    if let Some(cached) = state.upscale_cache.write().get(&cache_key).cloned() {
        return Ok(cached);
    }

    let tool_dir = find_realcugan_dir(app)?;
    let exe_path = tool_dir.join("realcugan-ncnn-vulkan.exe");
    let model_dir = tool_dir.join(model_directory_name(&upscale.model)?);
    if !model_dir.is_dir() {
        return Err(AppError::PathNotFound(model_dir));
    }

    let temp_dir = tempfile::tempdir().map_err(|e| {
        AppError::io_error(format!("failed to create upscale workspace: {}", e), None)
    })?;
    let input_path = temp_dir.path().join("source.png");
    let output_path = temp_dir.path().join("upscaled.png");
    image::DynamicImage::ImageRgba8((*source).clone())
        .save_with_format(&input_path, image::ImageFormat::Png)
        .map_err(|e| AppError::RenderFailed(format!("failed to prepare upscale input: {}", e)))?;

    super::upscale_service::run_upscale(
        &exe_path,
        &input_path,
        &output_path,
        upscale.scale,
        upscale.denoise,
        &model_dir,
    )?;

    let enhanced = Arc::new(crate::extractor::image_file::decode_image(&output_path)?);
    state.upscale_cache.write().put(cache_key, enhanced.clone());
    Ok(enhanced)
}

fn load_original_source(
    source_path: &Path,
    cache_key: &SourceCacheKey,
    state: &AppState,
) -> Result<Arc<RgbaImage>, AppError> {
    if let Some(cached) = state.source_cache.write().get(cache_key).cloned() {
        return Ok(cached);
    }

    let extractor = IconExtractor::new(&state.com_sta_worker);
    let extracted = extractor.extract(source_path)?;
    let source = Arc::new(extracted.pixels);
    state
        .source_cache
        .write()
        .put(cache_key.clone(), source.clone());
    Ok(source)
}

fn validate_upscale_config(config: &UpscaleConfig) -> Result<(), AppError> {
    if !(2..=4).contains(&config.scale) {
        return Err(AppError::InvalidArgument(
            "upscale scale must be 2..=4".into(),
        ));
    }
    if !(-1..=3).contains(&config.denoise) {
        return Err(AppError::InvalidArgument(
            "upscale denoise must be -1..=3".into(),
        ));
    }

    match config.model.as_str() {
        "se" => {
            if config.scale >= 3 && matches!(config.denoise, 1 | 2) {
                return Err(AppError::InvalidArgument(
                    "SE 3x/4x supports denoise -1, 0, or 3".into(),
                ));
            }
        }
        "pro" => {
            if config.scale == 4 || matches!(config.denoise, 1 | 2) {
                return Err(AppError::InvalidArgument(
                    "Pro supports 2x/3x with denoise -1, 0, or 3".into(),
                ));
            }
        }
        "nose" => {
            if config.scale != 2 || config.denoise != -1 {
                return Err(AppError::InvalidArgument(
                    "No-SE supports only 2x without denoise".into(),
                ));
            }
        }
        _ => {
            return Err(AppError::InvalidArgument(format!(
                "unknown Real-CUGAN model: {}",
                config.model
            )))
        }
    }
    Ok(())
}

fn model_directory_name(model: &str) -> Result<&'static str, AppError> {
    match model {
        "se" => Ok("models-se"),
        "pro" => Ok("models-pro"),
        "nose" => Ok("models-nose"),
        _ => Err(AppError::InvalidArgument(format!(
            "unknown Real-CUGAN model: {}",
            model
        ))),
    }
}

fn find_realcugan_dir(app: &AppHandle) -> Result<PathBuf, AppError> {
    static TOOL_DIR: OnceLock<PathBuf> = OnceLock::new();
    if let Some(cached) = TOOL_DIR.get() {
        return Ok(cached.clone());
    }

    let mut candidates = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
            candidates.push(parent.join("realcugan-ncnn-vulkan-20220728-windows"));
        }
    }
    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")))
            .join("realcugan-ncnn-vulkan-20220728-windows"),
    );
    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.clone());
        candidates.push(resource_dir.join("realcugan-ncnn-vulkan-20220728-windows"));
        for entry in walkdir::WalkDir::new(&resource_dir)
            .max_depth(5)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_name() == "realcugan-ncnn-vulkan.exe" {
                if let Some(parent) = entry.path().parent() {
                    candidates.push(parent.to_path_buf());
                    break;
                }
            }
        }
    }

    for candidate in candidates {
        if candidate.join("realcugan-ncnn-vulkan.exe").is_file() {
            let _ = TOOL_DIR.set(candidate.clone());
            return Ok(candidate);
        }
    }

    Err(AppError::PathNotFound(PathBuf::from(
        "realcugan-ncnn-vulkan.exe",
    )))
}

#[cfg(test)]
mod tests {
    use super::validate_upscale_config;
    use crate::domain::config::UpscaleConfig;

    fn config(model: &str, scale: u32, denoise: i32) -> UpscaleConfig {
        UpscaleConfig {
            enabled: true,
            model: model.into(),
            scale,
            denoise,
        }
    }

    #[test]
    fn accepts_supported_model_combinations() {
        assert!(validate_upscale_config(&config("se", 2, 1)).is_ok());
        assert!(validate_upscale_config(&config("se", 4, 3)).is_ok());
        assert!(validate_upscale_config(&config("pro", 3, 0)).is_ok());
        assert!(validate_upscale_config(&config("nose", 2, -1)).is_ok());
    }

    #[test]
    fn rejects_missing_model_files_and_unsupported_combinations() {
        assert!(validate_upscale_config(&config("se", 4, 1)).is_err());
        assert!(validate_upscale_config(&config("pro", 4, 3)).is_err());
        assert!(validate_upscale_config(&config("nose", 2, 0)).is_err());
        assert!(validate_upscale_config(&config("unknown", 2, 0)).is_err());
    }
}
