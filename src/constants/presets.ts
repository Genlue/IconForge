import { IconShape, BackplateType, ForegroundFit } from "../types/domain";
import type { PresetDefinition } from "../types/domain";

const MACOS_CLASSIC_ROUNDED: PresetDefinition = {
  id: "macos-classic-rounded",
  name: "macOS 经典圆角",
  description: "白色圆角背板，无边距、描边和阴影",
  config: {
    foregroundScalePercent: 100,
    foregroundFit: ForegroundFit.Contain,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    canvasInset: 0,
    shape: IconShape.RoundedRectangle,
    cornerRadius: 52,
    squircleExponent: 4,
    backplateType: BackplateType.Solid,
    backplateColor: "#FFFFFFFF",
    gradientStartColor: "#FFFFFFFF",
    gradientEndColor: "#FFFFFFFF",
    gradientAngleDegrees: 90,
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
    gloss: { enabled: false, width: 3, strength: 0.7, lightColor: "#FFFFFFFF", darkColor: "#00000080" },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
  },
};

const IOS_SQUIRCLE: PresetDefinition = {
  id: "ios-squircle",
  name: "iOS 超椭圆",
  description: "高饱和渐变超椭圆背板和紧凑前景",
  config: {
    foregroundScalePercent: 100,
    foregroundFit: ForegroundFit.Cover,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    canvasInset: 12,
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
    gloss: { enabled: true, width: 3, strength: 0.55, lightColor: "#FFFFFFB0", darkColor: "#17206070" },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
  },
};

const MINIMAL_GLYPH: PresetDefinition = {
  id: "minimal-glyph",
  name: "极简纯图",
  description: "只保留原始图像，不添加背板、阴影或描边",
  config: {
    foregroundScalePercent: 100,
    foregroundFit: ForegroundFit.Contain,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    canvasInset: 0,
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
    gloss: { enabled: false, width: 3, strength: 0.7, lightColor: "#FFFFFFFF", darkColor: "#00000080" },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
  },
};

const MACOS_GLOSS_LIGHT: PresetDefinition = {
  id: "macos-gloss-light",
  name: "macOS 光泽浅色",
  description: "明亮竖向渐变、左上高光与右下暗边",
  config: {
    ...MACOS_CLASSIC_ROUNDED.config,
    cornerRadius: 57,
    backplateType: BackplateType.Gradient,
    gradientStartColor: "#67C1FFFF",
    gradientEndColor: "#0878F5FF",
    gradientAngleDegrees: 90,
    stroke: { width: 1, color: "#FFFFFF80" },
    gloss: { enabled: true, width: 2, strength: 1, lightColor: "#FFFFFFCC", darkColor: "#BFBFBFCC" },
  },
};

const MACOS_GLOSS_DARK: PresetDefinition = {
  id: "macos-gloss-dark",
  name: "macOS 光泽深色",
  description: "炭灰至黑色渐变和金属感边缘光泽",
  config: {
    ...MACOS_CLASSIC_ROUNDED.config,
    cornerRadius: 57,
    backplateType: BackplateType.Gradient,
    gradientStartColor: "#3A3A3AFF",
    gradientEndColor: "#101010FF",
    gradientAngleDegrees: 90,
    stroke: { width: 1, color: "#FFFFFF60" },
    gloss: { enabled: true, width: 2, strength: 1, lightColor: "#FFFFFFCC", darkColor: "#BFBFBFCC" },
  },
};

const NEON_DEPTH: PresetDefinition = {
  id: "neon-depth",
  name: "霓虹纵深",
  description: "冷色渐变、内高光和柔和外阴影",
  config: {
    ...IOS_SQUIRCLE.config,
    gradientStartColor: "#22D3EEFF",
    gradientEndColor: "#4F46E5FF",
    gradientAngleDegrees: 110,
    outerShadow: { enabled: true, offsetX: 0, offsetY: 10, blurRadius: 24, spread: 2, color: "#312E8158" },
    gloss: { enabled: true, width: 3, strength: 0.75, lightColor: "#FFFFFFC0", darkColor: "#17255490" },
  },
};

export const BUILT_IN_PRESETS: readonly PresetDefinition[] = Object.freeze([
  Object.freeze(MACOS_CLASSIC_ROUNDED),
  Object.freeze(MACOS_GLOSS_LIGHT),
  Object.freeze(MACOS_GLOSS_DARK),
  Object.freeze(IOS_SQUIRCLE),
  Object.freeze(NEON_DEPTH),
  Object.freeze(MINIMAL_GLYPH),
]);

export function getPresetById(id: string): PresetDefinition | undefined {
  return BUILT_IN_PRESETS.find((p) => p.id === id);
}
