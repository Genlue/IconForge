use std::path::PathBuf;
use std::sync::Arc;

use image::RgbaImage;
use lru::LruCache;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::error::app_error::AppError;
use crate::windows::com::ComStaWorker;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SourceCacheKey {
    pub normalized_path: PathBuf,
    pub file_len: u64,
    pub modified_nanos: u128,
    pub shortcut_signature: Option<String>,
}

impl SourceCacheKey {
    pub fn for_path(path: &PathBuf) -> Result<Self, AppError> {
        let meta = std::fs::metadata(path).map_err(|_| AppError::PathNotFound(path.clone()))?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        Ok(Self {
            normalized_path: path.clone(),
            file_len: meta.len(),
            modified_nanos: modified,
            shortcut_signature: None,
        })
    }
}

pub struct AppState {
    pub source_cache: RwLock<LruCache<SourceCacheKey, Arc<RgbaImage>>>,
    pub com_sta_worker: ComStaWorker,
    pub instance_id: String,
}

impl AppState {
    pub fn new() -> Result<Self, AppError> {
        let com_sta_worker = ComStaWorker::spawn()?;
        Ok(Self {
            source_cache: RwLock::new(LruCache::new(std::num::NonZeroUsize::new(256).unwrap())),
            com_sta_worker,
            instance_id: Uuid::new_v4().to_string(),
        })
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        let _ = self.com_sta_worker.shutdown();
    }
}
