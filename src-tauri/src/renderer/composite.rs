use image::RgbaImage;

use super::color::linear_channel_to_srgb;

pub struct LinearPremultipliedImage {
    width: u32,
    height: u32,
    pub(crate) data: Vec<[f32; 4]>, // RGBA premultiplied
}

impl LinearPremultipliedImage {
    pub fn transparent(width: u32, height: u32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            data: vec![[0.0f32; 4]; count],
        }
    }

    pub fn from_srgb_rgba8(image: &RgbaImage) -> Self {
        let mut result = Self::transparent(image.width(), image.height());
        for (idx, pixel) in image.pixels().enumerate() {
            let [r, g, b, a] = pixel.0;
            let alpha = a as f32 / 255.0;
            result.data[idx] = [
                super::color::srgb_channel_to_linear(r as f32 / 255.0) * alpha,
                super::color::srgb_channel_to_linear(g as f32 / 255.0) * alpha,
                super::color::srgb_channel_to_linear(b as f32 / 255.0) * alpha,
                alpha,
            ];
        }
        result
    }

    pub fn alpha_plane(&self) -> super::mask::AlphaMask {
        self.data.iter().map(|p| p[3]).collect()
    }

    pub fn into_srgb_rgba8(self) -> RgbaImage {
        let mut bytes = Vec::with_capacity(self.data.len() * 4);
        for pixel in self.data {
            let [r, g, b, a] = pixel;
            if a <= 0.0 {
                bytes.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let r_srgb = (linear_channel_to_srgb(r / a) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let g_srgb = (linear_channel_to_srgb(g / a) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let b_srgb = (linear_channel_to_srgb(b / a) * 255.0)
                    .round()
                    .clamp(0.0, 255.0) as u8;
                let a_u8 = (a * 255.0).round().clamp(0.0, 255.0) as u8;
                bytes.extend_from_slice(&[r_srgb, g_srgb, b_srgb, a_u8]);
            }
        }
        RgbaImage::from_raw(self.width, self.height, bytes)
            .expect("inconsistent dimensions in into_srgb_rgba8")
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}

pub fn over(destination: &mut LinearPremultipliedImage, source: LinearPremultipliedImage) {
    assert_eq!(destination.width, source.width);
    assert_eq!(destination.height, source.height);

    for (dst, src) in destination.data.iter_mut().zip(source.data.iter()) {
        let src_a = src[3];
        if src_a <= 0.0 {
            continue;
        }
        if src_a >= 1.0 {
            *dst = *src;
            continue;
        }
        let dst_a = dst[3];
        let out_a = src_a + dst_a * (1.0 - src_a);
        if out_a <= 0.0 {
            dst[0] = 0.0;
            dst[1] = 0.0;
            dst[2] = 0.0;
            dst[3] = 0.0;
        } else {
            dst[0] = (src[0] + dst[0] * (1.0 - src_a)) / out_a * out_a;
            dst[1] = (src[1] + dst[1] * (1.0 - src_a)) / out_a * out_a;
            dst[2] = (src[2] + dst[2] * (1.0 - src_a)) / out_a * out_a;
            dst[3] = out_a;
        }
    }
}
