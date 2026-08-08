//! Comparison harness against the reference `iconmask.py` implementation.
//!
//! Prerequisites: run the reference script first, e.g.
//!   python iconmask.py input.png ref_light.png --size 256 --thresh 0.5 --iconratio 0.66 --bg 0.22
//!   python iconmask.py input.png ref_dark.png  --size 256 --thresh 0.9 --iconratio 0.66 --bg 0.10
//! Set `HQ_COMPARE_DIR` to the folder containing input.png / ref_light.png /
//! ref_dark.png. The test is skipped when the reference images are absent.

#[cfg(test)]
mod tests {
    use iconforge_lib::domain::config::*;
    use iconforge_lib::renderer::{render_master, validate_config};
    use image::RgbaImage;

    fn base() -> RenderConfig {
        RenderConfig {
            foreground_scale_percent: 100.0,
            foreground_fit: ForegroundFit::Contain,
            foreground_offset_x: 0.0,
            foreground_offset_y: 0.0,
            foreground_rotation_degrees: 0.0,
            canvas_inset: 0.0,
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
            },
            auto_cutout: AutoCutoutConfig {
                enabled: false,
                tolerance: 20.0,
                feather: 8.0,
            },
            hq_render: HqRenderConfig {
                enabled: true,
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

    fn compare(root: &std::path::Path, name: &str, thresh: f32, bg: f32) {
        let input_path = root.join("input.png");
        if !input_path.exists() {
            return;
        }
        let input = image::open(&input_path).unwrap().to_rgba8();
        let mut raw = base();
        raw.hq_render.thresh = thresh;
        raw.hq_render.bg = bg;
        let rendered = render_master(&input, &validate_config(&raw).unwrap()).unwrap();
        let rust_path = root.join(format!(
            "rust_{}.png",
            name.strip_suffix(".png").unwrap()
        ));
        rendered.save(&rust_path).unwrap();
        let ref_path = root.join(name);
        assert!(
            ref_path.exists(),
            "reference image {} not found; run iconmask.py first",
            ref_path.display()
        );
        let reference = image::open(&ref_path).unwrap().to_rgba8();
        assert_eq!(rendered.dimensions(), reference.dimensions());

        let mut total = [0.0f64; 4];
        let mut max_diff = [0u32; 4];
        let mut count = 0u64;
        for (a, b) in rendered.pixels().zip(reference.pixels()) {
            for c in 0..4 {
                let d = (a.0[c] as i32 - b.0[c] as i32).abs() as u32;
                total[c] += d as f64;
                max_diff[c] = max_diff[c].max(d);
            }
            count += 1;
        }
        let mean: Vec<f64> = total.iter().map(|t| t / count as f64).collect();
        // Alpha is (intentionally) where the AA mask and compositing differ most.
        println!(
            "{name}: mean={:?} max(alpha)={} max(rgb)={}",
            mean,
            max_diff[3],
            max_diff[0].max(max_diff[1]).max(max_diff[2])
        );
        // Tolerance absorbs filter differences (PIL Lanczos vs image crate,
        // BOX/BILINEAR sampling). Anything beyond that signals an algorithmic
        // regression against the reference implementation.
        assert!(
            mean[0] < 12.0 && mean[1] < 12.0 && mean[2] < 12.0 && mean[3] < 6.0,
            "mean diff too large: {mean:?}"
        );
    }

    #[test]
    fn hq_matches_reference_light() {
        let dir = std::env::var("HQ_COMPARE_DIR").unwrap_or_else(|_| {
            std::env::temp_dir()
                .join("opencode/hq_compare")
                .to_string_lossy()
                .to_string()
        });
        compare(std::path::Path::new(&dir), "ref_light.png", 0.5, 0.22);
    }

    #[test]
    fn hq_matches_reference_dark() {
        let dir = std::env::var("HQ_COMPARE_DIR").unwrap_or_else(|_| {
            std::env::temp_dir()
                .join("opencode/hq_compare")
                .to_string_lossy()
                .to_string()
        });
        compare(std::path::Path::new(&dir), "ref_dark.png", 0.9, 0.1);
    }
}