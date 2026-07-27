import { describe, it, expect } from "vitest";
import { BUILT_IN_PRESETS, getPresetById } from "../constants/presets";

describe("presets", () => {
  it("should have exactly 3 built-in presets", () => {
    expect(BUILT_IN_PRESETS.length).toBe(3);
  });

  it("should find preset by id", () => {
    const preset = getPresetById("macos-classic-rounded");
    expect(preset).toBeDefined();
    expect(preset?.name).toBe("macOS 经典圆角");
  });

  it("should return undefined for unknown id", () => {
    expect(getPresetById("nonexistent")).toBeUndefined();
  });
});
