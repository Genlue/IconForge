//! 窗口与拖放的 JS 桥（实现见 `ui/bridge.js`）。
//!
//! 这一层只做三件事：拿 `window.__iconforge`、把 Rust 闭包变成 JS 回调、
//! 把 Promise 变成 future。浏览器里直接打开（没有宿主）时全部退化成无操作。

use js_sys::{Function, Reflect};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

fn api() -> Option<JsValue> {
    let value = Reflect::get(&js_sys::global(), &JsValue::from_str("__iconforge")).ok()?;
    (!value.is_undefined() && !value.is_null()).then_some(value)
}

fn api_fn(name: &str) -> Option<Function> {
    Reflect::get(&api()?, &JsValue::from_str(name))
        .ok()?
        .dyn_into::<Function>()
        .ok()
}

/// 窗口动作：`close` / `minimize` / `toggle-maximize`。
pub fn window_action(action: &'static str) {
    if let Some(f) = api_fn("windowAction") {
        let _ = f.call1(&JsValue::NULL, &JsValue::from_str(action));
    }
}

/// 订阅「最大化 / 焦点」变化。回调会先被调用一次，给出当前状态。
pub fn subscribe_window_state(cb: impl Fn(bool, bool) + 'static) {
    let Some(f) = api_fn("onWindowState") else {
        return;
    };
    let closure = Closure::<dyn FnMut(JsValue, JsValue)>::new(move |maximized: JsValue, focused: JsValue| {
        cb(
            maximized.as_bool().unwrap_or(false),
            focused.as_bool().unwrap_or(true),
        );
    });
    let _ = f.call1(&JsValue::NULL, closure.as_ref());
    // 订阅活到窗口结束：不保留句柄，也不需要退订。
    closure.forget();
}

#[derive(Clone, Debug, PartialEq)]
pub enum DragEvent {
    Over,
    Leave,
    Drop(Vec<String>),
}

pub fn subscribe_drag_drop(cb: impl Fn(DragEvent) + 'static) {
    let Some(f) = api_fn("onDragDrop") else {
        return;
    };
    let closure = Closure::<dyn FnMut(JsValue, JsValue)>::new(move |kind: JsValue, paths: JsValue| {
        match kind.as_string().as_deref() {
            Some("over") => cb(DragEvent::Over),
            Some("leave") => cb(DragEvent::Leave),
            Some("drop") => {
                let paths = js_sys::Array::from(&paths)
                    .iter()
                    .filter_map(|path| path.as_string())
                    .collect();
                cb(DragEvent::Drop(paths));
            }
            _ => {}
        }
    });
    let _ = f.call1(&JsValue::NULL, closure.as_ref());
    closure.forget();
}
