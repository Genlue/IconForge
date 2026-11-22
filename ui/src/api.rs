//! 宿主调用桥。
//!
//! wasm 侧不引 `tauri` crate，也不引 `@tauri-apps/api`：`withGlobalTauri` 已经把
//! `window.__TAURI__.core.invoke` 注入了页面，这里只做「Rust 值 → JS 值 →
//! Promise → Rust 值」的翻译。
//!
//! 为什么走 JSON 而不是 serde-wasm-bindgen：契约本来就是 serde 结构
//! （`iconforge-domain`），多一层 JSON 只多一次拷贝，却省掉一套类型映射，
//! 出错时也能直接把原始文本打进错误详情。

use js_sys::{Function, Object, Reflect};
use serde::de::DeserializeOwned;
use serde::Serialize;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

use crate::types::*;

fn global_tauri() -> Option<JsValue> {
    let value = Reflect::get(&js_sys::global(), &JsValue::from_str("__TAURI__")).ok()?;
    (!value.is_undefined() && !value.is_null()).then_some(value)
}

fn invoke_fn() -> Option<Function> {
    let core = Reflect::get(&global_tauri()?, &JsValue::from_str("core")).ok()?;
    Reflect::get(&core, &JsValue::from_str("invoke"))
        .ok()?
        .dyn_into::<Function>()
        .ok()
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, CommandError> {
    let text = serde_json::to_string(value)
        .map_err(|e| CommandError::internal(format!("请求序列化失败：{e}")))?;
    js_sys::JSON::parse(&text).map_err(|_| CommandError::internal("请求无法转成 JS 值"))
}

fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, CommandError> {
    let text = js_sys::JSON::stringify(&value)
        .map(|s| String::from(s))
        .unwrap_or_else(|_| "null".to_string());
    serde_json::from_str(&text).map_err(|e| CommandError {
        code: "Internal".to_string(),
        message: format!("宿主返回值无法解析：{e}"),
        path: None,
        details: Some(text.chars().take(400).collect()),
    })
}

fn js_error(err: JsValue) -> CommandError {
    if let Some(text) = err.as_string() {
        return CommandError::internal(text);
    }
    let text = js_sys::JSON::stringify(&err)
        .map(|s| String::from(s))
        .unwrap_or_default();
    serde_json::from_str::<CommandError>(&text)
        .unwrap_or_else(|_| CommandError::internal(if text.is_empty() {
            "宿主调用失败".to_string()
        } else {
            text
        }))
}

/// 带 `{ request }` 参数的调用。
pub async fn call<T: Serialize, R: DeserializeOwned>(
    command: &str,
    request: &T,
) -> Result<R, CommandError> {
    let invoke = invoke_fn().ok_or_else(|| {
        CommandError::internal("未检测到 Tauri 宿主：请用 `npm run tauri dev` 启动")
    })?;
    let args = Object::new();
    let payload = to_js(request)?;
    Reflect::set(&args, &JsValue::from_str("request"), &payload)
        .map_err(|_| CommandError::internal("请求参数装配失败"))?;
    let promise = invoke
        .call2(&JsValue::NULL, &JsValue::from_str(command), &args)
        .map_err(js_error)?;
    let value = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(js_error)?;
    from_js(value)
}

/// 无参数调用。
pub async fn call_empty<R: DeserializeOwned>(command: &str) -> Result<R, CommandError> {
    let invoke = invoke_fn().ok_or_else(|| {
        CommandError::internal("未检测到 Tauri 宿主：请用 `npm run tauri dev` 启动")
    })?;
    let args = Object::new();
    let promise = invoke
        .call2(&JsValue::NULL, &JsValue::from_str(command), &args)
        .map_err(js_error)?;
    let value = JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(js_error)?;
    from_js(value)
}

pub async fn pick_import_files() -> Result<Vec<String>, CommandError> {
    call_empty("pick_import_files").await
}

pub async fn import_paths(paths: Vec<String>) -> Result<ImportPathsResponse, CommandError> {
    call("import_paths", &ImportPathsRequest { paths }).await
}

pub async fn render_preview(req: RenderPreviewRequest) -> Result<RenderPreviewResponse, CommandError> {
    call("render_preview", &req).await
}

pub async fn render_source_image(
    req: SourceImageRequest,
) -> Result<SourceImageResponse, CommandError> {
    call("render_source_image", &req).await
}

pub async fn compute_wand_selection(
    req: WandSelectionRequest,
) -> Result<WandSelectionResponse, CommandError> {
    call("compute_wand_selection", &req).await
}

pub async fn export_ico(items: Vec<ExportIcoItemRequest>) -> Result<ExportIcoResponse, CommandError> {
    call(
        "export_ico",
        &ExportIcoRequest {
            items,
            export_mode: ExportMode::ExportAsIco,
        },
    )
    .await
}

pub async fn export_png(
    items: Vec<ExportIcoItemRequest>,
    raw_source: bool,
) -> Result<ExportIcoResponse, CommandError> {
    call("export_png", &ExportPngRequest { items, raw_source }).await
}

pub async fn apply_to_lnk(items: Vec<ApplyToLnkItemRequest>) -> Result<ApplyToLnkResponse, CommandError> {
    call(
        "apply_to_lnk",
        &ApplyToLnkRequest {
            items,
            export_mode: ExportMode::ApplyToLnk,
        },
    )
    .await
}
