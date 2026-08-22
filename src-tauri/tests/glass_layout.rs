#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::{render_master, validate_config};
    use image::RgbaImage;

    fn glass_config() -> RenderConfig {
        RenderConfig {
            foreground_scale_percent: 100.0,
            foreground_fit: ForegroundFit::Contain,
            foreground_offset_x: 0.0,
            foreground_offset_y: 0.0,
            foreground_rotation_degrees: 0.0,
            foreground_opacity_percent: 100.0,
            canvas_inset: 16.0,
            content_scale_percent: 100.0,
            shape: IconShape::RoundedRectangle,
            corner_radius: 52.0,
            squircle_exponent: 4.0,
            backplate_type: BackplateType::Gradient,
            backplate_color: "#00000000".into(),
            gradient_start_color: "#9A9A9AFF".into(),
            gradient_end_color: "#828282FF".into(),
            gradient_angle_degrees: 90.0,
            outer_shadow: OuterShadowConfig {
                enabled: true,
                offset_x: 0.0,
                offset_y: 12.0,
                blur_radius: 24.0,
                spread: 0.0,
                color: "#00000060".into(),
            },
            stroke: StrokeConfig {
                width: 0.0,
                color: "#00000000".into(),
            },
            gloss: GlossConfig {
                enabled: true,
                width: 3.0,
                strength: 1.0,
                light_color: "#FFFFFFFF".into(),
                base_color: "#C4C4C4FF".into(),
                feather_blur: 0.0,
            },
            auto_cutout: AutoCutoutConfig {
                enabled: false,
                tolerance: 20.0,
                feather: 8.0,
            },
            hq_render: HqRenderConfig {
                enabled: false,
                thresh: 0.5,
                icon_ratio: 0.66,
                bg: 0.22,
                light_mix: 0.9,
                dark_mix: 0.72,
                gloss: 0.16,
                icon_light: 0.08,
                corner: 0.22,
                shape: HqPanelShape::Rect,
                offset_x: 0.0,
                offset_y: 0.0,
                custom_bg_enabled: false,
                custom_bg_color: "#FFFFFFFF".into(),
                shadow_opacity: 0.22,
                shadow_blur_factor: 0.022,
                shadow_offset_factor: 0.012,
                shadow_fade: 0.25,
                shadow_mode: HqShadowMode::Icon,
            },
            glass_render: GlassRenderConfig {
                enabled: true,
                color_retention: 0.05,
                bevel_radius: 0.12,
                normal_strength: 1.0,
                specular_strength: 0.25,
                fresnel_strength: 0.5,
                ao_strength: 0.35,
                contrast_strength: 0.0,
            },
        }
    }

    fn render_glass() -> RgbaImage {
        let source = RgbaImage::new(1, 1);
        render_master(&source, &validate_config(&glass_config()).unwrap()).unwrap()
    }

    fn sample(img: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
        img.get_pixel(x, y).0
    }

    #[test]
    fn glass_background_is_vertical_gradient() {
        // The backplate uses the classic gradient driven by the user's
        // backplate parameters (here the 1x1 source leaves it fully visible).
        let img = render_glass();
        let top = sample(&img, 128, 44);
        let bottom = sample(&img, 128, 210);
        assert_eq!(top[3], 255);
        assert_eq!(bottom[3], 255);
        assert!(
            top[0] as u16 > bottom[0] as u16,
            "top ({}) should be lighter than bottom ({})",
            top[0],
            bottom[0]
        );
        assert!(top[0] >= 138 && top[0] <= 170);
        assert!(bottom[0] >= 120 && bottom[0] < 145);
    }

    #[test]
    fn glass_plate_has_edge_gloss() {
        // The edge gloss is the classic one, driven by the user's gloss
        // parameters on the classic shape mask: top/left highlight, base ring
        // elsewhere.
        let img = render_glass();
        let left = sample(&img, 18, 128);
        assert!(
            left[0] > 235,
            "left plate edge should show the gloss highlight, got {}",
            left[0]
        );
        let bottom_edge = sample(&img, 128, 238);
        assert!(
            bottom_edge[0] >= 165 && bottom_edge[0] <= 225,
            "bottom edge should show the base ring, got {}",
            bottom_edge[0]
        );
        let corner = sample(&img, 32, 32);
        assert!(
            corner[0] > 235,
            "plate corner should show the gloss highlight, got {}",
            corner[0]
        );
    }

    #[test]
    fn glass_subject_is_grayscale_with_relief() {
        // The original image is rendered as black & white (max RGB channel
        // keeps bright glyphs white), with a beveled relief: the bevels catch
        // the top-left light, so the top edge is brighter than the flat
        // middle and the bottom edge darker.
        let source = RgbaImage::from_pixel(256, 256, image::Rgba([200, 60, 20, 255]));
        let rendered =
            render_master(&source, &validate_config(&glass_config()).unwrap()).unwrap();
        let middle = rendered.get_pixel(128, 128).0;
        assert!(
            (middle[0] as i16 - middle[1] as i16).abs() <= 10,
            "glyph must be near-gray (r~g), got {} vs {}",
            middle[0],
            middle[1]
        );
        assert!(
            (middle[1] as i16 - middle[2] as i16).abs() <= 10,
            "glyph must be near-gray (g~b)"
        );
        // Value of (200, 60, 20) = 200.
        assert!(
            (170..=225).contains(&middle[0]),
            "glyph middle should stay near the max-channel gray, got {}",
            middle[0]
        );
        // Bevel midpoints: 15px inside the top/bottom edges (inset 16, bevel 30px).
        let top = rendered.get_pixel(128, 31).0;
        let bottom = rendered.get_pixel(128, 225).0;
        assert!(
            top[0] as u16 > middle[0] as u16 + 5,
            "top bevel should catch the light (top {}, middle {})",
            top[0],
            middle[0]
        );
        assert!(
            middle[0] as u16 > bottom[0] as u16 + 5,
            "bottom bevel should fall into shadow (middle {}, bottom {})",
            middle[0],
            bottom[0]
        );
    }

    #[test]
    fn glass_projects_shadow_below_panel() {
        // The outer shadow is the classic one from the user's shadow params.
        let img = render_glass();
        let below = sample(&img, 128, 250);
        assert!(below[3] > 0, "shadow should appear below the panel");
        // Far corners are only faintly reached by the blurred shadow.
        assert!(sample(&img, 4, 4)[3] < 30, "far corner should be nearly untouched");
    }
}
