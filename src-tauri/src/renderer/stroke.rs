use super::color::{parse_hex_rgba, LinearRgba};
use super::composite::LinearPremultipliedImage;
use super::mask::{generate_inset_shape_mask, AlphaMask};
use super::pipeline::ValidatedRenderConfig;

pub fn render_inner_stroke(
    mask: &AlphaMask,
    config: &ValidatedRenderConfig,
) -> LinearPremultipliedImage {
    let size = 256u32;
    let mut result = LinearPremultipliedImage::transparent(size, size);

    let width = config.stroke_width;
    if width <= 0.0 {
        return result;
    }

    let stroke_color = parse_hex_rgba(&config.stroke_color).unwrap_or(LinearRgba {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });

    let inset_mask = generate_inset_shape_mask(size, config, super::BASE_SHAPE_INSET + width);

    for i in 0..mask.len() {
        let outer_alpha = mask[i];
        let inner_alpha = inset_mask[i];
        let stroke_alpha = (outer_alpha - inner_alpha).clamp(0.0, 1.0);
        if stroke_alpha <= 0.0 {
            continue;
        }
        result.data[i][0] = stroke_color.r * stroke_alpha;
        result.data[i][1] = stroke_color.g * stroke_alpha;
        result.data[i][2] = stroke_color.b * stroke_alpha;
        result.data[i][3] = stroke_color.a * stroke_alpha;
    }

    result
}
