import { describe, it, expect } from "vitest";
import { DEFAULT_RENDER_CONFIG } from "../constants/defaults";
import { BUILT_IN_PRESETS, getPresetById } from "../constants/presets";

describe("presets", () => {
  it("should include the expanded built-in preset collection", () => {
    expect(BUILT_IN_PRESETS.length).toBeGreaterThanOrEqual(6);
  });

  it("should find preset by id", () => {
    const preset = getPresetById("macos-classic-rounded");
    expect(preset).toBeDefined();
    expect(preset?.name).toBe("macOS 经典圆角");
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

  it.each(["macos-gloss-light", "macos-gloss-dark"])(
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
      });
    },
  );

  it("should return undefined for unknown id", () => {
    expect(getPresetById("nonexistent")).toBeUndefined();
  });

  it.each([
    ["hq-render-light", 0.5, 0.22],
    ["hq-render-dark", 0.9, 0.1],
  ])("hq preset %s uses the reference parameters", (id, thresh, bg) => {
    const preset = getPresetById(id);
    expect(preset).toBeDefined();
    expect(preset?.config.hqRender.enabled).toBe(true);
    expect(preset?.config.hqRender.thresh).toBe(thresh);
    expect(preset?.config.hqRender.bg).toBe(bg);
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
  });

  it("non-hq presets keep hq rendering disabled", () => {
    for (const preset of BUILT_IN_PRESETS) {
      if (preset.id.startsWith("hq-render-")) continue;
      expect(preset.config.hqRender.enabled).toBe(false);
    }
  });
});
