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
});
