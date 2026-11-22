//! 预览画布。
//!
//! 宿主只返回一张 PNG，所以这里的职责很窄：把 PNG 画到 canvas、把正在拖的笔迹
//! 叠在上面、把点击换算回 256 坐标系。渲染本身全在 Rust 宿主里，前端不复制一份
//! 合成逻辑 —— 那是最容易两边不一致的地方。

use glass_design::prelude::*;
use leptos::html;
use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::api;
use crate::canvas::{
    clear_canvas, context, draw_ants, draw_scene, event_canvas, live_stroke, point_in_space,
};
use crate::state::App;
use crate::types::*;
use crate::util;

#[component]
pub fn PreviewPane() -> impl IntoView {
    let app = expect_context::<App>();
    let canvas_ref = NodeRef::<html::Canvas>::new();
    let ants_ref = NodeRef::<html::Canvas>::new();
    let image = StoredValue::new_local(Option::<web_sys::HtmlImageElement>::None);
    let points = StoredValue::new_local(Vec::<BrushPoint>::new());
    let generation = StoredValue::new(0u64);

    // 预览：配置一变就防抖重渲染。用代数（generation）丢弃过期响应，
    // 而不是取消任务 —— wasm 侧没有可靠的取消原语。
    Effect::new(move |_| {
        let item = app.current_item();
        let config = app.current_config();
        let (Some(item), Some(config)) = (item, config) else {
            generation.update_value(|value| *value += 1);
            app.preview.set(PreviewState::default());
            image.set_value(None);
            draw_scene(canvas_ref.get_untracked(), None, None);
            return;
        };

        let token = generation.get_value() + 1;
        generation.set_value(token);
        app.preview.update(|state| state.loading = true);
        let request = config.render_request(item.source_path);

        spawn_local(async move {
            util::sleep_ms(180).await;
            if generation.get_value() != token {
                return;
            }
            match api::render_preview(request).await {
                Ok(response) => {
                    if generation.get_value() != token {
                        return;
                    }
                    let data_url = format!("data:image/png;base64,{}", response.png_base64);
                    app.preview.set(PreviewState {
                        data_url: Some(data_url.clone()),
                        processed: Some((
                            response.processed_source_width,
                            response.processed_source_height,
                        )),
                        loading: false,
                        error: None,
                    });
                    if let Some(loaded) = util::load_image(&data_url).await {
                        if generation.get_value() != token {
                            return;
                        }
                        image.set_value(Some(loaded.clone()));
                        draw_scene(canvas_ref.get_untracked(), Some(&loaded), None);
                    }
                }
                Err(err) => {
                    if generation.get_value() != token {
                        return;
                    }
                    app.preview.set(PreviewState {
                        loading: false,
                        error: Some(err),
                        ..PreviewState::default()
                    });
                }
            }
        });
    });

    // 魔棒选区：静态蚂蚁线。不做逐帧动画 —— 选区只活到「删除/取消」为止，
    // 静态虚线已经足够读，省掉一条常驻定时器。
    Effect::new(move |_| {
        let selection = app.wand_selection.get();
        let Some(selection) = selection else {
            clear_canvas(ants_ref.get_untracked());
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
        let Some(canvas) = event_canvas(&ev) else {
            return;
        };
        let Some(config) = app.current_config() else {
            return;
        };
        let (x, y) = point_in_space(&ev, &canvas);
        let tool = app.tool.get_untracked();

        match tool {
            Tool::Eyedropper => {
                if let Some(context) = context(&canvas) {
                    let px = (x / CANVAS_SPACE * canvas.width() as f32) as i32;
                    let py = (y / CANVAS_SPACE * canvas.height() as f32) as i32;
                    if let Ok(data) = context.get_image_data(px as f64, py as f64, 1.0, 1.0) {
                        let pixels = data.data().0;
                        let hex = format!(
                            "#{:02X}{:02X}{:02X}",
                            pixels.first().copied().unwrap_or(0),
                            pixels.get(1).copied().unwrap_or(0),
                            pixels.get(2).copied().unwrap_or(0)
                        );
                        app.apply_picked_color(&hex);
                    }
                }
            }
            Tool::MagicWand | Tool::SourceEraser => {
                app.source_editor_open.set(true);
            }
            Tool::Brush | Tool::Eraser => {
                let _ = canvas.set_pointer_capture(ev.pointer_id());
                points.set_value(vec![BrushPoint { x, y }]);
                let stroke = live_stroke(&config, &points.get_value(), tool == Tool::Eraser);
                draw_scene(Some(canvas), image.get_value().as_ref(), Some(&stroke));
            }
            Tool::None => {}
        }
    };

    let pointer_move = move |ev: web_sys::PointerEvent| {
        let tool = app.tool.get_untracked();
        if tool != Tool::Brush && tool != Tool::Eraser {
            return;
        }
        if points.with_value(Vec::is_empty) {
            return;
        }
        let Some(canvas) = event_canvas(&ev) else {
            return;
        };
        let Some(config) = app.current_config() else {
            return;
        };
        let (x, y) = point_in_space(&ev, &canvas);
        points.update_value(|list| list.push(BrushPoint { x, y }));
        let stroke = live_stroke(&config, &points.get_value(), tool == Tool::Eraser);
        draw_scene(Some(canvas), image.get_value().as_ref(), Some(&stroke));
    };

    let pointer_up = move |ev: web_sys::PointerEvent| {
        let tool = app.tool.get_untracked();
        if tool != Tool::Brush && tool != Tool::Eraser {
            return;
        }
        if points.with_value(Vec::is_empty) {
            return;
        }
        let Some(canvas) = event_canvas(&ev) else {
            return;
        };
        let Some(config) = app.current_config() else {
            return;
        };
        let (x, y) = point_in_space(&ev, &canvas);
        points.update_value(|list| {
            if list.len() == 1 {
                list.push(BrushPoint { x, y });
            }
        });
        let _ = canvas.release_pointer_capture(ev.pointer_id());

        let stroke = BrushStroke {
            points: points.get_value(),
            color: config.brush_color.clone(),
            size: config.brush_size,
            opacity: 1.0,
            mode: if tool == Tool::Eraser {
                BrushMode::Erase
            } else {
                BrushMode::Paint
            },
            clip_to_mask: config.brush_clip_to_mask,
        };
        points.set_value(Vec::new());
        app.add_brush_stroke(stroke);
    };

    let cancel = move |_| points.set_value(Vec::new());

    // ------------------------------------------------------------ 视图

    let has_item = move || app.current_item().is_some();
    let loading = move || app.preview.get().loading;
    let error_text = move || app.preview.get().error.map(|err| err.message);
    let processed = move || app.preview.get().processed;
    let tool_hint = move || app.tool.get().hint();
    let cursor = move || match app.tool.get() {
        Tool::None => "default",
        Tool::Eraser => "cell",
        _ => "crosshair",
    };
    let source_size = move || {
        app.current_item()
            .map(|item| format!("源图 {}×{}", item.source_width, item.source_height))
    };
    let wand_pending = move || app.wand_selection.get().is_some();
    let wand_count = move || {
        app.current_config()
            .map(|config| config.wand_strokes.len() + config.eraser_strokes.len())
            .unwrap_or(0)
    };

    view! {
        <section class="if-pane if-stage">
            <Show
                when=has_item
                fallback=move || view! {
                    <div class="if-drop" class:if-drop--active=move || app.drag_active.get()>
                        <Icon name=IconName::UPLOAD size=36 />
                        <p class="if-drop__title">"拖入图片、EXE、DLL、LNK"</p>
                        <p class="if-drop__hint">"或点工具栏「导入」"</p>
                    </div>
                }
            >
                <div class="if-canvas-wrap">
                    <div class="if-canvas">
                        <div class="if-checker"></div>
                        <canvas
                            class="if-canvas__surface"
                            width=PREVIEW_SIZE
                            height=PREVIEW_SIZE
                            style=move || format!("cursor:{}", cursor())
                            node_ref=canvas_ref
                            on:pointerdown=pointer_down
                            on:pointermove=pointer_move
                            on:pointerup=pointer_up
                            on:pointercancel=cancel
                        ></canvas>
                        <canvas
                            class="if-canvas__ants"
                            width=PREVIEW_SIZE
                            height=PREVIEW_SIZE
                            node_ref=ants_ref
                        ></canvas>
                        <Show when=loading>
                            <div class="if-canvas__busy">
                                <Spinner label="渲染中…" />
                            </div>
                        </Show>
                    </div>
                </div>

                <div class="if-meta">
                    <span>{source_size()}</span>
                    {move || processed().map(|(w, h)| view! {
                        <span>{format!("处理源图 {w}×{h}")}</span>
                    })}
                    <span>"主图 256×256"</span>
                    {move || {
                        let hint = tool_hint();
                        (!hint.is_empty()).then(|| view! {
                            <span class="if-meta__accent">{hint}</span>
                        })
                    }}
                    {move || {
                        let count = wand_count();
                        let label = if count > 0 {
                            format!("编辑原图（{count} 处修改）")
                        } else {
                            "编辑原图".to_string()
                        };
                        view! {
                            <Button
                                label=label
                                size=Size::Sm
                                on_click=Callback::new(move |_| app.source_editor_open.set(true))
                            />
                        }
                    }}
                    <Show when=wand_pending>
                        <Button
                            label="删除选中区域"
                            variant=ButtonVariant::Danger
                            size=Size::Sm
                            on_click=Callback::new(move |_| app.delete_wand_selection())
                        />
                    </Show>
                </div>

                {move || error_text().map(|message| view! {
                    <InlineMessage
                        tone=Tone::Danger
                        title="预览失败"
                        description=message
                    />
                })}
            </Show>
        </section>
    }
}

