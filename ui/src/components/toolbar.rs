//! 窗口工具栏：红绿灯 | 标题 | 任务胶囊。
//!
//! 玻璃只出现在这一层（系统铬）与临时浮层上；侧栏、内容区、检查器都是不透明窗格。
//! 这样「同一视线最多两层玻璃」的约束自然成立。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::bridge;
use crate::components::fields::flag;
use crate::state::App;

#[component]
pub fn WindowToolbar() -> impl IntoView {
    let app = expect_context::<App>();
    let focused = RwSignal::new(true);
    let maximized = RwSignal::new(false);

    bridge::subscribe_window_state(move |is_maximized, is_focused| {
        maximized.set(is_maximized);
        focused.set(is_focused);
    });

    let subtitle = move || {
        app.current_item()
            .map(|item| item.display_name)
            .unwrap_or_else(|| "拖入图片、EXE、DLL、LNK 开始".to_string())
    };
    let can_export = move || app.item_count() > 0 && !app.exporting.get();
    let export_label = move || {
        if app.exporting.get() {
            "导出中…"
        } else {
            "导出 ICO"
        }
    };

    view! {
        <Toolbar>
            <ToolbarChrome>
                <TrafficLights
                    focused=focused
                    maximized=maximized
                    on_close=Callback::new(move |_| bridge::window_action("close"))
                    on_minimize=Callback::new(move |_| bridge::window_action("minimize"))
                    on_zoom=Callback::new(move |_| bridge::window_action("toggle-maximize"))
                />
            </ToolbarChrome>

            <ToolbarLead>
                // 标题与副标题都必须是信号：`ToolbarTitle` 的 subtitle 是 String，
                // 传进去就钉死了，换图标时不会跟着变。
                <span class="c-title">"IconForge"</span>
                <span class="if-subtitle">{move || subtitle()}</span>
            </ToolbarLead>

            <div class="if-spacer" data-tauri-drag-region="deep"></div>

            <Capsule>
                <Button
                    label="导入"
                    icon=IconName::UPLOAD
                    variant=ButtonVariant::Plain
                    size=Size::Sm
                    disabled=flag(move || app.importing.get())
                    on_click=Callback::new(move |_| app.pick_and_import())
                />
                <Button
                    label=export_label()
                    icon=IconName::DOWNLOAD
                    variant=ButtonVariant::Primary
                    size=Size::Sm
                    disabled=flag(move || !can_export())
                    on_click=Callback::new(move |_| app.export_selected_ico())
                />
            </Capsule>

            <Capsule size=CapsuleSize::Round>
                <IconButton
                    icon=IconName::SETTINGS
                    label="软件设置"
                    size=Size::Sm
                    on_click=Callback::new(move |_| app.settings_open.set(true))
                />
            </Capsule>
        </Toolbar>
    }
}
