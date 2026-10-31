import { describe, it, expect } from "vitest";
import { DEFAULT_RENDER_CONFIG } from "../constants/defaults";
import { BUILT_IN_PRESETS, DEFAULT_PRESET_ID, getPresetById } from "../constants/presets";

describe("presets", () => {
  it("should expose the three built-in presets", () => {
    expect(
      BUILT_IN_PRESETS.map((p) => p.id),
    ).toEqual(["macos27", "hq-render", "blank"]);
    expect(DEFAULT_PRESET_ID).toBe("macos27");
    expect(getPresetById(DEFAULT_PRESET_ID)).toBeDefined();
  });

  it("macOS27 preset keeps the tuned squircle and rim light", () => {
    const preset = getPresetById("macos27");
    expect(preset).toBeDefined();
    expect(preset?.name).toBe("macOS27 风格");
    expect(preset?.config.shape).toBe("Squircle");
    expect(preset?.config.cornerRadius).toBe(56);
    expect(preset?.config.squircleExponent).toBe(4.5);
    expect(preset?.config.backplateType).toBe("None");
    expect(preset?.config.canvasInset).toBe(0);
    expect(preset?.config.gloss).toEqual({
      enabled: true,
      width: 1,
      strength: 1,
      lightColor: "#FFFFFFFF",
      topStrength: 0.8,
      bottomStrength: 0.6,
      topDecay: 25,
      bottomDecay: 10,
      angleDegrees: 0,
    });
  });

  it("blank preset turns everything off", () => {
    const preset = getPresetById("blank");
    expect(preset).toBeDefined();
    expect(preset?.name).toBe("空白");
    expect(preset?.config.shape).toBe("Rectangle");
    expect(preset?.config.cornerRadius).toBe(0);
    expect(preset?.config.backplateType).toBe("None");
    expect(preset?.config.outerShadow.enabled).toBe(false);
    expect(preset?.config.stroke.width).toBe(0);
    expect(preset?.config.gloss.enabled).toBe(false);
    expect(preset?.config.hqRender.enabled).toBe(false);
    expect(preset?.config.autoCutout.enabled).toBe(false);
    expect(preset?.config.sourceSaturationPercent).toBe(100);
    expect(preset?.config.foregroundScalePercent).toBe(100);
    expect(preset?.config.foregroundOpacityPercent).toBe(100);
    expect(preset?.config.canvasInset).toBe(0);
    expect(preset?.config.contentScalePercent).toBe(100);
  });

  it("should use centered foreground offsets by default", () => {
    expect(DEFAULT_RENDER_CONFIG.foregroundOffsetX).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.foregroundOffsetY).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.canvasInset).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.outerShadow.enabled).toBe(false);
    expect(DEFAULT_RENDER_CONFIG.stroke.width).toBe(0);
  });

  it("removed presets are gone", () => {
    for (const id of [
      "macos-classic-rounded",
      "macos-gloss-dark",
      "ios-squircle",
      "minimal-glyph",
      "glass",
      "macos-gloss-light",
      "neon-depth",
    ]) {
      expect(getPresetById(id)).toBeUndefined();
    }
  });

  it("should return undefined for unknown id", () => {
    expect(getPresetById("nonexistent")).toBeUndefined();
  });

  it("hq preset uses the reference parameters", () => {
    const preset = getPresetById("hq-render");
    expect(preset).toBeDefined();
    expect(preset?.config.hqRender.enabled).toBe(true);
    expect(preset?.config.hqRender.thresh).toBe(0.5);
    expect(preset?.config.hqRender.bg).toBe(0.22);
    expect(preset?.config.hqRender.iconRatio).toBe(0.66);
    expect(preset?.config.hqRender.lightMix).toBe(0.9);
    expect(preset?.config.hqRender.darkMix).toBe(0.72);
    expect(preset?.config.hqRender.gloss).toBe(0.16);
    expect(preset?.config.hqRender.iconLight).toBe(0.08);
    expect(preset?.config.hqRender.corner).toBe(0.22);
    expect(preset?.config.hqRender.shadowOpacity).toBe(0.22);
    expect(preset?.config.hqRender.shadowBlurFactor).toBe(0.022);
    expect(preset?.config.hqRender.shadowOffsetFactor).toBe(0.012);
    expect(preset?.config.hqRender.shadowFade).toBe(0.25);
    expect(preset?.config.hqRender.shadowMode).toBe("icon");
    expect(preset?.config.hqRender.offsetX).toBe(0);
    expect(preset?.config.hqRender.offsetY).toBe(0);
    expect(preset?.config.hqRender.customBgEnabled).toBe(false);
  });

  it("non-hq presets keep hq rendering disabled", () => {
    for (const preset of BUILT_IN_PRESETS) {
      if (preset.id === "hq-render") continue;
      expect(preset.config.hqRender.enabled).toBe(false);
    }
  });
});
