#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::{render_master, validate_config};
    use image::RgbaImage;

    fn gloss_config() -> RenderConfig {
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
            backplate_type: BackplateType::None,
            backplate_color: "#00000000".into(),
            gradient_start_color: "#00000000".into(),
            gradient_end_color: "#00000000".into(),
            gradient_angle_degrees: 0.0,
            outer_shadow: OuterShadowConfig {
                enabled: false,
                offset_x: 0.0,
                offset_y: 0.0,
                blur_radius: 0.0,
                spread: 0.0,
                color: "#00000000".into(),
            },
            stroke: StrokeConfig {
                width: 0.0,
                color: "#00000000".into(),
            },
            gloss: GlossConfig {
                enabled: true,
                width: 4.0,
                strength: 1.0,
                light_color: "#FFFFFFFF".into(),
                base_color: "#808080FF".into(),
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
                enabled: false,
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

    fn render_gloss() -> RgbaImage {
        let source = RgbaImage::new(1, 1);
        render_master(&source, &validate_config(&gloss_config()).unwrap()).unwrap()
    }

    fn sample(img: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
        img.get_pixel(x, y).0
    }

    #[test]
    fn gloss_base_ring_edges() {
        let img = render_gloss();
        // The base ring must exist on the edges outside the highlight spans:
        // bottom edge, right edge, and the corner arcs that flank the
        // top-left highlight (bottom-left and top-right roundings).
        assert_eq!(sample(&img, 128, 238), [128, 128, 128, 255]);
        assert_eq!(sample(&img, 238, 128), [128, 128, 128, 255]);
        assert_eq!(sample(&img, 32, 224), [128, 128, 128, 255]);
        assert_eq!(sample(&img, 214, 24), [128, 128, 128, 255]);
    }

    #[test]
    fn gloss_top_left_highlight() {
        let img = render_gloss();
        // The top-left corner arc peaks at full strength...
        assert_eq!(sample(&img, 32, 32), [255, 255, 255, 255]);
        // ...while the straight left/top edges sit slightly below it, and both
        // ends fade out into the base ring (samples here are still clearly
        // above the 128 gray base).
        for (x, y) in [(18, 128), (128, 18)] {
            let p = sample(&img, x, y);
            assert!(p[0] > 235 && p[0] < 255, "edge ({x},{y}) should be near 0.85 opacity, got {}", p[0]);
        }
    }

    #[test]
    fn gloss_top_left_ends_fade_into_base() {
        let img = render_gloss();
        // Near the upper end of the bottom-left rounding (start of the gloss)
        // and the left end of the top-right rounding (end of the gloss) the
        // highlight must transition gradually back to the edge base color.
        let start = sample(&img, 18, 186);
        assert!(start[0] < 160, "gloss start should be close to the base color, got {}", start[0]);
        let mid_fade = sample(&img, 18, 178);
        assert!(
            mid_fade[0] > 135 && mid_fade[0] < 235,
            "gloss start should show a gradient in between, got {}",
            mid_fade[0]
        );
        let end = sample(&img, 186, 18);
        assert!(end[0] < 160, "gloss end should be close to the base color, got {}", end[0]);
    }

    #[test]
    fn gloss_bottom_right_highlight_fades_in_and_out() {
        let img = render_gloss();
        // The bottom-right rounding arc fades in from the edge base color at
        // its left side (bottom edge end)...
        assert!(sample(&img, 188, 237)[0] < 170, "arc start should fade in from the base color");
        assert!(
            sample(&img, 207, 233)[0] > 180,
            "arc should brighten towards the middle"
        );
        // ...holds the same reduced level as the straight edges across the
        // middle...
        let mid = sample(&img, 223, 223);
        assert!(
            mid[0] > 235 && mid[0] < 255,
            "arc middle should match the edge level, got {}",
            mid[0]
        );
        // ...and fades back out to the base color at its upper side (right
        // edge end).
        assert!(sample(&img, 238, 188)[0] < 170, "arc end should fade back to the base color");
    }
}
