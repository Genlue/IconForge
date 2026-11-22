//! 浮层：设置、导出结果、错误提示。
//!
//! 三个都是瞬态表面，所以都走玻璃/浮层栈；常驻的侧栏与检查器不叠玻璃。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::components::fields::{flag, seg_options, select_options, Hint, SegmentedRow, SelectRow};
use crate::state::App;
use crate::types::*;

#[component]
pub fn SettingsDialog() -> impl IntoView {
    let app = expect_context::<App>();
    let theme = use_context::<Theme>();

    let open = move || app.settings_open.get();
    let theme_value = Signal::derive(move || match app.theme_pref.get() {
        ThemePreference::System => "system".to_string(),
        ThemePreference::Light => "light".to_string(),
        ThemePreference::Dark => "dark".to_string(),
    });
    let close = Callback::new(move |_: ()| app.settings_open.set(false));

    // 玻璃档位。**不用** `<GlassEffectsSetting/>`：那个控件是原生 `<select>`，
    // 在 WebView2 里弹层由系统绘制、跟着操作系统明暗走，而选项文字继承页面的
    // `color: inherit` —— 深色界面 + 浅色系统就是「白底浅灰字」，一个字都读不出来。
    // 这里换成库里的 `Select`（浮层栈 + 主题令牌），外观和检查器里的下拉一致。
    let effects_current = RwSignal::new(effects());
    let effects_value = Signal::derive(move || effects_current.get().as_str().to_string());

    view! {
        <Dialog
            open=flag(open)
            title="软件设置"
            description="窗口外观与玻璃材质。改动即时生效，下次启动沿用。"
            width=400
            on_close=close
        >
            <div class="if-settings">
                <SegmentedRow
                    label="外观"
                    options=seg_options(&[("system", "跟随系统"), ("light", "浅色"), ("dark", "深色")])
                    value=theme_value
                    on_change=Callback::new(move |value: String| {
                        let preference = match value.as_str() {
                            "light" => ThemePreference::Light,
                            "dark" => ThemePreference::Dark,
                            _ => ThemePreference::System,
                        };
                        app.theme_pref.set(preference);
                        if let Some(theme) = theme {
                            theme.set_mode(match preference {
                                ThemePreference::Light => ThemeMode::Light,
                                ThemePreference::Dark => ThemeMode::Dark,
                                ThemePreference::System => ThemeMode::System,
                            });
                        }
                        app.persist_settings();
                    })
                />

                <div class="if-settings__effects">
                    <SelectRow
                        label="玻璃效果"
                        options=select_options(&[
                            ("auto", "完整效果"),
                            ("blur", "仅模糊"),
                            ("solid", "性能优先"),
                        ])
                        value=effects_value
                        width=150
                        on_change=Callback::new(move |value: String| {
                            if let Some(mode) = GlassEffects::from_value(&value) {
                                effects_current.set(mode);
                                save_effects_preference(mode);
                            }
                        })
                    />
                    <Hint>"低配机器可切「仅模糊」或「性能优先」；选择会记住。"</Hint>
                </div>

                <p class="if-settings__about">
                    {format!(
                        "IconForge {} · GlassDesign {}",
                        env!("CARGO_PKG_VERSION"),
                        glass_design::VERSION,
                    )}
                </p>

                <div class="c-dialog__actions">
                    <Button
                        label="完成"
                        variant=ButtonVariant::Primary
                        on_click=Callback::new(move |_| app.settings_open.set(false))
                    />
                </div>
            </div>
        </Dialog>
    }
}

#[component]
pub fn ExportResultDialog() -> impl IntoView {
    let app = expect_context::<App>();
    let shown = RwSignal::new(None::<ExportResult>);
    let open = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(result) = app.export_result.get() {
            shown.set(Some(result));
            open.set(true);
        }
    });

    let close = Callback::new(move |_: ()| {
        open.set(false);
        app.export_result.set(None);
    });

    let title = move || {
        shown
            .get()
            .map(|result| {
                if result.issue_count() > 0 {
                    "导出完成（有提示）"
                } else {
                    "导出完成"
                }
            })
            .unwrap_or("导出完成")
    };
    let subtitle = move || {
        shown
            .get()
            .map(|result| {
                let success = result.success_count();
                let issues = result.issue_count();
                if issues > 0 {
                    format!("成功 {success} 项，提示 {issues} 项")
                } else {
                    format!("成功 {success} 项")
                }
            })
            .unwrap_or_default()
    };

    view! {
        <Dialog open=open title=title() description=subtitle() width=560 on_close=close>
            <div class="if-results">
                {move || {
                    shown
                        .get()
                        .map(|result| {
                            result
                                .rows()
                                .into_iter()
                                .map(|(label, path, warning)| {
                                    let class = if warning {
                                        "if-result if-result--warn"
                                    } else {
                                        "if-result"
                                    };
                                    view! {
                                        <div class=class>
                                            <span class="if-result__label">{label}</span>
                                            <span class="if-result__path">{path}</span>
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                }}
            </div>
            <div class="c-dialog__actions">
                <Button
                    label="确定"
                    variant=ButtonVariant::Primary
                    on_click=Callback::new(move |_| close.run(()))
                />
            </div>
        </Dialog>
    }
}

#[component]
pub fn ErrorBanner() -> impl IntoView {
    let app = expect_context::<App>();

    view! {
        {move || {
            app.error.get().map(|error| {
                let detail = {
                    let mut extra = String::new();
                    if let Some(path) = error.path.clone() {
                        extra.push_str(&path);
                    }
                    if let Some(details) = error.details.clone() {
                        if !extra.is_empty() {
                            extra.push('\n');
                        }
                        extra.push_str(&details);
                    }
                    (!extra.is_empty()).then_some(extra)
                };
                view! {
                    <div class="if-error">
                        <InlineMessage
                            tone=Tone::Danger
                            title="操作失败"
                            description=error.message
                            dismissible=true
                            on_dismiss=Callback::new(move |_| app.clear_error())
                        />
                        {detail.map(|detail| view! { <pre class="if-error__detail">{detail}</pre> })}
                    </div>
                }
            })
        }}
    }
}
