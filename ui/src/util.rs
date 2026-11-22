//! 小工具：延时、数值夹取、颜色规范化。

use js_sys::Promise;

/// `setTimeout` 的 future 版本。用于预览防抖，不引第三方定时器 crate。
pub async fn sleep_ms(ms: i32) {
    let promise = Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// 渲染端只认 `#RRGGBBAA`。颜色控件可能给出 `#rrggbb`，这里补齐并统一大写；
/// 解析失败时返回 `fallback`，绝不把半截字符串送到宿主。
pub fn normalize_color8(value: &str, fallback: &str) -> String {
    let hex = value.trim().trim_start_matches('#');
    let digits: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    match digits.len() {
        6 => format!("#{}FF", digits.to_uppercase()),
        8 => format!("#{}", digits.to_uppercase()),
        _ => fallback.to_string(),
    }
}

/// 只换 RGB，保留原有 alpha。
pub fn replace_rgb(current: &str, rgb_hex: &str) -> String {
    let alpha = normalize_color8(current, "#000000FF");
    let alpha = &alpha[7..9];
    let rgb = normalize_color8(rgb_hex, "#000000FF");
    format!("#{}{}", &rgb[1..7], alpha)
}

pub fn alpha_percent(value: &str) -> u8 {
    let hex = normalize_color8(value, "#000000FF");
    let alpha = u8::from_str_radix(&hex[7..9], 16).unwrap_or(255);
    ((alpha as f32 / 255.0) * 100.0).round() as u8
}

pub fn file_name(path: &str) -> String {
    path.rsplit(['\\', '/']).next().unwrap_or(path).to_string()
}

/// 弹一个系统级文件选择器，返回选中的路径。
pub async fn pick_paths() -> Vec<String> {
    match crate::api::pick_import_files().await {
        Ok(paths) => paths,
        Err(err) => {
            leptos::logging::error!("pick_import_files: {}", err.message);
            Vec::new()
        }
    }
}

/// 把 data URL 解码进 `HtmlImageElement`，返回一个在 `onload` 后才 resolve 的 future。
pub async fn load_image(data_url: &str) -> Option<web_sys::HtmlImageElement> {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::{JsCast, JsValue};

    let image = web_sys::HtmlImageElement::new().ok()?;

    let promise = Promise::new(&mut |resolve, reject| {
        let loaded = image.clone();
        let on_load = Closure::once(move || {
            let _ = resolve.call1(&JsValue::NULL, loaded.as_ref());
        });
        image.set_onload(Some(on_load.as_ref().unchecked_ref::<js_sys::Function>()));
        on_load.forget();

        let on_error = Closure::once(move || {
            let _ = reject.call1(&JsValue::NULL, &JsValue::from_str("image decode failed"));
        });
        image.set_onerror(Some(on_error.as_ref().unchecked_ref::<js_sys::Function>()));
        on_error.forget();
    });

    image.set_src(data_url);
    wasm_bindgen_futures::JsFuture::from(promise).await.ok()?;
    Some(image)
}
