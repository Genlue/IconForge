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
        // A full-canvas gray source: the foreground fills the shape mask, so
        // the gloss has real icon pixels to light along the edge band.
        let source = RgbaImage::from_pixel(256, 256, image::Rgba([100, 100, 100, 255]));
        render_master(&source, &validate_config(&gloss_config()).unwrap()).unwrap()
    }

    fn sample(img: &RgbaImage, x: u32, y: u32) -> [u8; 4] {
        img.get_pixel(x, y).0
    }

    #[test]
    fn gloss_unlit_edges_shade_down() {
        let img = render_gloss();
        // Outside the two highlight spans the base tone multiplies the
        // original pixels down: bottom edge, right edge, and the corner arcs
        // flanking the top-left highlight (bottom-left and top-right
        // roundings) all end up darker than the 100-gray source instead of
        // showing a painted ring.
        for (x, y) in [(128, 238), (238, 128), (32, 224), (214, 24)] {
            let p = sample(&img, x, y);
            assert!(
                p[0] < 90,
                "unlit edge ({x},{y}) should shade below the source gray, got {}",
                p[0]
            );
        }
    }

    #[test]
    fn gloss_top_left_highlight() {
        let img = render_gloss();
        // The top-left corner arc peaks at full strength (the screen blend
        // pushes the gray source to near white)...
        assert!(
            sample(&img, 32, 32)[0] > 250,
            "corner should peak near white, got {}",
            sample(&img, 32, 32)[0]
        );
        // ...while the straight left/top edges sit slightly below it.
        for (x, y) in [(18, 128), (128, 18)] {
            let p = sample(&img, x, y);
            assert!(
                p[0] > 200 && p[0] < 250,
                "edge ({x},{y}) should be brightened below the peak, got {}",
                p[0]
            );
        }
    }

    #[test]
    fn gloss_top_left_ends_fade_into_base() {
        let img = render_gloss();
        // Near the upper end of the bottom-left rounding (start of the gloss)
        // and the left end of the top-right rounding (end of the gloss) the
        // highlight must fall off gradually toward the shaded edge.
        let start = sample(&img, 18, 186);
        assert!(
            start[0] < 160,
            "gloss start should fade toward the shaded edge, got {}",
            start[0]
        );
        let mid_fade = sample(&img, 18, 178);
        assert!(
            mid_fade[0] > start[0] && mid_fade[0] < 235,
            "gloss start should show a gradient in between, got {} (start {})",
            mid_fade[0],
            start[0]
        );
        let end = sample(&img, 186, 18);
        assert!(
            end[0] < 160,
            "gloss end should fade toward the shaded edge, got {}",
            end[0]
        );
    }

    #[test]
    fn gloss_bottom_right_highlight_fades_in_and_out() {
        let img = render_gloss();
        // The bottom-right rounding arc brightens from the shaded bottom
        // edge...
        assert!(
            sample(&img, 188, 237)[0] < 170,
            "arc start should stay in the shaded band"
        );
        assert!(
            sample(&img, 207, 233)[0] > 180,
            "arc should brighten towards the middle"
        );
        // ...holds the same reduced level as the straight edges across the
        // middle...
        let mid = sample(&img, 223, 223);
        assert!(
            mid[0] > 200 && mid[0] < 250,
            "arc middle should match the edge level, got {}",
            mid[0]
        );
        // ...and fades back out to the shaded band at its upper side (right
        // edge end).
        assert!(
            sample(&img, 238, 188)[0] < 170,
            "arc end should fade back to the shaded band"
        );
    }

    #[test]
    fn gloss_needs_underlying_content() {
        // The gloss is a mask over the original icon, not a painted ring: with
        // a small foreground and no backplate the edge band has no icon
        // pixels, so it must stay fully transparent instead of showing a base
        // ring.
        let mut config = gloss_config();
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
        // The center keeps the original pixels untouched.
        assert_eq!(sample(&img, 128, 128), [100, 100, 100, 255]);
    }
}
