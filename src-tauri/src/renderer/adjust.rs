use image::RgbaImage;

/// Rec.601 luma weights — the same mix image editors use for a grayscale pass.
const LUMA: [f32; 3] = [0.299, 0.587, 0.114];

/// Scale the chroma of every pixel: `factor` 0 = fully desaturated
/// (grayscale), 1 = unchanged. Alpha is left alone.
pub fn apply_saturation(source: &RgbaImage, factor: f32) -> RgbaImage {
    let factor = factor.clamp(0.0, 1.0);
    let mut out = source.clone();
    for px in out.pixels_mut() {
        let [r, g, b, _] = px.0;
        let (r, g, b) = (r as f32, g as f32, b as f32);
        let luma = r * LUMA[0] + g * LUMA[1] + b * LUMA[2];
        for (channel, value) in px.0.iter_mut().take(3).enumerate() {
            let original = [r, g, b][channel];
            *value = (luma + (original - luma) * factor)
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
    out
}
