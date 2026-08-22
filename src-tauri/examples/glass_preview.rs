//! Render a PNG through the glass renderer and save the result.
//! Usage: cargo run --example glass_preview -- "素材/QQ.png" out.png [icon_scale]
use iconforge_lib::domain::config::*;
use iconforge_lib::renderer::{render_master, validate_config};

fn glass_config() -> RenderConfig {
    RenderConfig {
        foreground_scale_percent: 100.0,
        foreground_fit: ForegroundFit::Contain,
        foreground_offset_x: 0.0,
        foreground_offset_y: 0.0,
        foreground_rotation_degrees: 0.0,
        foreground_opacity_percent: 100.0,
        canvas_inset: 0.0,
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
            offset_y: 24.0,
            blur_radius: 26.0,
            spread: 0.0,
            color: "#00000073".into(),
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
            color_retention: 0.08,
            bevel_radius: 0.12,
            normal_strength: 1.0,
            specular_strength: 0.4,
            fresnel_strength: 0.6,
            ao_strength: 0.55,
            contrast_strength: 0.0,
        },
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: glass_preview <input.png> [output.png]");
        std::process::exit(1);
    }
    let input = &args[1];
    let output = args.get(2).cloned().unwrap_or_else(|| "glass_preview.png".into());

    let bytes = std::fs::read(input).expect("cannot read input");
    let source = image::load_from_memory(&bytes)
        .expect("cannot decode input (PNG/WebP expected)")
        .to_rgba8();
    let validated = validate_config(&glass_config()).expect("invalid config");
    let rendered = render_master(&source, &validated).expect("render failed");
    rendered.save(&output).expect("cannot save output");
    println!("saved {output} ({}x{})", rendered.width(), rendered.height());
}