#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::validate_config;

    fn make_default_config() -> RenderConfig {
        RenderConfig {
            foreground_scale_percent: 100.0,
            foreground_fit: ForegroundFit::Contain,
            foreground_offset_x: 0.0,
            foreground_offset_y: 0.0,
            foreground_rotation_degrees: 0.0,
            canvas_inset: 16.0,
            content_scale_percent: 100.0,
            shape: IconShape::RoundedRectangle,
            corner_radius: 52.0,
            squircle_exponent: 4.0,
            backplate_type: BackplateType::Solid,
            backplate_color: "#FFFFFFFF".into(),
            gradient_start_color: "#FFFFFFFF".into(),
            gradient_end_color: "#FFFFFFFF".into(),
            gradient_angle_degrees: 90.0,
            outer_shadow: OuterShadowConfig {
                enabled: true,
                offset_x: 0.0,
                offset_y: 10.0,
                blur_radius: 22.0,
                spread: 0.0,
                color: "#0000002E".into(),
            },
            stroke: StrokeConfig {
                width: 1.0,
                color: "#0000001F".into(),
            },
            gloss: GlossConfig {
                enabled: false,
                width: 3.0,
                strength: 0.7,
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
        }
    }

    #[test]
    fn test_valid_default_config() {
        let config = make_default_config();
        let result = validate_config(&config);
        assert!(result.is_ok(), "default config should be valid");
    }

    #[test]
    fn test_invalid_color() {
        let mut config = make_default_config();
        config.backplate_color = "#GGG".into();
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_invalid_scale() {
        let mut config = make_default_config();
        config.foreground_scale_percent = 301.0;
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn test_nan_rejected() {
        let mut config = make_default_config();
        config.foreground_scale_percent = f32::NAN;
        assert!(validate_config(&config).is_err());
    }
}
