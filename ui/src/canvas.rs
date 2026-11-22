//! canvas 绘制工具。
//!
//! 预览与原图编辑共用同一套：两边画的都是「一张 PNG + 一条笔迹」，
//! 分开写就会一边支持 `clip_to_mask`、另一边不支持。

use wasm_bindgen::{Clamped, JsCast};

use crate::types::*;
use crate::util;

pub fn context(canvas: &web_sys::HtmlCanvasElement) -> Option<web_sys::CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .ok()
}

pub fn clear_canvas(canvas: Option<web_sys::HtmlCanvasElement>) {
    let Some(canvas) = canvas else { return };
    if let Some(context) = context(&canvas) {
        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
    }
}

pub fn draw_scene(
    canvas: Option<web_sys::HtmlCanvasElement>,
    image: Option<&web_sys::HtmlImageElement>,
    stroke: Option<&BrushStroke>,
) {
    let Some(canvas) = canvas else { return };
    let Some(context) = context(&canvas) else { return };
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    context.clear_rect(0.0, 0.0, width, height);
    let Some(image) = image else { return };
    let _ = context.draw_image_with_html_image_element_and_dw_and_dh(image, 0.0, 0.0, width, height);

    let Some(stroke) = stroke else { return };
    if stroke.points.is_empty() {
        return;
    }

    let erase = stroke.mode == BrushMode::Erase;
    let _ = context.save();
    let _ = context.set_global_composite_operation(if erase {
        "destination-out"
    } else {
        "source-over"
    });
    let color = util::normalize_color8(&stroke.color, "#FF3B30FF");
    let alpha = u8::from_str_radix(&color[7..9], 16).unwrap_or(255) as f64 / 255.0;
    context.set_global_alpha(stroke.opacity as f64 * alpha);
    context.set_line_width(stroke.size as f64 * width / CANVAS_SPACE as f64);
    context.set_line_cap("round");
    context.set_line_join("round");
    if erase {
        context.set_stroke_style_str("rgba(0,0,0,1)");
    } else {
        context.set_stroke_style_str(&color[0..7]);
    }
    context.begin_path();
    let scale = width / CANVAS_SPACE as f64;
    let first = &stroke.points[0];
    context.move_to(first.x as f64 * scale, first.y as f64 * scale);
    for point in stroke.points.iter().skip(1) {
        context.line_to(point.x as f64 * scale, point.y as f64 * scale);
    }
    context.stroke();
    let _ = context.restore();

    if !erase && stroke.clip_to_mask {
        let _ = context.save();
        let _ = context.set_global_composite_operation("destination-in");
        let _ = context.draw_image_with_html_image_element_and_dw_and_dh(
            image, 0.0, 0.0, width, height,
        );
        let _ = context.restore();
    }
}

/// 把魔棒掩码画成「半透明红填充 + 黑白虚线边界」。
///
/// 边界判定用四邻域内外翻转，虚线相位由坐标散列决定：不依赖时间，
/// 所以同一块选区每次画出来一模一样，不会闪。
pub fn draw_ants(canvas: Option<web_sys::HtmlCanvasElement>, mask: &web_sys::HtmlImageElement) {
    let Some(canvas) = canvas else { return };
    let Some(context) = context(&canvas) else { return };
    let width = canvas.width();
    let height = canvas.height();
    context.clear_rect(0.0, 0.0, width as f64, height as f64);
    let _ = context.draw_image_with_html_image_element_and_dw_and_dh(
        mask,
        0.0,
        0.0,
        width as f64,
        height as f64,
    );
    let Ok(source) = context.get_image_data(0.0, 0.0, width as f64, height as f64) else {
        return;
    };
    let mut pixels = source.data().to_vec();
    for index in (0..pixels.len()).step_by(4) {
        let value = pixels[index];
        if value > 0 {
            pixels[index] = 255;
            pixels[index + 1] = 0;
            pixels[index + 2] = 0;
            pixels[index + 3] = value.max(60);
        }
    }
    if let Ok(tinted) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(pixels.as_slice()),
        width,
        height,
    ) {
        let _ = context.put_image_data(&tinted, 0.0, 0.0);
    }

    let stride = width as usize;
    let mut edges = vec![0u8; pixels.len()];
    for y in 1..height as usize - 1 {
        for x in 1..width as usize - 1 {
            let index = (y * stride + x) * 4;
            let inside = pixels[index] >= 128;
            let neighbours = [index - 4, index + 4, index - stride * 4, index + stride * 4];
            if !neighbours
                .iter()
                .any(|offset| (pixels[*offset] >= 128) != inside)
            {
                continue;
            }
            let dash = (x * 5 + y * 7) % 10;
            let value = if inside {
                if dash < 5 {
                    0
                } else {
                    255
                }
            } else if dash < 5 {
                255
            } else {
                0
            };
            edges[index] = value;
            edges[index + 1] = value;
            edges[index + 2] = value;
            edges[index + 3] = 255;
        }
    }
    if let Ok(overlay) = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(edges.as_slice()),
        width,
        height,
    ) {
        let _ = context.put_image_data(&overlay, 0.0, 0.0);
    }
}

pub fn event_canvas(ev: &web_sys::PointerEvent) -> Option<web_sys::HtmlCanvasElement> {
    ev.current_target()?.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

/// 屏幕坐标 → 256 渲染坐标系。预览和原图编辑都用这一个换算，
/// 否则同一笔在两个界面里落点会差几个像素。
pub fn point_in_space(ev: &web_sys::PointerEvent, canvas: &web_sys::HtmlCanvasElement) -> (f32, f32) {
    let rect = canvas.get_bounding_client_rect();
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return (0.0, 0.0);
    }
    let x = ((ev.client_x() as f64 - rect.left()) / rect.width()) * CANVAS_SPACE as f64;
    let y = ((ev.client_y() as f64 - rect.top()) / rect.height()) * CANVAS_SPACE as f64;
    (
        x.clamp(0.0, CANVAS_SPACE as f64) as f32,
        y.clamp(0.0, CANVAS_SPACE as f64) as f32,
    )
}

/// 拖拽中的笔迹。`erase` 由当前工具决定，不读 `brush_mode`：
/// 工具是「现在要干什么」，`brush_mode` 是「画笔上次用哪种模式」，两者可能不同步。
pub fn live_stroke(config: &ItemConfig, points: &[BrushPoint], erase: bool) -> BrushStroke {
    BrushStroke {
        points: points.to_vec(),
        color: config.brush_color.clone(),
        size: config.brush_size,
        opacity: 1.0,
        mode: if erase { BrushMode::Erase } else { BrushMode::Paint },
        clip_to_mask: config.brush_clip_to_mask,
    }
}
