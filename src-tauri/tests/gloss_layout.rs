#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::gloss::{edge_glow_field, EdgeGlossParams};
    use iconforge_lib::renderer::{render_master, validate_config};
    use image::RgbaImage;

    /// Gloss profile with the reference defaults (light from the top).
    fn gloss_config() -> GlossConfig {
        GlossConfig {
            enabled: true,
            width: 4.0,
            strength: 1.0,
            light_color: "#FFFFFFFF".into(),
            top_strength: 0.42,
            bottom_strength: 0.35,
            top_decay: 38.0,
            bottom_decay: 18.0,
            angle_degrees: 0.0,
        }
    }

    fn render_config(gloss: GlossConfig) -> RenderConfig {
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
            gloss,
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
            source_saturation_percent: 100.0,
        }
    }

    /// A full-canvas gray source: the foreground fills the shape mask, so the
    /// glow has real icon pixels to light along the edge.
    fn render(gloss: GlossConfig) -> RgbaImage {
        let source = RgbaImage::from_pixel(256, 256, image::Rgba([100, 100, 100, 255]));
        render_master(&source, &validate_config(&render_config(gloss)).unwrap()).unwrap()
    }

    fn sample(img: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
        img.get_pixel(x, y).0
    }

    #[test]
    fn gloss_lights_the_lit_side_only() {
        let img = render(gloss_config());
        // Light comes from the top: the top edge is clearly lifted off the
        // 100-gray source...
        let top = sample(&img, 128, 18)[0];
        assert!(top > 130, "top edge should be lit, got {top}");
        // ...the bottom edge gets the weaker opposite band...
        let bottom = sample(&img, 128, 237)[0];
        assert!(
            bottom > 100 && bottom < top,
            "bottom edge should be dimmer than the top, got {bottom} (top {top})"
        );
        // ...and the left/right edges sit in the middle of the vertical ramp,
        // where the rim light has all but faded out.
        for (x, y) in [(18, 128), (238, 128)] {
            let p = sample(&img, x, y)[0];
            assert!(
                p <= 108,
                "side edge ({x},{y}) should stay near the source gray, got {p}"
            );
        }
    }

    #[test]
    fn gloss_angle_rotates_the_light_axis() {
        // 90° turns the light source to the right: the right edge takes the
        // strong band, the left edge keeps the weaker opposite band, and the
        // top/bottom edges (now in the middle of the ramp) go back to the
        // source gray.
        let mut cfg = gloss_config();
        cfg.angle_degrees = 90.0;
        let img = render(cfg);
        let right = sample(&img, 238, 128)[0];
        let left = sample(&img, 18, 128)[0];
        assert!(right > 130, "right edge should be lit at 90°, got {right}");
        assert!(
            left > 100 && left < right,
            "left edge keeps the weaker band, got {left} (right {right})"
        );
        for (x, y) in [(128, 18), (128, 237)] {
            let p = sample(&img, x, y)[0];
            assert!(
                p <= 108,
                "top/bottom edge ({x},{y}) should be dark at 90°, got {p}"
            );
        }
    }

    #[test]
    fn gloss_angle_180_flips_the_lit_side() {
        // With only the upper band enabled, 180° must move the whole highlight
        // from the top edge to the bottom edge.
        let mut cfg = gloss_config();
        cfg.bottom_strength = 0.0;
        cfg.angle_degrees = 180.0;
        let img = render(cfg);
        let bottom = sample(&img, 128, 237)[0];
        assert!(bottom > 130, "bottom edge should be lit at 180°, got {bottom}");
        let top = sample(&img, 128, 18)[0];
        assert!(top <= 108, "top edge should be dark at 180°, got {top}");
    }

    #[test]
    fn gloss_fades_away_from_the_boundary() {
        let img = render(gloss_config());
        // The rim decays as exp(-d / width), so it is gone a few pixels in.
        let edge = sample(&img, 128, 18)[0];
        let inner = sample(&img, 128, 26)[0];
        let deep = sample(&img, 128, 40)[0];
        assert!(
            edge > inner && inner > deep,
            "glow should decay inwards: {edge} > {inner} > {deep}"
        );
        assert_eq!(deep, 100, "the icon body far from the edge is untouched");
    }

    #[test]
    fn gloss_needs_underlying_content() {
        // The glow lights existing pixels, it does not paint a ring: with a
        // small foreground and no backplate the edge band has no icon pixels,
        // so it must stay fully transparent.
        let mut config = render_config(gloss_config());
        config.foreground_scale_percent = 50.0;
        let source = RgbaImage::from_pixel(256, 256, image::Rgba([100, 100, 100, 255]));
        let img = render_master(&source, &validate_config(&config).unwrap()).unwrap();
        for (x, y) in [(18, 128), (128, 18), (128, 238), (238, 128), (32, 32)] {
            assert_eq!(
                sample(&img, x, y)[3],
                0,
                "edge band ({x},{y}) has no icon pixels and must stay transparent"
            );
        }
        assert_eq!(sample(&img, 128, 128), [100, 100, 100, 255]);
    }

    #[test]
    fn gloss_follows_the_edge_when_the_shape_touches_the_canvas() {
        // The default preset sits flush against the canvas (canvasInset 0), so
        // there are no uncovered pixels above the top edge. The rim must still
        // run along the whole border instead of only around the corners.
        let mut config = render_config(gloss_config());
        config.canvas_inset = 0.0;
        let source = RgbaImage::from_pixel(256, 256, image::Rgba([100, 100, 100, 255]));
        let img = render_master(&source, &validate_config(&config).unwrap()).unwrap();

        for x in [64u32, 100, 128, 156, 192] {
            let p = sample(&img, x, 1)[0];
            assert!(p > 130, "top edge ({x},1) should be lit, got {p}");
        }
        for x in [64u32, 128, 192] {
            let p = sample(&img, x, 254)[0];
            assert!(p > 110, "bottom edge ({x},254) should be lit, got {p}");
        }
    }

    fn square_mask(size: usize, inset: f32) -> Vec<f32> {
        let mut mask = vec![0.0f32; size * size];
        for y in 0..size {
            for x in 0..size {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                if px >= inset && px <= size as f32 - inset && py >= inset && py <= size as f32 - inset
                {
                    mask[y * size + x] = 1.0;
                }
            }
        }
        mask
    }

    #[test]
    fn edge_glow_field_stays_inside_the_silhouette() {
        let size = 256usize;
        let mask = square_mask(size, 16.0);
        let params = EdgeGlossParams {
            rim: 4.0,
            strength: 1.0,
            top: 0.42,
            bottom: 0.35,
            top_decay: 38.0,
            bottom_decay: 18.0,
            angle_degrees: 0.0,
        };
        let field = edge_glow_field(&mask, size, 112.0, &params);
        assert_eq!(field[0], 0.0, "the corner outside the shape stays dark");
        assert_eq!(
            field[128 * size + 128],
            0.0,
            "the deep interior is out of the rim's reach"
        );
        let top = field[18 * size + 128];
        let bottom = field[237 * size + 128];
        assert!(top > 0.0 && bottom > 0.0);
        assert!(top > bottom, "the lit side is stronger: {top} vs {bottom}");
        assert!(
            top > field[26 * size + 128],
            "the rim decays inwards from the boundary"
        );
    }
}
