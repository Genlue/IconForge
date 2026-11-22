//! 右侧检查器：样式 / 原图 / 导出三视图。
//!
//! 为什么分三页而不是一条长滚动：参数一共四十多项，一页到底的话
//! 「改个描边」要滚过画质增强和原图处理。分页之后每页都只装一类事，
//! 切页的成本比滚动低得多，而且每页都能从顶部开始读。

use glass_design::prelude::*;
use leptos::prelude::*;

use crate::components::fields::*;
use crate::state::App;
use crate::types::*;

#[component]
pub fn InspectorPanel() -> impl IntoView {
    let app = expect_context::<App>();
    let tab = app.inspector_tab;

    view! {
        <aside class="if-inspector c-inspector">
            <div class="c-inspector__head">
                <div class="c-inspector__tabs">
                    <Segmented
                        options=seg_options(&[("style", "样式"), ("source", "原图"), ("export", "导出")])
                        value=Signal::derive(move || tab.get().as_str().to_string())
                        label="检查器视图"
                        on_change=Callback::new(move |value: String| {
                            tab.set(InspectorTab::from_str(&value))
                        })
                    />
                </div>
            </div>
            <div class="c-inspector__body">
                {move || match tab.get() {
                    InspectorTab::Style => view! { <StyleTab /> }.into_any(),
                    InspectorTab::Source => view! { <SourceTab /> }.into_any(),
                    InspectorTab::Export => view! { <ExportTab /> }.into_any(),
                }}
            </div>
        </aside>
    }
}

// ==================================================================== 样式页

#[component]
fn StyleTab() -> impl IntoView {
    let app = expect_context::<App>();
    let hq = move || app.hq_active();

    view! {
        <PresetSection />
        <Show
            when=hq
            fallback=move || view! { <BasicSections /> }
        >
            <HqSection />
        </Show>
        <BrushSection />
    }
}

#[component]
fn PresetSection() -> impl IntoView {
    let app = expect_context::<App>();
    let presets = move || app.all_presets();
    let multi = move || app.item_count() > 1;

    view! {
        <Section title="预设" open=true>
            <div class="if-presets">
                <For each=presets key=|preset| preset.id.clone() let:preset>
                    <PresetRow preset=preset />
                </For>
            </div>
            <BlockButton
                label="保存当前设置为预设"
                on_click=Callback::new(move |_| app.save_current_as_preset())
            />
            <BlockButton
                label="应用当前配置到全部图标"
                disabled=flag(move || !multi())
                on_click=Callback::new(move |_| app.apply_config_to_all())
            />
            <Hint>"参数只作用于当前图标；要统一多个图标时用上面的按钮。"</Hint>
        </Section>
    }
}

#[component]
fn PresetRow(preset: Preset) -> impl IntoView {
    let app = expect_context::<App>();
    let id = preset.id.clone();
    let name = preset.name.clone();
    let description = preset.description.clone();
    let custom = preset.custom;
    let editing = RwSignal::new(false);
    let draft = RwSignal::new(name.clone());

    let is_active = {
        let id = id.clone();
        move || {
            app.current_config()
                .and_then(|config| config.active_preset)
                .as_deref()
                == Some(id.as_str())
        }
    };

    // 外层视图闭包必须是 Fn（Leptos 会多次调用），所以每次重绘都在内部
    // 重新 clone 一份再交给内层 move 闭包 —— 直接吃掉外层捕获的 String
    // 会把整个视图退化成 FnOnce，之后就不再更新了。
    let id_row = id;
    let name_row = name;
    let description_row = description;

    view! {
        <div class=move || {
            if is_active() { "if-preset if-preset--active" } else { "if-preset" }
        }>
            {move || {
                if editing.get() {
                    let id_enter = id_row.clone();
                    let id_button = id_row.clone();
                    view! {
                        <div class="if-preset__edit">
                            <Field
                                value=draft
                                label="预设名称"
                                on_input=Callback::new(move |value: String| draft.set(value))
                                on_enter=Callback::new(move |value: String| {
                                    app.rename_preset(id_enter.clone(), value);
                                    editing.set(false);
                                })
                            />
                            <Button
                                label="确定"
                                size=Size::Sm
                                on_click=Callback::new(move |_| {
                                    app.rename_preset(id_button.clone(), draft.get_untracked());
                                    editing.set(false);
                                })
                            />
                        </div>
                    }
                    .into_any()
                } else {
                    let id_click = id_row.clone();
                    let id_rename = id_row.clone();
                    let id_delete = id_row.clone();
                    let name = name_row.clone();
                    let description = description_row.clone();
                    view! {
                        <button
                            type="button"
                            class="if-preset__main"
                            title=description
                            on:click=move |_| app.apply_preset(id_click.clone())
                        >
                            <span class="if-preset__name">{name}</span>
                            {custom.then(|| view! { <span class="if-preset__badge">"自定"</span> })}
                        </button>
                        {custom.then(|| view! {
                            <IconButton
                                icon=IconName::SQUARE_PEN
                                label="重命名预设"
                                size=Size::Sm
                                variant=IconButtonVariant::Plain
                                on_click=Callback::new(move |_| {
                                    let _ = &id_rename;
                                    editing.set(true);
                                })
                            />
                        })}
                        {custom.then(|| view! {
                            <IconButton
                                icon=IconName::TRASH
                                label="删除预设"
                                size=Size::Sm
                                variant=IconButtonVariant::Plain
                                on_click=Callback::new(move |_| app.delete_preset(id_delete.clone()))
                            />
                        })}
                    }
                    .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn BasicSections() -> impl IntoView {
    let app = expect_context::<App>();

    // ---- 前景
    let fit = app.render_value(|render| fit_value(render.foreground_fit).to_string());
    let scale = app.render_value(|render| render.foreground_scale_percent as f64);
    let opacity = app.render_value(|render| render.foreground_opacity_percent as f64);
    let offset_x = app.render_value(|render| render.foreground_offset_x as f64);
    let offset_y = app.render_value(|render| render.foreground_offset_y as f64);
    let rotation = app.render_value(|render| render.foreground_rotation_degrees as f64);

    // ---- 形状
    let shape = app.render_value(|render| shape_value(render.shape).to_string());
    let inset = app.render_value(|render| render.canvas_inset as f64);
    let corner = app.render_value(|render| render.corner_radius as f64);
    let exponent = app.render_value(|render| render.squircle_exponent as f64);
    let rounded = app.render_value(|render| render.shape == IconShape::RoundedRectangle);
    let squircle = app.render_value(|render| render.shape == IconShape::Squircle);

    // ---- 背板
    let backplate = app.render_value(|render| backplate_value(render.backplate_type).to_string());
    let backplate_solid = app.render_value(|render| render.backplate_type == BackplateType::Solid);
    let backplate_gradient =
        app.render_value(|render| render.backplate_type == BackplateType::Gradient);
    let backplate_color = app.render_value(|render| render.backplate_color.clone());
    let gradient_start = app.render_value(|render| render.gradient_start_color.clone());
    let gradient_end = app.render_value(|render| render.gradient_end_color.clone());
    let gradient_angle = app.render_value(|render| render.gradient_angle_degrees as f64);

    // ---- 阴影
    let shadow_enabled = app.render_value(|render| render.outer_shadow.enabled);
    let shadow_x = app.render_value(|render| render.outer_shadow.offset_x as f64);
    let shadow_y = app.render_value(|render| render.outer_shadow.offset_y as f64);
    let shadow_blur = app.render_value(|render| render.outer_shadow.blur_radius as f64);
    let shadow_spread = app.render_value(|render| render.outer_shadow.spread as f64);
    let shadow_color = app.render_value(|render| render.outer_shadow.color.clone());

    // ---- 描边
    let stroke_width = app.render_value(|render| render.stroke.width as f64);
    let stroke_color = app.render_value(|render| render.stroke.color.clone());
    let stroke_visible = app.render_value(|render| render.stroke.width > 0.0);

    // ---- 光泽
    let gloss_enabled = app.render_value(|render| render.gloss.enabled);
    let gloss_angle = app.render_value(|render| render.gloss.angle_degrees as f64);
    let gloss_width = app.render_value(|render| render.gloss.width as f64);
    let gloss_strength = app.render_value(|render| render.gloss.strength as f64);
    let gloss_top = app.render_value(|render| render.gloss.top_strength as f64);
    let gloss_bottom = app.render_value(|render| render.gloss.bottom_strength as f64);
    let gloss_top_decay = app.render_value(|render| render.gloss.top_decay as f64);
    let gloss_bottom_decay = app.render_value(|render| render.gloss.bottom_decay as f64);
    let gloss_light = app.render_value(|render| render.gloss.light_color.clone());

    view! {
        <Section title="前景" open=true>
            <SegmentedRow
                label="适应方式"
                options=seg_options(&[("Contain", "适应"), ("Cover", "填满")])
                value=fit
                on_change=Callback::new(move |value: String| {
                    app.update_render(|render| {
                        render.foreground_fit = if value == "Cover" {
                            ForegroundFit::Cover
                        } else {
                            ForegroundFit::Contain
                        }
                    })
                })
            />
            <RangeField
                label="缩放"
                value=scale
                min=10.0
                max=300.0
                unit="%"
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.foreground_scale_percent = value as f32)
                })
            />
            <RangeField
                label="源图透明度"
                value=opacity
                min=0.0
                max=100.0
                unit="%"
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.foreground_opacity_percent = value as f32)
                })
            />
            <Hint>"以源图的非透明内容为基准；「填满」会铺满形状并裁切，超过 100% 可继续放大。"</Hint>
            <RangeField
                label="偏移 X"
                value=offset_x
                min=-128.0
                max=128.0
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.foreground_offset_x = value as f32)
                })
            />
            <RangeField
                label="偏移 Y"
                value=offset_y
                min=-128.0
                max=128.0
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.foreground_offset_y = value as f32)
                })
            />
            <RangeField
                label="旋转"
                value=rotation
                min=-180.0
                max=180.0
                unit="°"
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.foreground_rotation_degrees = value as f32)
                })
            />
        </Section>

        <Section title="形状">
            <SegmentedRow
                label="外形"
                options=seg_options(&[
                    ("Rectangle", "矩形"),
                    ("RoundedRectangle", "圆角"),
                    ("Squircle", "超椭圆"),
                ])
                value=shape
                on_change=Callback::new(move |value: String| {
                    app.update_render(|render| {
                        render.shape = match value.as_str() {
                            "Rectangle" => IconShape::Rectangle,
                            "Squircle" => IconShape::Squircle,
                            _ => IconShape::RoundedRectangle,
                        }
                    })
                })
            />
            <RangeField
                label="画布边距"
                value=inset
                min=0.0
                max=112.0
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.canvas_inset = value as f32)
                })
            />
            <Hint>"形状与描边离 ICO 画布边缘的距离。设 0 撑满画布；阴影较大时应保留边距。"</Hint>
            <div class=move || {
                if rounded.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField
                    label="圆角半径"
                    value=corner
                    min=0.0
                    max=128.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.corner_radius = value as f32)
                    })
                />
            </div>
            <div class=move || {
                if squircle.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField
                    label="超椭圆指数"
                    value=exponent
                    min=2.0
                    max=8.0
                    step=0.1
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.squircle_exponent = value as f32)
                    })
                />
            </div>
        </Section>

        <Section title="背板">
            <SegmentedRow
                label="类型"
                options=seg_options(&[("None", "无"), ("Solid", "纯色"), ("Gradient", "渐变")])
                value=backplate
                on_change=Callback::new(move |value: String| {
                    app.update_render(|render| {
                        render.backplate_type = match value.as_str() {
                            "None" => BackplateType::None,
                            "Gradient" => BackplateType::Gradient,
                            _ => BackplateType::Solid,
                        }
                    })
                })
            />
            <Show when=move || backplate_solid.get()>
                <ColorField
                    label="背板颜色"
                    value=backplate_color
                    pick=ColorTarget::Backplate
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.backplate_color = value)
                    })
                />
            </Show>
            <Show when=move || backplate_gradient.get()>
                <ColorField
                    label="渐变起点"
                    value=gradient_start
                    pick=ColorTarget::GradientStart
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.gradient_start_color = value)
                    })
                />
                <ColorField
                    label="渐变终点"
                    value=gradient_end
                    pick=ColorTarget::GradientEnd
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.gradient_end_color = value)
                    })
                />
                <RangeField
                    label="渐变角度"
                    value=gradient_angle
                    min=0.0
                    max=360.0
                    unit="°"
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gradient_angle_degrees = value as f32)
                    })
                />
            </Show>
        </Section>

        <Section title="外阴影">
            <ToggleRow
                label="启用阴影"
                checked=shadow_enabled
                on_change=Callback::new(move |on: bool| {
                    app.update_render(|render| render.outer_shadow.enabled = on)
                })
            />
            <Hint>"阴影仍受 256×256 画布限制；被裁切时在「形状」里加大画布边距。"</Hint>
            <div class=move || {
                if shadow_enabled.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField
                    label="偏移 X"
                    value=shadow_x
                    min=-64.0
                    max=64.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.outer_shadow.offset_x = value as f32)
                    })
                />
                <RangeField
                    label="偏移 Y"
                    value=shadow_y
                    min=-64.0
                    max=64.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.outer_shadow.offset_y = value as f32)
                    })
                />
                <RangeField
                    label="模糊"
                    value=shadow_blur
                    min=0.0
                    max=64.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.outer_shadow.blur_radius = value as f32)
                    })
                />
                <RangeField
                    label="扩散"
                    value=shadow_spread
                    min=0.0
                    max=32.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.outer_shadow.spread = value as f32)
                    })
                />
                <ColorField
                    label="阴影颜色"
                    value=shadow_color
                    pick=ColorTarget::Shadow
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.outer_shadow.color = value)
                    })
                />
            </div>
        </Section>

        <Section title="描边">
            <RangeField
                label="宽度"
                value=stroke_width
                min=0.0
                max=32.0
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.stroke.width = value as f32)
                })
            />
            <div class=move || {
                if stroke_visible.get() { "" } else { "if-group if-group--off" }
            }>
                <ColorField
                    label="描边颜色"
                    value=stroke_color
                    pick=ColorTarget::Stroke
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.stroke.color = value)
                    })
                />
            </div>
        </Section>

        <Section title="边缘光泽">
            <ToggleRow
                label="启用光泽"
                checked=gloss_enabled
                on_change=Callback::new(move |on: bool| {
                    app.update_render(|render| render.gloss.enabled = on)
                })
            />
            <div class=move || {
                if gloss_enabled.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField
                    label="光源角度"
                    value=gloss_angle
                    min=0.0
                    max=360.0
                    unit="°"
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.angle_degrees = value as f32)
                    })
                />
                <RangeField
                    label="边缘衰减"
                    value=gloss_width
                    min=0.5
                    max=16.0
                    step=0.5
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.width = value as f32)
                    })
                />
                <RangeField
                    label="光泽强度"
                    value=gloss_strength
                    min=0.0
                    max=1.0
                    step=0.05
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.strength = value as f32)
                    })
                />
                <RangeField
                    label="受光侧亮度"
                    value=gloss_top
                    min=0.0
                    max=1.0
                    step=0.01
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.top_strength = value as f32)
                    })
                />
                <RangeField
                    label="背光侧亮度"
                    value=gloss_bottom
                    min=0.0
                    max=1.0
                    step=0.01
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.bottom_strength = value as f32)
                    })
                />
                <RangeField
                    label="受光侧延伸"
                    value=gloss_top_decay
                    min=2.0
                    max=128.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.top_decay = value as f32)
                    })
                />
                <RangeField
                    label="背光侧延伸"
                    value=gloss_bottom_decay
                    min=2.0
                    max=128.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.gloss.bottom_decay = value as f32)
                    })
                />
                <ColorField
                    label="高光颜色"
                    value=gloss_light
                    pick=ColorTarget::GlossLight
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|render| render.gloss.light_color = value)
                    })
                />
            </div>
            <Hint>"光泽沿外形边缘向内衰减，并沿可旋转光轴叠加受光/背光两道柔光。0° 表示光从正上方照下。"</Hint>
        </Section>
    }
}

// ============================================================== 高质量渲染

#[component]
fn HqSection() -> impl IntoView {
    let app = expect_context::<App>();

    let thresh = app.render_value(|r| r.hq_render.thresh as f64);
    let icon_ratio = app.render_value(|r| r.hq_render.icon_ratio as f64);
    let bg = app.render_value(|r| r.hq_render.bg as f64);
    let light_mix = app.render_value(|r| r.hq_render.light_mix as f64);
    let dark_mix = app.render_value(|r| r.hq_render.dark_mix as f64);
    let gloss = app.render_value(|r| r.hq_render.gloss as f64);
    let icon_light = app.render_value(|r| r.hq_render.icon_light as f64);
    let corner = app.render_value(|r| r.hq_render.corner as f64);
    let offset_x = app.render_value(|r| r.hq_render.offset_x as f64);
    let offset_y = app.render_value(|r| r.hq_render.offset_y as f64);
    let custom_bg = app.render_value(|r| r.hq_render.custom_bg_enabled);
    let custom_bg_color = app.render_value(|r| r.hq_render.custom_bg_color.clone());
    let shape = app.render_value(|r| hq_shape_value(r.hq_render.shape).to_string());
    let shadow_opacity = app.render_value(|r| r.hq_render.shadow_opacity as f64);
    let shadow_blur = app.render_value(|r| r.hq_render.shadow_blur_factor as f64);
    let shadow_offset = app.render_value(|r| r.hq_render.shadow_offset_factor as f64);
    let shadow_fade = app.render_value(|r| r.hq_render.shadow_fade as f64);
    let shadow_mode = app.render_value(|r| shadow_mode_value(r.hq_render.shadow_mode).to_string());

    let gloss_enabled = app.render_value(|r| r.gloss.enabled);
    let gloss_angle = app.render_value(|r| r.gloss.angle_degrees as f64);
    let gloss_width = app.render_value(|r| r.gloss.width as f64);
    let gloss_strength = app.render_value(|r| r.gloss.strength as f64);
    let gloss_top = app.render_value(|r| r.gloss.top_strength as f64);
    let gloss_bottom = app.render_value(|r| r.gloss.bottom_strength as f64);
    let gloss_top_decay = app.render_value(|r| r.gloss.top_decay as f64);
    let gloss_bottom_decay = app.render_value(|r| r.gloss.bottom_decay as f64);
    let gloss_light = app.render_value(|r| r.gloss.light_color.clone());

    view! {
        <Section title="高质量渲染参数" open=true>
            <Hint>"按图标颜色自动生成面板（还原 MyDockFinder 全局遮罩效果）。仅对「高质量渲染」预设生效，切到其他预设自动失效。"</Hint>
            <RangeField label="明暗分界阈值" value=thresh min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.thresh = v as f32)) />
            <RangeField label="图标占比" value=icon_ratio min=0.1 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.icon_ratio = v as f32)) />
            <RangeField label="径向光晕浓度" value=bg min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.bg = v as f32)) />
            <RangeField label="浅色提白比例" value=light_mix min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.light_mix = v as f32)) />
            <RangeField label="深色加深系数" value=dark_mix min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.dark_mix = v as f32)) />
            <RangeField label="顶部光泽强度" value=gloss min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.gloss = v as f32)) />
            <RangeField label="图标提亮" value=icon_light min=0.0 max=0.5 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.icon_light = v as f32)) />
            <RangeField label="圆角比例" value=corner min=0.05 max=0.5 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.corner = v as f32)) />
            <RangeField label="位置 X" value=offset_x min=-64.0 max=64.0
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.offset_x = v as f32)) />
            <RangeField label="位置 Y" value=offset_y min=-64.0 max=64.0
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.offset_y = v as f32)) />
            <Hint>"在 256 画布内平移图标本体（含阴影与光晕），用于调整图标在面板内的位置。"</Hint>

            <ToggleRow label="自定义基底颜色" checked=custom_bg
                on_change=Callback::new(move |on: bool| app.update_render(|r| r.hq_render.custom_bg_enabled = on)) />
            <div class=move || {
                if custom_bg.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <ColorField
                    label="基底颜色"
                    value=custom_bg_color
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|r| r.hq_render.custom_bg_color = value)
                    })
                />
                <Hint>"固定用该颜色作为面板基底，径向光晕仍按图标配色。"</Hint>
            </div>

            <SegmentedRow
                label="面板形状"
                options=seg_options(&[("Rect", "圆角矩形"), ("Squircle", "超椭圆"), ("Circle", "圆形")])
                value=shape
                on_change=Callback::new(move |value: String| {
                    app.update_render(|r| {
                        r.hq_render.shape = match value.as_str() {
                            "Squircle" => HqPanelShape::Squircle,
                            "Circle" => HqPanelShape::Circle,
                            _ => HqPanelShape::Rect,
                        }
                    })
                })
            />
            <RangeField label="阴影浓度" value=shadow_opacity min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.shadow_opacity = v as f32)) />
            <RangeField label="阴影模糊（×画布）" value=shadow_blur min=0.0 max=0.1 step=0.001
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.shadow_blur_factor = v as f32)) />
            <RangeField label="阴影偏移（×画布）" value=shadow_offset min=0.0 max=0.1 step=0.001
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.shadow_offset_factor = v as f32)) />
            <RangeField label="阴影尾部衰减" value=shadow_fade min=0.0 max=1.0 step=0.01
                on_change=Callback::new(move |v: f64| app.update_render(|r| r.hq_render.shadow_fade = v as f32)) />
            <SegmentedRow
                label="阴影轮廓来源"
                options=seg_options(&[("icon", "图标轮廓"), ("badge", "面板整体")])
                value=shadow_mode
                on_change=Callback::new(move |value: String| {
                    app.update_render(|r| {
                        r.hq_render.shadow_mode = if value == "badge" {
                            HqShadowMode::Badge
                        } else {
                            HqShadowMode::Icon
                        }
                    })
                })
            />
        </Section>

        <Section title="边缘光泽">
            <ToggleRow label="启用光泽" checked=gloss_enabled
                on_change=Callback::new(move |on: bool| app.update_render(|r| r.gloss.enabled = on)) />
            <div class=move || {
                if gloss_enabled.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField label="光源角度" value=gloss_angle min=0.0 max=360.0 unit="°"
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.angle_degrees = v as f32)) />
                <RangeField label="边缘衰减" value=gloss_width min=0.5 max=16.0 step=0.5
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.width = v as f32)) />
                <RangeField label="光泽强度" value=gloss_strength min=0.0 max=1.0 step=0.05
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.strength = v as f32)) />
                <RangeField label="受光侧亮度" value=gloss_top min=0.0 max=1.0 step=0.01
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.top_strength = v as f32)) />
                <RangeField label="背光侧亮度" value=gloss_bottom min=0.0 max=1.0 step=0.01
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.bottom_strength = v as f32)) />
                <RangeField label="受光侧延伸" value=gloss_top_decay min=2.0 max=128.0
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.top_decay = v as f32)) />
                <RangeField label="背光侧延伸" value=gloss_bottom_decay min=2.0 max=128.0
                    on_change=Callback::new(move |v: f64| app.update_render(|r| r.gloss.bottom_decay = v as f32)) />
                <ColorField label="高光颜色" value=gloss_light pick=ColorTarget::GlossLight
                    on_change=Callback::new(move |value: String| {
                        app.update_render(|r| r.gloss.light_color = value)
                    }) />
            </div>
        </Section>
    }
}

// ==================================================================== 画笔

#[component]
fn BrushSection() -> impl IntoView {
    let app = expect_context::<App>();
    let color = app.config_value(|config| config.brush_color.clone());
    let size = app.config_value(|config| config.brush_size as f64);
    let clip = app.config_value(|config| config.brush_clip_to_mask);
    let count = app.config_value(|config| config.brush_strokes.len());

    view! {
        <Section title="画笔">
            <div class="if-row-note">
                <span>{move || format!("已绘制 {} 笔", count.get())}</span>
            </div>
            {move || {
                let tool = app.tool.get();
                view! {
                    <div class="if-btn-pair">
                        <Button
                            label="画笔"
                            size=Size::Sm
                            variant=if tool == Tool::Brush { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                            on_click=Callback::new(move |_| {
                                app.update_config(|config| config.brush_mode = BrushMode::Paint);
                                app.set_tool(Tool::Brush);
                            })
                        />
                        <Button
                            label="涂抹擦除"
                            size=Size::Sm
                            variant=if tool == Tool::Eraser { ButtonVariant::Primary } else { ButtonVariant::Secondary }
                            on_click=Callback::new(move |_| {
                                app.update_config(|config| config.brush_mode = BrushMode::Erase);
                                app.set_tool(Tool::Eraser);
                            })
                        />
                    </div>
                }
            }}
            <ColorField
                label="画笔颜色"
                value=color
                pick=ColorTarget::Brush
                on_change=Callback::new(move |value: String| {
                    app.update_config(|config| config.brush_color = value)
                })
            />
            <RangeField
                label="画笔大小"
                value=size
                min=1.0
                max=64.0
                on_change=Callback::new(move |value: f64| {
                    app.update_config(|config| config.brush_size = value as f32)
                })
            />
            <ToggleRow
                label="裁切到图标"
                checked=clip
                on_change=Callback::new(move |on: bool| {
                    app.update_config(|config| config.brush_clip_to_mask = on)
                })
            />
            <ButtonPair
                left_label="撤销一笔".to_string()
                right_label="清空画笔".to_string()
                right_variant=ButtonVariant::Danger
                left_disabled=flag(move || count.get() == 0)
                right_disabled=flag(move || count.get() == 0)
                on_left=Callback::new(move |_| app.undo_brush())
                on_right=Callback::new(move |_| app.clear_brush())
            />
            <Hint>"笔迹属于当前图标，会进入预览、PNG 和最终 ICO。"</Hint>
        </Section>
    }
}

// ==================================================================== 原图页

#[component]
fn SourceTab() -> impl IntoView {
    let app = expect_context::<App>();

    // 画质增强
    let upscale_enabled = app.upscale_value(|config| config.enabled);
    let scale = app.upscale_value(|config| config.scale.to_string());
    let denoise = app.upscale_value(|config| config.denoise.to_string());
    let model = app.upscale_value(|config| config.model.clone());

    // 原图处理
    let saturation = app.render_value(|render| render.source_saturation_percent as f64);
    let cutout_enabled = app.render_value(|render| render.auto_cutout.enabled);
    let cutout_tolerance = app.render_value(|render| render.auto_cutout.tolerance as f64);
    let cutout_feather = app.render_value(|render| render.auto_cutout.feather as f64);
    let wand_tolerance = app.config_value(|config| config.wand_tolerance as f64);
    let wand_count = app.config_value(|config| config.wand_strokes.len());
    let eraser_size = app.config_value(|config| config.eraser_size as f64);
    let eraser_hardness = app.config_value(|config| config.eraser_hardness as f64);
    let eraser_count = app.config_value(|config| config.eraser_strokes.len());
    let wand_pending = move || app.wand_selection.get().is_some();

    view! {
        <Section title="画质增强" open=true>
            <ToggleRow
                label="启用 Real-CUGAN 放大"
                checked=upscale_enabled
                on_change=Callback::new(move |on: bool| {
                    app.update_upscale(|config| config.enabled = on)
                })
            />
            <Hint>"先增强导入源图，再用于最终预览和所有 ICO 档位导出。首次处理可能需要几秒。"</Hint>
            <div class=move || {
                if upscale_enabled.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <SelectRow
                    label="缩放倍数"
                    options=select_options(&[("2", "2x"), ("3", "3x"), ("4", "4x")])
                    value=scale
                    on_change=Callback::new(move |value: String| {
                        let scale: u32 = value.parse().unwrap_or(2);
                        app.update_upscale(|config| {
                            config.scale = scale;
                            // Pro 模型不支持 4x；3x 以上中/低降噪会越界，抬到「极高」。
                            if config.model == "pro" && scale == 4 {
                                config.scale = 2;
                            }
                            if config.model == "nose" {
                                config.scale = 2;
                                config.denoise = -1;
                            } else if scale >= 3 && (config.denoise == 1 || config.denoise == 2) {
                                config.denoise = 3;
                            }
                        })
                    })
                />
                <SelectRow
                    label="降噪级别"
                    options=select_options(&[
                        ("-1", "无"),
                        ("0", "低"),
                        ("1", "中"),
                        ("2", "高"),
                        ("3", "极高"),
                    ])
                    value=denoise
                    on_change=Callback::new(move |value: String| {
                        let denoise: i32 = value.parse().unwrap_or(-1);
                        app.update_upscale(|config| config.denoise = denoise)
                    })
                />
                <SelectRow
                    label="模型"
                    options=select_options(&[
                        ("se", "轻量 (SE)"),
                        ("pro", "高质量 (Pro)"),
                        ("nose", "No-SE（仅 2x 无降噪）"),
                    ])
                    value=model
                    width=180
                    on_change=Callback::new(move |value: String| {
                        app.update_upscale(|config| {
                            config.model = value;
                            if config.model == "nose" {
                                config.scale = 2;
                                config.denoise = -1;
                            } else if config.model == "pro" && config.scale == 4 {
                                config.scale = 2;
                                if config.denoise == 1 || config.denoise == 2 {
                                    config.denoise = 3;
                                }
                            }
                        })
                    })
                />
            </div>
        </Section>

        <Section title="原图处理" open=true>
            <Hint>"以下操作直接作用于导入的原图。"</Hint>
            <RangeField
                label="饱和度"
                value=saturation
                min=0.0
                max=100.0
                unit="%"
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.source_saturation_percent = value as f32)
                })
            />
            <Hint>"100 保持原图色彩，0 完全去色。"</Hint>
            <ToggleRow
                label="自动抠图"
                checked=cutout_enabled
                on_change=Callback::new(move |on: bool| {
                    app.update_render(|render| render.auto_cutout.enabled = on)
                })
            />
            <div class=move || {
                if cutout_enabled.get() { "if-group" } else { "if-group if-group--off" }
            }>
                <RangeField
                    label="背景容差"
                    value=cutout_tolerance
                    min=0.0
                    max=100.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.auto_cutout.tolerance = value as f32)
                    })
                />
                <RangeField
                    label="边缘羽化"
                    value=cutout_feather
                    min=0.0
                    max=32.0
                    on_change=Callback::new(move |value: f64| {
                        app.update_render(|render| render.auto_cutout.feather = value as f32)
                    })
                />
            </div>
            <Hint>"从图像边界识别连通背景，适合白底、纯色底图标。"</Hint>
        </Section>

        <Section title="魔棒与橡皮擦">
            <BlockButton
                label="打开原图编辑"
                icon=IconName::SQUARE_PEN
                on_click=Callback::new(move |_| app.source_editor_open.set(true))
            />
            <Hint>"魔棒与橡皮擦都在原图编辑里操作：那里坐标与原图一一对应，不会因为缩放而漂移。"</Hint>
            <RangeField
                label="魔棒容差"
                value=wand_tolerance
                min=0.0
                max=100.0
                on_change=Callback::new(move |value: f64| {
                    app.update_config(|config| config.wand_tolerance = value as f32);
                    app.wand_selection.set(None);
                })
            />
            <ButtonPair
                left_label="撤销删除".to_string()
                right_label="清空删除".to_string()
                right_variant=ButtonVariant::Danger
                left_disabled=flag(move || wand_count.get() == 0)
                right_disabled=flag(move || wand_count.get() == 0)
                on_left=Callback::new(move |_| app.undo_wand())
                on_right=Callback::new(move |_| app.clear_wand())
            />
            <div class="if-row-note">
                <span>{move || format!("已删除 {} 处", wand_count.get())}</span>
                <Show when=wand_pending>
                    <Button
                        label="删除选中区域"
                        variant=ButtonVariant::Danger
                        size=Size::Sm
                        on_click=Callback::new(move |_| app.delete_wand_selection())
                    />
                </Show>
            </div>
            <RangeField
                label="橡皮大小"
                value=eraser_size
                min=1.0
                max=64.0
                on_change=Callback::new(move |value: f64| {
                    app.update_config(|config| config.eraser_size = value as f32)
                })
            />
            <RangeField
                label="橡皮硬度"
                value=eraser_hardness
                min=0.0
                max=100.0
                on_change=Callback::new(move |value: f64| {
                    app.update_config(|config| config.eraser_hardness = value as f32)
                })
            />
            <ButtonPair
                left_label="撤销擦除".to_string()
                right_label="清空擦除".to_string()
                right_variant=ButtonVariant::Danger
                left_disabled=flag(move || eraser_count.get() == 0)
                right_disabled=flag(move || eraser_count.get() == 0)
                on_left=Callback::new(move |_| app.undo_eraser())
                on_right=Callback::new(move |_| app.clear_eraser())
            />
        </Section>
    }
}

// ==================================================================== 导出页

#[component]
fn ExportTab() -> impl IntoView {
    let app = expect_context::<App>();
    let content_scale = app.render_value(|render| render.content_scale_percent as f64);
    let busy = move || app.exporting.get();
    let count = move || app.item_count();
    let has_lnk = move || app.has_lnk();
    let has_ico = move || app.has_ico_source();

    view! {
        <Section title="导出" open=true>
            <RangeField
                label="图标整体缩放"
                value=content_scale
                min=0.0
                max=100.0
                unit="%"
                on_change=Callback::new(move |value: f64| {
                    app.update_render(|render| render.content_scale_percent = value as f32)
                })
            />
            <Hint>"保持 256×256 总画布，把最终图标整体缩小靠中（默认 100%）。"</Hint>

            <BlockButton
                label="导出当前 ICO"
                variant=ButtonVariant::Primary
                disabled=flag(move || count() == 0 || busy())
                on_click=Callback::new(move |_| app.export_selected_ico())
            />
            <BlockButton
                label="全部导出 ICO"
                disabled=flag(move || count() == 0 || busy())
                on_click=Callback::new(move |_| app.export_all_ico())
            />
            <ButtonPair
                left_label="当前 PNG".to_string()
                right_label="全部 PNG".to_string()
                left_disabled=flag(move || count() == 0 || busy())
                right_disabled=flag(move || count() == 0 || busy())
                on_left=Callback::new(move |_| app.export_selected_png())
                on_right=Callback::new(move |_| app.export_all_png())
            />
            <BlockButton
                label="批量提取 ICO 最大帧为 PNG"
                size=Size::Sm
                disabled=flag(move || !has_ico() || busy())
                on_click=Callback::new(move |_| app.extract_ico_png())
            />
            <BlockButton
                label="应用到 LNK 快捷方式"
                size=Size::Sm
                disabled=flag(move || !has_lnk() || busy())
                on_click=Callback::new(move |_| app.apply_to_shortcuts())
            />
            <Hint>"应用到快捷方式会改写目标 LNK 的图标并保留备份；只对列表里的 LNK 生效。"</Hint>
        </Section>

        <Section title="快捷键" open=true>
            <div class="if-keys">
                <div class="if-key"><kbd>"Ctrl"</kbd>" + "<kbd>"O"</kbd>"  导入文件"</div>
                <div class="if-key"><kbd>"Ctrl"</kbd>" + "<kbd>"E"</kbd>"  导出当前 ICO"</div>
                <div class="if-key"><kbd>"Del"</kbd>"  删除魔棒选区"</div>
                <div class="if-key"><kbd>"Esc"</kbd>"  关闭原图编辑 / 对话框"</div>
            </div>
        </Section>
    }
}

// ==================================================================== 枚举映射

fn shape_value(shape: IconShape) -> &'static str {
    match shape {
        IconShape::Rectangle => "Rectangle",
        IconShape::RoundedRectangle => "RoundedRectangle",
        IconShape::Squircle => "Squircle",
    }
}

fn fit_value(fit: ForegroundFit) -> &'static str {
    match fit {
        ForegroundFit::Contain => "Contain",
        ForegroundFit::Cover => "Cover",
    }
}

fn backplate_value(value: BackplateType) -> &'static str {
    match value {
        BackplateType::None => "None",
        BackplateType::Solid => "Solid",
        BackplateType::Gradient => "Gradient",
    }
}

fn hq_shape_value(value: HqPanelShape) -> &'static str {
    match value {
        HqPanelShape::Rect => "Rect",
        HqPanelShape::Squircle => "Squircle",
        HqPanelShape::Circle => "Circle",
    }
}

fn shadow_mode_value(value: HqShadowMode) -> &'static str {
    match value {
        HqShadowMode::Icon => "icon",
        HqShadowMode::Badge => "badge",
    }
}
