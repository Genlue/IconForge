//! 默认渲染配置与内置预设。
//!
//! 数值必须与宿主渲染器的语义对齐：颜色统一 `#RRGGBBAA`（渲染端只认 8 位），
//! 角度单位为度，比例类字段用 0..=1 的小数。

use crate::types::*;

pub const DEFAULT_PRESET_ID: &str = "macos27";

pub fn default_hq_render() -> HqRenderConfig {
    HqRenderConfig {
        enabled: false,
        thresh: 0.5,
        icon_ratio: 0.66,
        bg: 0.1,
        light_mix: 0.9,
        dark_mix: 0.72,
        gloss: 0.16,
        icon_light: 0.08,
        corner: 0.22,
        shape: HqPanelShape::Rect,
        offset_x: 0.0,
        offset_y: 0.0,
        custom_bg_enabled: false,
        custom_bg_color: "#FFFFFFFF".to_string(),
        shadow_opacity: 0.22,
        shadow_blur_factor: 0.022,
        shadow_offset_factor: 0.012,
        shadow_fade: 0.25,
        shadow_mode: HqShadowMode::Icon,
    }
}

pub fn default_gloss() -> GlossConfig {
    GlossConfig {
        enabled: false,
        width: 3.5,
        strength: 1.0,
        light_color: "#FFFFFFFF".to_string(),
        top_strength: 0.42,
        bottom_strength: 0.35,
        top_decay: 38.0,
        bottom_decay: 18.0,
        angle_degrees: 0.0,
    }
}

pub fn default_render_config() -> RenderConfig {
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
        backplate_type: BackplateType::Solid,
        backplate_color: "#FFFFFFFF".to_string(),
        gradient_start_color: "#FFFFFFFF".to_string(),
        gradient_end_color: "#FFFFFFFF".to_string(),
        gradient_angle_degrees: 90.0,
        outer_shadow: OuterShadowConfig {
            enabled: false,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread: 0.0,
            color: "#00000000".to_string(),
        },
        stroke: StrokeConfig {
            width: 0.0,
            color: "#00000000".to_string(),
        },
        gloss: default_gloss(),
        source_saturation_percent: 100.0,
        auto_cutout: AutoCutoutConfig {
            enabled: false,
            tolerance: 20.0,
            feather: 8.0,
        },
        hq_render: default_hq_render(),
    }
}

pub fn default_upscale_config() -> UpscaleConfig {
    UpscaleConfig {
        enabled: false,
        scale: 2,
        denoise: 1,
        model: "se".to_string(),
    }
}

pub fn default_item_config() -> ItemConfig {
    let preset = built_in_presets()
        .into_iter()
        .find(|p| p.id == DEFAULT_PRESET_ID);
    ItemConfig {
        render: preset
            .as_ref()
            .map(|p| p.config.clone())
            .unwrap_or_else(default_render_config),
        upscale: default_upscale_config(),
        brush_strokes: Vec::new(),
        brush_color: "#FF3B30FF".to_string(),
        brush_size: 8.0,
        brush_mode: BrushMode::Paint,
        brush_clip_to_mask: true,
        wand_strokes: Vec::new(),
        wand_tolerance: 72.0,
        eraser_strokes: Vec::new(),
        eraser_size: 16.0,
        eraser_hardness: 50.0,
        active_preset: preset.map(|p| p.id),
    }
}

fn macos27() -> Preset {
    let mut config = default_render_config();
    config.foreground_fit = ForegroundFit::Cover;
    config.shape = IconShape::Squircle;
    config.corner_radius = 56.0;
    config.squircle_exponent = 4.5;
    config.backplate_type = BackplateType::None;
    config.gradient_start_color = "#8B5CFFFF".to_string();
    config.gradient_end_color = "#3AA9FFFF".to_string();
    config.gradient_angle_degrees = 135.0;
    config.outer_shadow = OuterShadowConfig {
        enabled: false,
        offset_x: 0.0,
        offset_y: 12.0,
        blur_radius: 26.0,
        spread: 1.0,
        color: "#1E2A5A38".to_string(),
    };
    config.stroke = StrokeConfig {
        width: 0.0,
        color: "#FFFFFF42".to_string(),
    };
    config.gloss = GlossConfig {
        enabled: true,
        width: 1.0,
        strength: 1.0,
        light_color: "#FFFFFFFF".to_string(),
        top_strength: 0.8,
        bottom_strength: 0.6,
        top_decay: 25.0,
        bottom_decay: 10.0,
        angle_degrees: 0.0,
    };
    config.source_saturation_percent = 100.0;
    Preset {
        id: DEFAULT_PRESET_ID.to_string(),
        name: "macOS27 风格".to_string(),
        description: "超椭圆裁剪、无背板，配一道窄边柔光的边缘光泽".to_string(),
        config,
        custom: false,
    }
}

fn hq_render() -> Preset {
    let mut config = default_render_config();
    config.hq_render = HqRenderConfig {
        enabled: true,
        bg: 0.22,
        ..default_hq_render()
    };
    Preset {
        id: "hq-render".to_string(),
        name: "高质量渲染".to_string(),
        description: "自适应面板：径向光晕、顶部光泽与柔和阴影，按图标亮度自动判定深浅"
            .to_string(),
        config,
        custom: false,
    }
}

fn blank() -> Preset {
    let mut config = default_render_config();
    config.shape = IconShape::Rectangle;
    config.corner_radius = 0.0;
    config.backplate_type = BackplateType::None;
    config.backplate_color = "#00000000".to_string();
    config.gradient_start_color = "#00000000".to_string();
    config.gradient_end_color = "#00000000".to_string();
    config.gradient_angle_degrees = 0.0;
    config.outer_shadow = OuterShadowConfig {
        enabled: false,
        offset_x: 0.0,
        offset_y: 0.0,
        blur_radius: 0.0,
        spread: 0.0,
        color: "#00000000".to_string(),
    };
    config.stroke = StrokeConfig {
        width: 0.0,
        color: "#00000000".to_string(),
    };
    config.gloss = default_gloss();
    config.hq_render = default_hq_render();
    config.source_saturation_percent = 100.0;
    Preset {
        id: "blank".to_string(),
        name: "空白".to_string(),
        description: "什么都不改：无背板、圆角、描边、阴影、光泽与画质增强".to_string(),
        config,
        custom: false,
    }
}

pub fn built_in_presets() -> Vec<Preset> {
    vec![macos27(), hq_render(), blank()]
}
