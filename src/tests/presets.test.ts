import { describe, it, expect } from "vitest";
import { DEFAULT_RENDER_CONFIG } from "../constants/defaults";
import { BUILT_IN_PRESETS, getPresetById } from "../constants/presets";

describe("presets", () => {
  it("should include the collapsed preset collection", () => {
    expect(
      BUILT_IN_PRESETS.map((p) => p.id),
    ).toEqual([
      "hq-render",
      "macos-classic-rounded",
      "macos-gloss-dark",
      "ios-squircle",
      "minimal-glyph",
    ]);
  });

  it("should find preset by id", () => {
    const preset = getPresetById("macos-classic-rounded");
    expect(preset).toBeDefined();
    expect(preset?.name).toBe("扁平圆角矩形");
    expect(preset?.config.foregroundOffsetX).toBe(0);
    expect(preset?.config.foregroundOffsetY).toBe(0);
    expect(preset?.config.canvasInset).toBe(0);
    expect(preset?.config.outerShadow.enabled).toBe(false);
    expect(preset?.config.outerShadow.offsetX).toBe(0);
    expect(preset?.config.outerShadow.offsetY).toBe(0);
    expect(preset?.config.outerShadow.blurRadius).toBe(0);
    expect(preset?.config.outerShadow.spread).toBe(0);
    expect(preset?.config.stroke.width).toBe(0);
  });

  it("should use centered foreground offsets by default", () => {
    expect(DEFAULT_RENDER_CONFIG.foregroundOffsetX).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.foregroundOffsetY).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.canvasInset).toBe(0);
    expect(DEFAULT_RENDER_CONFIG.outerShadow.enabled).toBe(false);
    expect(DEFAULT_RENDER_CONFIG.stroke.width).toBe(0);
  });

  it.each(["macos-gloss-dark"])(
    "should use the tuned macOS gloss values for %s",
    (presetId) => {
      const preset = getPresetById(presetId);
      expect(preset?.config.cornerRadius).toBe(57);
      expect(preset?.config.gloss).toEqual({
        enabled: true,
        width: 2,
        strength: 1,
        lightColor: "#FFFFFFCC",
        darkColor: "#BFBFBFCC",
        featherBlur: 0,
      });
    },
  );

  it("removed presets are gone", () => {
    expect(getPresetById("macos-gloss-light")).toBeUndefined();
    expect(getPresetById("neon-depth")).toBeUndefined();
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
