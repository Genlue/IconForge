import { IconShape, BackplateType, ForegroundFit } from "../types/domain";
import type { HqRenderConfig, PresetDefinition } from "../types/domain";
import { DEFAULT_HQ_RENDER_CONFIG, DEFAULT_RENDER_CONFIG } from "./defaults";

function hqConfig(patch: Partial<HqRenderConfig>): HqRenderConfig {
  return { ...DEFAULT_HQ_RENDER_CONFIG, ...patch };
}

const MACOS_CLASSIC_ROUNDED: PresetDefinition = {
  id: "macos-classic-rounded",
  name: "扁平圆角矩形",
  description: "白色圆角背板，无边距、描边和阴影",
  config: {
    foregroundScalePercent: 100,
    foregroundFit: ForegroundFit.Contain,
    foregroundOffsetX: 0,
    foregroundOffsetY: 0,
    foregroundRotationDegrees: 0,
    canvasInset: 0,
    contentScalePercent: 100,
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
    gloss: { enabled: false, width: 3, strength: 0.7, lightColor: "#FFFFFFFF", baseColor: "#808080FF", featherBlur: 0 },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
    hqRender: hqConfig({}),
  },
};

const HQ_RENDER: PresetDefinition = {
  id: "hq-render",
  name: "高质量渲染",
  description: "自适应面板：径向光晕、顶部光泽与柔和阴影，按图标亮度自动判定深浅",
  config: {
    ...DEFAULT_RENDER_CONFIG,
    hqRender: hqConfig({ enabled: true, thresh: 0.5, bg: 0.22, iconRatio: 0.66 }),
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
    contentScalePercent: 100,
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
    gloss: { enabled: true, width: 3, strength: 0.55, lightColor: "#FFFFFFB0", baseColor: "#808080FF", featherBlur: 0 },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
    hqRender: hqConfig({}),
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
    contentScalePercent: 100,
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
    gloss: { enabled: false, width: 3, strength: 0.7, lightColor: "#FFFFFFFF", baseColor: "#808080FF", featherBlur: 0 },
    autoCutout: { enabled: false, tolerance: 20, feather: 8 },
    hqRender: hqConfig({}),
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
    gloss: { enabled: true, width: 2, strength: 1, lightColor: "#C8C8C8CC", baseColor: "#454545FF", featherBlur: 0 },
  },
};

export const BUILT_IN_PRESETS: readonly PresetDefinition[] = Object.freeze([
  Object.freeze(HQ_RENDER),
  Object.freeze(MACOS_CLASSIC_ROUNDED),
  Object.freeze(MACOS_GLOSS_DARK),
  Object.freeze(IOS_SQUIRCLE),
  Object.freeze(MINIMAL_GLYPH),
]);

export function getPresetById(id: string): PresetDefinition | undefined {
  return BUILT_IN_PRESETS.find((p) => p.id === id);
}
