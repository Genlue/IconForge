//! IconForge 前端入口（Leptos 0.8 CSR + GlassDesign）。
//!
//! 一行 `bootstrap` 做完四件事：恢复玻璃档位、注入样式与贴图、建主题与浮层栈、
//! 挂载根组件。应用自己的状态在 [`Root`] 里 provide 一次，后面所有组件
//! 用 `expect_context::<App>()` 取。

mod api;
mod bridge;
mod canvas;
mod components;
mod defaults;
mod state;
mod storage;
mod types;
mod util;

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::components::shell::Shell;
use crate::state::App;

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    bootstrap_with(
        Root,
        BootstrapOptions {
            // 图标工具默认浅色：判断颜色时，界面本身不该带色偏。
            theme: ThemeMode::Light,
            ..Default::default()
        },
    );
}

#[component]
fn Root() -> impl IntoView {
    let app = App::new();
    provide_context(app);
    view! { <Shell /> }
}
