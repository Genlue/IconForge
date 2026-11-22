//! localStorage 读写。
//!
//! 全部失败都静默降级成「没有偏好」：桌面应用里存储不可用（隐私模式、
//! 配额满）不该让界面白屏，用户最多是下次启动丢掉自定义预设。

use serde::de::DeserializeOwned;
use serde::Serialize;

fn store() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn read(key: &str) -> Option<String> {
    store()?.get_item(key).ok().flatten()
}

pub fn write(key: &str, value: &str) {
    if let Some(store) = store() {
        let _ = store.set_item(key, value);
    }
}

pub fn read_json<T: DeserializeOwned>(key: &str) -> Option<T> {
    let raw = read(key)?;
    serde_json::from_str(&raw).ok()
}

pub fn write_json<T: Serialize>(key: &str, value: &T) {
    if let Ok(raw) = serde_json::to_string(value) {
        write(key, &raw);
    }
}

/// 逐条解析：一条坏记录不该让整份自定义预设消失。
pub fn read_json_array<T: DeserializeOwned>(key: &str) -> Vec<T> {
    let Some(raw) = read(key) else {
        return Vec::new();
    };
    let Ok(values) = serde_json::from_str::<Vec<serde_json::Value>>(&raw) else {
        return Vec::new();
    };
    values
        .into_iter()
        .filter_map(|value| serde_json::from_value(value).ok())
        .collect()
}
