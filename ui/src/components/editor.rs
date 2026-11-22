//! 原图编辑层。
//!
//! 魔棒与橡皮擦必须作用在**原图坐标**上：在 256 预览里点选，换算回原图会随缩放
//! 漂移。所以这里单独开一层，显示宿主处理过的原图（已放大、已应用历史修改），
//! 所有交互都在这一层完成。

use glass_design::prelude::*;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::api;
use crate::canvas::{draw_ants, draw_scene, event_canvas, point_in_space};
use crate::components::fields::{flag, seg_options, RangeField};
use crate::state::App;
use crate::types::*;
use crate::util;

#[component]
pub fn SourceEditor() -> impl IntoView {
    let app = expect_context::<App>();
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let ants_ref = NodeRef::<html::Canvas>::new();
    let image = StoredValue::new_local(Option::<web_sys::HtmlImageElement>::None);
    let points = StoredValue::new_local(Vec::<BrushPoint>::new());
    let generation = StoredValue::new(0u64);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let dimensions = RwSignal::new(None::<(u32, u32)>);
    let selecting = RwSignal::new(false);

    // 原图：放大 / 魔棒 / 橡皮擦任一变化都要重新取一次。
    Effect::new(move |_| {
        let item = app.current_item();
        let config = app.current_config();
        let (Some(item), Some(config)) = (item, config) else {
            return;
        };
        let token = generation.get_value() + 1;
        generation.set_value(token);
        loading.set(true);
        error.set(None);

        let request = SourceImageRequest {
            source_path: item.source_path,
            upscale_config: config.upscale.clone(),
            wand_strokes: config.wand_strokes.clone(),
            eraser_strokes: config.eraser_strokes.clone(),
        };

        spawn_local(async move {
            match api::render_source_image(request).await {
                Ok(response) => {
                    if generation.get_value() != token {
                        return;
                    }
                    dimensions.set(Some((response.width, response.height)));
                    let data_url = format!("data:image/png;base64,{}", response.png_base64);
                    if let Some(loaded) = util::load_image(&data_url).await {
                        if generation.get_value() != token {
                            return;
                        }
                        image.set_value(Some(loaded.clone()));
                        draw_scene(canvas_ref.get_untracked(), Some(&loaded), None);
                    }
                }
                Err(err) => {
                    if generation.get_value() == token {
                        error.set(Some(err.message));
                    }
                }
            }
            if generation.get_value() == token {
                loading.set(false);
            }
        });
    });

    // 待确认的魔棒选区。
    Effect::new(move |_| {
        let selection = app.wand_selection.get();
        let Some(selection) = selection else {
            crate::canvas::clear_canvas(ants_ref.get_untracked());
            return;
        };
        let overlay = ants_ref.get_untracked();
        spawn_local(async move {
            let mask = util::load_image(&format!(
                "data:image/png;base64,{}",
                selection.mask_png_base64
            ))
            .await;
            if let Some(mask) = mask {
                draw_ants(overlay, &mask);
            }
        });
    });

    // ------------------------------------------------------------ 指针交互

    let pointer_down = move |ev: web_sys::PointerEvent| {
        let Some(canvas) = event_canvas(&ev) else { return };
        let Some(config) = app.current_config() else { return };
        let (x, y) = point_in_space(&ev, &canvas);
        let tool = app.tool.get_untracked();

        if tool == Tool::MagicWand {
            let Some(item) = app.current_item() else { return };
            selecting.set(true);
            app.wand_selection.set(None);
            let request = WandSelectionRequest {
                source_path: item.source_path,
                upscale_config: config.upscale.clone(),
                point: WandPoint { x, y },
                tolerance: config.wand_tolerance,
                wand_strokes: config.wand_strokes.clone(),
                eraser_strokes: config.eraser_strokes.clone(),
            };
            spawn_local(async move {
                match api::compute_wand_selection(request).await {
                    Ok(response) => app.set_wand_selection(Some(WandSelection {
                        x,
                        y,
                        tolerance: config.wand_tolerance,
                        mask_png_base64: response.mask_png_base64,
                    })),
                    Err(err) => app.error.set(Some(err)),
                }
                selecting.set(false);
            });
            return;
        }

        if tool != Tool::SourceEraser {
            return;
        }
        let _ = canvas.set_pointer_capture(ev.pointer_id());
        points.set_value(vec![BrushPoint { x, y }]);
        let stroke = eraser_stroke(&config, &points.get_value());
        draw_scene(Some(canvas), image.get_value().as_ref(), Some(&stroke));
    };

    let pointer_move = move |ev: web_sys::PointerEvent| {
        if app.tool.get_untracked() != Tool::SourceEraser || points.with_value(Vec::is_empty) {
            return;
        }
        let Some(canvas) = event_canvas(&ev) else { return };
        let Some(config) = app.current_config() else { return };
        let (x, y) = point_in_space(&ev, &canvas);
        points.update_value(|list| list.push(BrushPoint { x, y }));
        let stroke = eraser_stroke(&config, &points.get_value());
        draw_scene(Some(canvas), image.get_value().as_ref(), Some(&stroke));
    };

    let pointer_up = move |ev: web_sys::PointerEvent| {
        if app.tool.get_untracked() != Tool::SourceEraser || points.with_value(Vec::is_empty) {
            return;
        }
        let Some(canvas) = event_canvas(&ev) else { return };
        let Some(config) = app.current_config() else { return };
        let (x, y) = point_in_space(&ev, &canvas);
        points.update_value(|list| {
            if list.len() == 1 {
                list.push(BrushPoint { x, y });
            }
        });
        let _ = canvas.release_pointer_capture(ev.pointer_id());
        let stroke = EraserStroke {
            points: points
                .get_value()
                .into_iter()
                .map(|point| WandPoint { x: point.x, y: point.y })
                .collect(),
            size: config.eraser_size,
            hardness: config.eraser_hardness,
        };
        points.set_value(Vec::new());
        app.add_eraser_stroke(stroke);
    };

    // ------------------------------------------------------------ 视图

    let tool_value = move || match app.tool.get() {
        Tool::MagicWand => "magic-wand".to_string(),
        Tool::SourceEraser => "source-eraser".to_string(),
        _ => "none".to_string(),
    };
    let cursor = move || match app.tool.get() {
        Tool::None => "default",
        _ => "crosshair",
    };
    let size_text = move || {
        dimensions
            .get()
            .map(|(w, h)| format!("{w}×{h}"))
            .unwrap_or_default()
    };
    // 画布显示尺寸在这里算死：CSS 的 aspect-ratio 遇上 max-width 会被压扁，
    // 而原图比例必须严格保持 —— 魔棒坐标是按显示框换算的。
    let size_style = move || {
        let (w, h) = dimensions.get().unwrap_or((1, 1));
        let ratio = w.max(1) as f64 / h.max(1) as f64;
        let max = 520.0;
        let (dw, dh) = if ratio >= 1.0 {
            (max, max / ratio)
        } else {
            (max * ratio, max)
        };
        format!("width:{:.0}px;height:{:.0}px", dw.max(1.0), dh.max(1.0))
    };
    let item_name = move || {
        app.current_item()
            .map(|item| item.display_name)
            .unwrap_or_default()
    };
    let eraser_count = move || {
        app.current_config()
            .map(|config| config.eraser_strokes.len())
            .unwrap_or(0)
    };
    let wand_count = move || {
        app.current_config()
            .map(|config| config.wand_strokes.len())
            .unwrap_or(0)
    };
    let wand_pending = move || app.wand_selection.get().is_some();
    let hint = move || {
        if let Some(message) = error.get() {
            return message;
        }
        match app.tool.get() {
            Tool::MagicWand => "点击选中相似区域，按 Delete 删除为透明；容差可在上方调整。".to_string(),
            Tool::SourceEraser => "涂抹即擦除原图（变为透明），大小/硬度可在上方调整。".to_string(),
            _ => "先选工具再在原图上修改。棋盘格表示透明，修改会进入预览、PNG 和最终 ICO。"
                .to_string(),
        }
    };
    let close = Callback::new(move |_: ()| {
        app.tool.set(Tool::None);
        app.wand_selection.set(None);
        app.source_editor_open.set(false);
    });

    let cutout = move || {
        app.current_config()
            .map(|config| config.render.auto_cutout)
            .unwrap_or(AutoCutoutConfig {
                enabled: false,
                tolerance: 20.0,
                feather: 8.0,
            })
    };
    let wand_tolerance = move || app.current_config().map(|c| c.wand_tolerance).unwrap_or(72.0);
    let eraser_size = move || app.current_config().map(|c| c.eraser_size).unwrap_or(16.0);
    let eraser_hardness = move || app.current_config().map(|c| c.eraser_hardness).unwrap_or(50.0);

    view! {
        <div class="if-editor">
            <div class="if-editor__bar">
                <div class="if-editor__title">
                    <Icon name=IconName::SQUARE_PEN size=15 />
                    <span class="if-editor__name">"原图编辑"</span>
                    <span class="if-editor__sub">{move || format!("{} · {}", item_name(), size_text())}</span>
                </div>

                <Segmented
                    options=seg_options(&[
                        ("none", "选择"),
                        ("magic-wand", "魔棒"),
                        ("source-eraser", "橡皮擦"),
                    ])
                    value=tool_value()
                    label="原图工具"
                    on_change=Callback::new(move |value: String| {
                        let tool = match value.as_str() {
                            "magic-wand" => Tool::MagicWand,
                            "source-eraser" => Tool::SourceEraser,
                            _ => Tool::None,
                        };
                        if app.tool.get_untracked() == tool {
                            app.tool.set(Tool::None);
                        } else {
                            app.tool.set(tool);
                        }
                    })
                />

                <Push />

                <Button
                    label="撤销擦除"
                    size=Size::Sm
                    disabled=flag(move || eraser_count() == 0)
                    on_click=Callback::new(move |_| app.undo_eraser())
                />
                <Button
                    label="撤销删除"
                    size=Size::Sm
                    disabled=flag(move || wand_count() == 0)
                    on_click=Callback::new(move |_| app.undo_wand())
                />
                <Show when=wand_pending>
                    <Button
                        label="删除选中"
                        variant=ButtonVariant::Danger
                        size=Size::Sm
                        disabled=flag(move || selecting.get())
                        on_click=Callback::new(move |_| app.delete_wand_selection())
                    />
                    <Button
                        label="取消选区"
                        size=Size::Sm
                        on_click=Callback::new(move |_| app.wand_selection.set(None))
                    />
                </Show>
                <Button
                    label="完成"
                    variant=ButtonVariant::Primary
                    size=Size::Sm
                    on_click=close
                />
            </div>

            <div class="if-editor__params">
                <div class="if-editor__param if-editor__param--toggle">
                    <span class="if-editor__param-label">"自动抠图"</span>
                    <Switch
                        checked=flag(move || cutout().enabled)
                        on_change=Callback::new(move |on: bool| {
                            app.update_render(|render| render.auto_cutout.enabled = on)
                        })
                    />
                </div>
                <div class=move || {
                    if cutout().enabled {
                        "if-editor__param"
                    } else {
                        "if-editor__param if-group--off"
                    }
                }>
                    <RangeField
                        label="背景容差"
                        value=Signal::derive(move || cutout().tolerance as f64)
                        min=0.0
                        max=100.0
                        on_change=Callback::new(move |value: f64| {
                            app.update_render(|render| render.auto_cutout.tolerance = value as f32)
                        })
                    />
                </div>
                <div class=move || {
                    if cutout().enabled {
                        "if-editor__param"
                    } else {
                        "if-editor__param if-group--off"
                    }
                }>
                    <RangeField
                        label="边缘羽化"
                        value=Signal::derive(move || cutout().feather as f64)
                        min=0.0
                        max=32.0
                        on_change=Callback::new(move |value: f64| {
                            app.update_render(|render| render.auto_cutout.feather = value as f32)
                        })
                    />
                </div>
                <div class="if-editor__param">
                    <RangeField
                        label="魔棒容差"
                        value=Signal::derive(move || wand_tolerance() as f64)
                        min=0.0
                        max=100.0
                        on_change=Callback::new(move |value: f64| {
                            app.update_config(|config| config.wand_tolerance = value as f32);
                            app.wand_selection.set(None);
                        })
                    />
                </div>
                <div class="if-editor__param">
                    <RangeField
                        label="橡皮大小"
                        value=Signal::derive(move || eraser_size() as f64)
                        min=1.0
                        max=64.0
                        on_change=Callback::new(move |value: f64| {
                            app.update_config(|config| config.eraser_size = value as f32)
                        })
                    />
                </div>
                <div class="if-editor__param">
                    <RangeField
                        label="橡皮硬度"
                        value=Signal::derive(move || eraser_hardness() as f64)
                        min=0.0
                        max=100.0
                        on_change=Callback::new(move |value: f64| {
                            app.update_config(|config| config.eraser_hardness = value as f32)
                        })
                    />
                </div>
            </div>

            <div class="if-editor__stage">
                <div class="if-canvas if-canvas--editor">
                    <div class="if-checker"></div>
                    <canvas
                        class="if-canvas__surface if-editor__canvas"
                        width=PREVIEW_SIZE
                        height=PREVIEW_SIZE
                        style=move || format!("cursor:{};{}", cursor(), size_style())
                        node_ref=canvas_ref
                        on:pointerdown=pointer_down
                        on:pointermove=pointer_move
                        on:pointerup=pointer_up
                        on:pointercancel=move |_| points.set_value(Vec::new())
                    ></canvas>
                    <canvas
                        class="if-canvas__ants if-editor__canvas"
                        width=PREVIEW_SIZE
                        height=PREVIEW_SIZE
                        style=size_style
                        node_ref=ants_ref
                    ></canvas>
                    <Show when=move || loading.get()>
                        <div class="if-canvas__busy">
                            <Spinner label="加载原图…" />
                        </div>
                    </Show>
                </div>
            </div>

            <div class="if-editor__hint">{hint()}</div>
        </div>
    }
}

fn eraser_stroke(config: &ItemConfig, points: &[BrushPoint]) -> BrushStroke {
    BrushStroke {
        points: points.to_vec(),
        color: "#00000000".to_string(),
        size: config.eraser_size,
        opacity: 1.0,
        mode: BrushMode::Erase,
        clip_to_mask: false,
    }
}
