//! 外壳：三栏布局 + 窗口级副作用（标题栏、主题、快捷键、拖放）。
//!
//! 布局是「工具栏 / 来源列表 / 画布 / 检查器 / 状态栏」五块的自有 grid，
//! 不套 `.l-nav` + `.l-main`：那套是「导航覆盖内容」的读法，而这里三栏是并列的。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::bridge::{self, DragEvent};
use crate::components::dialogs::{ErrorBanner, ExportResultDialog, SettingsDialog};
use crate::components::editor::SourceEditor;
use crate::components::inspector::InspectorPanel;
use crate::components::preview::PreviewPane;
use crate::components::sidebar::SourceList;
use crate::components::toolbar::WindowToolbar;
use crate::state::App;
use crate::types::*;

#[component]
pub fn Shell() -> impl IntoView {
    let app = expect_context::<App>();
    let theme = use_context::<Theme>();

    // 主题：把上次的选择套回去。设置面板里也会改，这里负责「启动时」。
    Effect::new(move |_| {
        if let Some(theme) = theme {
            theme.set_mode(match app.theme_pref.get() {
                ThemePreference::Light => ThemeMode::Light,
                ThemePreference::Dark => ThemeMode::Dark,
                ThemePreference::System => ThemeMode::System,
            });
        }
    });

    // 全局快捷键。桌面工具的肌肉记忆，比工具栏快。
    Effect::new(move |_| {
        let handle = window_event_listener(leptos::ev::keydown, move |ev: web_sys::KeyboardEvent| {
            let key = ev.key();
            let command = ev.ctrl_key() || ev.meta_key();

            if command && key.eq_ignore_ascii_case("o") {
                ev.prevent_default();
                app.pick_and_import();
                return;
            }
            if command && key.eq_ignore_ascii_case("e") {
                ev.prevent_default();
                app.export_selected_ico();
                return;
            }
            if key == "Delete" || key == "Backspace" {
                if app.wand_selection.get_untracked().is_some() {
                    ev.prevent_default();
                    app.delete_wand_selection();
                }
                return;
            }
            if key == "Escape" {
                if app.source_editor_open.get_untracked() {
                    app.tool.set(Tool::None);
                    app.wand_selection.set(None);
                    app.source_editor_open.set(false);
                } else if app.settings_open.get_untracked() {
                    app.settings_open.set(false);
                } else if app.export_result.get_untracked().is_some() {
                    app.export_result.set(None);
                }
            }
        });
        on_cleanup(move || handle.remove());
    });

    // 系统拖放：Tauri 会拦截 HTML5 drop，所以走原生事件。
    Effect::new(move |_| {
        bridge::subscribe_drag_drop(move |event| match event {
            DragEvent::Over => app.drag_active.set(true),
            DragEvent::Leave => app.drag_active.set(false),
            DragEvent::Drop(paths) => {
                app.drag_active.set(false);
                app.import_paths(paths);
            }
        });
    });

    view! {
        <AppShell class="if-shell">
            <div class="if-layout">
                <div class="if-toolbar-slot">
                    <WindowToolbar />
                </div>

                <SourceList />
                <PreviewPane />
                <InspectorPanel />

                <div class="if-status">
                    <StatusBar right=Box::new(move || view! {
                        <StatusItem label="画布" value="256×256" />
                        <IconButton
                            icon=IconName::SETTINGS
                            label="软件设置"
                            size=Size::Sm
                            on_click=Callback::new(move |_| app.settings_open.set(true))
                        />
                    }.into_any())>
                        {move || {
                            if app.importing.get() {
                                view! {
                                    <StatusItem icon=IconName::IMAGES label="导入" value="进行中" live=StatusTone::Accent />
                                }.into_any()
                            } else if app.exporting.get() {
                                view! {
                                    <StatusItem icon=IconName::DOWNLOAD label="导出" value="进行中" live=StatusTone::Accent />
                                }.into_any()
                            } else {
                                view! {
                                    <StatusItem
                                        icon=IconName::IMAGES
                                        label="导入项"
                                        value=format!("{} 个", app.item_count())
                                    />
                                }.into_any()
                            }
                        }}
                    </StatusBar>
                </div>

                <Show when=move || app.source_editor_open.get()>
                    <SourceEditor />
                </Show>
                <SettingsDialog />
                <ExportResultDialog />
                <ErrorBanner />
            </div>
        </AppShell>
    }
}
