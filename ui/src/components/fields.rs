//! 检查器里复用的行控件。
//!
//! 全部走 GlassDesign 的 `Row` / `Slider` / `NumberField` / `ColorWell` / `Switch`：
//! 分隔线、焦点环、键盘语义都由库负责，这里只决定「一行里放什么」。
//!
//! **值一律是 `Signal` 而不是裸值**：检查器分组的函数体只跑一次，
//! 裸值会把「创建时那一刻」的参数永久钉死。拖滑杆时 `Slider` 自己持有拖拽值，
//! 外部信号更新不会打断拖拽。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::state::App;
use crate::types::{ColorTarget, Tool};
use crate::util;

/// 可折叠分组。`Disclosure` 负责高度动画与 `inert`，这里只补一层内边距。
#[component]
pub fn Section(
    #[prop(into)] title: String,
    #[prop(into, optional)] hint: String,
    #[prop(default = false)] open: bool,
    children: Children,
) -> impl IntoView {
    let expanded = RwSignal::new(open);
    view! {
        <Disclosure
            title=title
            hint=hint
            open=expanded
            on_toggle=Callback::new(move |value: bool| expanded.set(value))
        >
            <div class="if-section">{children()}</div>
        </Disclosure>
    }
}

#[component]
pub fn Hint(children: Children) -> impl IntoView {
    view! { <p class="if-hint">{children()}</p> }
}

/// 滑块 + 数值输入。两者写同一个值，谁动都提交。
#[component]
pub fn RangeField(
    #[prop(into)] label: String,
    value: Signal<f64>,
    min: f64,
    max: f64,
    #[prop(default = 1.0)] step: f64,
    #[prop(into, optional)] unit: String,
    #[prop(default = false)] disabled: bool,
    on_change: Callback<f64>,
) -> impl IntoView {
    let has_unit = !unit.is_empty();
    let row_label = label.clone();
    let slider_label = label.clone();
    view! {
        <Row label=row_label wide=true>
            <div class="if-slider">
                <Slider
                    value=value
                    min=min
                    max=max
                    step=step
                    disabled=disabled
                    label=slider_label
                    on_change=on_change
                />
            </div>
            <div class="if-number">
                <NumberField
                    value=value
                    min=min
                    max=max
                    step=step
                    disabled=disabled
                    label=label
                    on_change=on_change
                />
            </div>
            {has_unit.then(|| view! { <span class="if-unit">{unit.clone()}</span> })}
        </Row>
    }
}

/// 颜色行：色井（含取色面板）+ 可选吸管 + 不透明度读数。
#[component]
pub fn ColorField(
    #[prop(into)] label: String,
    value: Signal<String>,
    #[prop(default = false)] disabled: bool,
    #[prop(optional)] pick: Option<ColorTarget>,
    on_change: Callback<String>,
) -> impl IntoView {
    let app = expect_context::<App>();
    let normalized = Signal::derive(move || util::normalize_color8(&value.get(), "#000000FF"));
    let alpha = Signal::derive(move || util::alpha_percent(&normalized.get()).to_string());

    let commit = Callback::new(move |next: String| {
        let fallback = normalized.get_untracked();
        on_change.run(util::normalize_color8(&next, &fallback));
    });

    let picking_this = move || {
        app.color_target.get() == pick && app.tool.get() == Tool::Eyedropper
    };
    let label_for_pick = label.clone();
    let row_label = label.clone();

    view! {
        <Row label=row_label>
            <ColorWell
                value=normalized
                label=label_for_pick.clone()
                show_hex=false
                disabled=disabled
                on_change=commit
            />
            {pick.map(|target| view! {
                <IconButton
                    icon=IconName::EYE
                    label=format!("取色：{label_for_pick}")
                    size=Size::Sm
                    variant=IconButtonVariant::Plain
                    active=picking_this()
                    toggle=true
                    disabled=disabled
                    on_click=Callback::new(move |_| app.start_color_pick(target))
                />
            })}
            <span class="if-alpha">{move || alpha.get()}"%"</span>
        </Row>
    }
}

/// 开关行。
#[component]
pub fn ToggleRow(
    #[prop(into)] label: String,
    checked: Signal<bool>,
    on_change: Callback<bool>,
) -> impl IntoView {
    view! {
        <Row label=label>
            <Switch checked=checked on_change=on_change />
        </Row>
    }
}

/// 下拉行。
#[component]
pub fn SelectRow(
    #[prop(into)] label: String,
    options: Vec<SelectOption>,
    value: Signal<String>,
    #[prop(default = 132)] width: u32,
    on_change: Callback<String>,
) -> impl IntoView {
    let row_label = label.clone();
    view! {
        <Row label=row_label>
            <Select options=options value=value width=width label=label on_change=on_change />
        </Row>
    }
}

/// 分段行。
#[component]
pub fn SegmentedRow(
    #[prop(into)] label: String,
    options: Vec<SegmentedOption>,
    value: Signal<String>,
    on_change: Callback<String>,
) -> impl IntoView {
    let row_label = label.clone();
    view! {
        <Row label=row_label wide=true>
            <div class="if-seg">
                <Segmented options=options value=value label=label on_change=on_change />
            </div>
        </Row>
    }
}

/// `MaybeProp<bool>` 的推导式写法。
///
/// 闭包不能直接进 `MaybeProp`（它只认 `Signal` / `Option` / 具体值），
/// 而 `Signal::derive(...).into()` 又太长，检查器里几乎每一行都要用。
pub fn flag(f: impl Fn() -> bool + Send + Sync + 'static) -> MaybeProp<bool> {
    Signal::derive(f).into()
}

pub fn seg_options(items: &[(&str, &str)]) -> Vec<SegmentedOption> {
    items
        .iter()
        .map(|(value, label)| SegmentedOption::new(*value, *label))
        .collect()
}

pub fn select_options(items: &[(&str, &str)]) -> Vec<SelectOption> {
    items
        .iter()
        .map(|(value, label)| SelectOption::new(*value, *label))
        .collect()
}

/// 满宽按钮：`Button` 本身不提供 block 变体，宽度交给外层容器。
#[component]
pub fn BlockButton(
    #[prop(into)] label: String,
    #[prop(optional)] icon: Option<IconName>,
    #[prop(default = ButtonVariant::Secondary)] variant: ButtonVariant,
    #[prop(default = Size::Md)] size: Size,
    #[prop(into, optional)] disabled: MaybeProp<bool>,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="if-block">
            {match icon {
                Some(name) => view! {
                    <Button label=label icon=name variant=variant size=size disabled=disabled on_click=on_click />
                }.into_any(),
                None => view! {
                    <Button label=label variant=variant size=size disabled=disabled on_click=on_click />
                }.into_any(),
            }}
        </div>
    }
}

/// 一行两枚等宽按钮。
#[component]
pub fn ButtonPair(
    left_label: String,
    right_label: String,
    #[prop(default = ButtonVariant::Secondary)] left_variant: ButtonVariant,
    #[prop(default = ButtonVariant::Secondary)] right_variant: ButtonVariant,
    #[prop(into, optional)] left_disabled: MaybeProp<bool>,
    #[prop(into, optional)] right_disabled: MaybeProp<bool>,
    on_left: Callback<()>,
    on_right: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="if-btn-pair">
            <Button label=left_label variant=left_variant size=Size::Sm disabled=left_disabled on_click=on_left />
            <Button label=right_label variant=right_variant size=Size::Sm disabled=right_disabled on_click=on_right />
        </div>
    }
}
