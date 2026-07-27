import { IconShape, BackplateType } from "../types/domain";
import type { PresetDefinition } from "../types/domain";

const MACOS_CLASSIC_ROUNDED: PresetDefinition = {
  id: "macos-classic-rounded",
  name: "macOS 经典圆角",
  description: "白色圆角背板、轻描边和柔和底部阴影",
  config: {
    foregroundScalePercent: 78,
    foregroundOffsetX: 0,
    foregroundOffsetY: -2,
    foregroundRotationDegrees: 0,
    shape: IconShape.RoundedRectangle,
    cornerRadius: 52,
    squircleExponent: 4,
    backplateType: BackplateType.Solid,
    backplateColor: "#FFFFFFFF",
    gradientStartColor: "#FFFFFFFF",
    gradientEndColor: "#FFFFFFFF",
    gradientAngleDegrees: 90,
    outerShadow: {
      enabled: true,
      offsetX: 0,
      offsetY: 10,
      blurRadius: 22,
      spread: 0,
      color: "#0000002E",
    },
    stroke: {
      width: 1,
      color: "#0000001F",
    },
  },
};

const IOS_SQUIRCLE: PresetDefinition = {
  id: "ios-squircle",
  name: "iOS 超椭圆",
  description: "高饱和渐变超椭圆背板和紧凑前景",
  config: {
    foregroundScalePercent: 72,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    shape: IconShape.Squircle,
    cornerRadius: 56,
    squircleExponent: 4.5,
    backplateType: BackplateType.Gradient,
    backplateColor: "#6E7BFFFF",
    gradientStartColor: "#8B5CFFFF",
    gradientEndColor: "#3AA9FFFF",
    gradientAngleDegrees: 135,
    outerShadow: {
      enabled: true,
      offsetX: 0,
      offsetY: 12,
      blurRadius: 26,
      spread: 1,
      color: "#1E2A5A38",
    },
    stroke: {
      width: 1,
      color: "#FFFFFF42",
    },
  },
};

const MINIMAL_GLYPH: PresetDefinition = {
  id: "minimal-glyph",
  name: "极简纯图",
  description: "只保留原始图像，不添加背板、阴影或描边",
  config: {
    foregroundScalePercent: 100,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    shape: IconShape.Rectangle,
    cornerRadius: 0,
    squircleExponent: 4,
    backplateType: BackplateType.None,
    backplateColor: "#00000000",
    gradientStartColor: "#00000000",
    gradientEndColor: "#00000000",
    gradientAngleDegrees: 0,
    outerShadow: {
      enabled: false,
      offsetX: 0,
      offsetY: 0,
      blurRadius: 0,
      spread: 0,
      color: "#00000000",
    },
    stroke: {
      width: 0,
      color: "#00000000",
    },
  },
};

export const BUILT_IN_PRESETS: readonly PresetDefinition[] = Object.freeze([
  Object.freeze(MACOS_CLASSIC_ROUNDED),
  Object.freeze(IOS_SQUIRCLE),
  Object.freeze(MINIMAL_GLYPH),
]);

export function getPresetById(id: string): PresetDefinition | undefined {
  return BUILT_IN_PRESETS.find((p) => p.id === id);
}
