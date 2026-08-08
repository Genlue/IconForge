#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::{composite_brush_strokes, render_master, validate_config};
    use image::{Rgba, RgbaImage};

    fn config() -> RenderConfig {
        RenderConfig {
            foreground_scale_percent: 100.0,
            foreground_fit: ForegroundFit::Contain,
            foreground_offset_x: 0.0,
            foreground_offset_y: 0.0,
            foreground_rotation_degrees: 0.0,
            canvas_inset: 0.0,
            content_scale_percent: 100.0,
            shape: IconShape::Rectangle,
            corner_radius: 0.0,
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
                enabled: false,
                width: 3.0,
                strength: 0.7,
                light_color: "#FFFFFFFF".into(),
                dark_color: "#00000080".into(),
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
                shadow_opacity: 0.22,
                shadow_blur_factor: 0.022,
                shadow_offset_factor: 0.012,
                shadow_fade: 0.25,
                shadow_mode: HqShadowMode::Icon,
            },
        }
    }

    fn alpha_bounds(image: &RgbaImage) -> (u32, u32, u32, u32) {
        let mut min_x = image.width();
        let mut min_y = image.height();
        let mut max_x = 0;
        let mut max_y = 0;
        for (x, y, pixel) in image.enumerate_pixels() {
            if pixel.0[3] > 8 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
        (min_x, min_y, max_x, max_y)
    }

    fn padded_portrait_source() -> RgbaImage {
        let mut source = RgbaImage::new(100, 100);
        for y in 20..80 {
            for x in 40..60 {
                source.put_pixel(x, y, Rgba([30, 180, 240, 255]));
            }
        }
        source
    }

    #[test]
    fn contain_ignores_transparent_padding_and_touches_long_axis() {
        let validated = validate_config(&config()).unwrap();
        let rendered = render_master(&padded_portrait_source(), &validated).unwrap();
        let (min_x, min_y, max_x, max_y) = alpha_bounds(&rendered);

        assert!(
            min_y <= 1 && max_y >= 254,
            "vertical content should fill the shape"
        );
        assert!(
            min_x > 70 && max_x < 185,
            "portrait aspect ratio should be preserved"
        );
    }

    #[test]
    fn cover_fills_the_entire_shape() {
        let mut raw = config();
        raw.foreground_fit = ForegroundFit::Cover;
        let rendered =
            render_master(&padded_portrait_source(), &validate_config(&raw).unwrap()).unwrap();
        let (min_x, min_y, max_x, max_y) = alpha_bounds(&rendered);

        assert!(min_x <= 1 && min_y <= 1 && max_x >= 254 && max_y >= 254);
    }

    #[test]
    fn canvas_inset_is_explicit_and_can_be_zero() {
        let source = RgbaImage::new(1, 1);
        let mut raw = config();
        raw.backplate_type = BackplateType::Solid;
        raw.backplate_color = "#FFFFFFFF".into();

        let full = render_master(&source, &validate_config(&raw).unwrap()).unwrap();
        assert_eq!(full.get_pixel(0, 0).0[3], 255);

        raw.canvas_inset = 16.0;
        let inset = render_master(&source, &validate_config(&raw).unwrap()).unwrap();
        assert_eq!(inset.get_pixel(0, 0).0[3], 0);
        assert_eq!(inset.get_pixel(128, 128).0[3], 255);
    }

    #[test]
    fn translucent_stroke_keeps_its_configured_alpha() {
        let source = RgbaImage::new(1, 1);
        let mut raw = config();
        raw.stroke.width = 8.0;
        raw.stroke.color = "#FF000080".into();
        let rendered = render_master(&source, &validate_config(&raw).unwrap()).unwrap();
        let alpha = rendered.get_pixel(2, 128).0[3];

        assert!((120..=136).contains(&alpha), "stroke alpha was {alpha}");
    }

    #[test]
    fn zero_radius_is_a_rectangle_not_a_diamond() {
        let source = RgbaImage::new(1, 1);
        let mut raw = config();
        raw.canvas_inset = 16.0;
        raw.shape = IconShape::RoundedRectangle;
        raw.corner_radius = 0.0;
        raw.backplate_type = BackplateType::Solid;
        raw.backplate_color = "#FFFFFFFF".into();

        let rendered = render_master(&source, &validate_config(&raw).unwrap()).unwrap();
        assert_eq!(rendered.get_pixel(0, 128).0[3], 0);
        assert_eq!(rendered.get_pixel(16, 128).0[3], 255);
        assert_eq!(rendered.get_pixel(128, 16).0[3], 255);
        assert_eq!(rendered.get_pixel(16, 16).0[3], 255);
    }

    #[test]
    fn rounded_rectangle_keeps_straight_edges_between_corners() {
        let source = RgbaImage::new(1, 1);
        let mut raw = config();
        raw.canvas_inset = 16.0;
        raw.shape = IconShape::RoundedRectangle;
        raw.corner_radius = 52.0;
        raw.backplate_type = BackplateType::Solid;
        raw.backplate_color = "#FFFFFFFF".into();

        let rendered = render_master(&source, &validate_config(&raw).unwrap()).unwrap();
        assert_eq!(rendered.get_pixel(0, 128).0[3], 0);
        assert_eq!(rendered.get_pixel(16, 128).0[3], 255);
        assert_eq!(rendered.get_pixel(128, 16).0[3], 255);
        assert_eq!(rendered.get_pixel(16, 16).0[3], 0);
        assert_eq!(rendered.get_pixel(68, 16).0[3], 255);
    }

    #[test]
    fn brush_strokes_are_composited_into_the_master_image() {
        let source = RgbaImage::new(1, 1);
        let rendered = render_master(&source, &validate_config(&config()).unwrap()).unwrap();
        let painted = composite_brush_strokes(
            rendered,
            &[BrushStroke {
                points: vec![
                    BrushPoint { x: 32.0, y: 128.0 },
                    BrushPoint { x: 224.0, y: 128.0 },
                ],
                color: "#FF0000FF".into(),
                size: 16.0,
                opacity: 1.0,
                mode: BrushMode::Paint,
                clip_to_mask: false,
            }],
        )
        .unwrap();

        assert!(painted.get_pixel(128, 128).0[0] > 240);
        assert!(painted.get_pixel(128, 128).0[3] > 240);
        assert_eq!(painted.get_pixel(128, 32).0[3], 0);
    }

    #[test]
    fn eraser_removes_alpha_and_clipped_paint_stays_inside() {
        let source = RgbaImage::from_pixel(256, 256, Rgba([0, 80, 200, 255]));
        let erased = composite_brush_strokes(
            source,
            &[BrushStroke {
                points: vec![BrushPoint { x: 128.0, y: 128.0 }],
                color: "#FFFFFFFF".into(),
                size: 24.0,
                opacity: 1.0,
                mode: BrushMode::Erase,
                clip_to_mask: true,
            }],
        )
        .unwrap();
        assert_eq!(erased.get_pixel(128, 128).0[3], 0);
    }
}
