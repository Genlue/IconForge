import { IconShape, BackplateType, ForegroundFit } from "../types/domain";
import type { GlossConfig, HqRenderConfig, PresetDefinition } from "../types/domain";
import { DEFAULT_HQ_RENDER_CONFIG, DEFAULT_RENDER_CONFIG } from "./defaults";

function hqConfig(patch: Partial<HqRenderConfig>): HqRenderConfig {
  return { ...DEFAULT_HQ_RENDER_CONFIG, ...patch };
}

function glossConfig(patch: Partial<GlossConfig>): GlossConfig {
  return { ...DEFAULT_RENDER_CONFIG.gloss, ...patch };
}

/** 新导入图标默认套用的内置预设。 */
export const DEFAULT_PRESET_ID = "macos27";

const MACOS27: PresetDefinition = {
  id: "macos27",
  name: "macOS27 风格",
  description: "超椭圆裁剪、无背板，配一道窄边柔光的边缘光泽",
  config: {
    ...DEFAULT_RENDER_CONFIG,
    foregroundFit: ForegroundFit.Cover,
    shape: IconShape.Squircle,
    cornerRadius: 56,
    squircleExponent: 4.5,
    backplateType: BackplateType.None,
    gradientStartColor: "#8B5CFFFF",
    gradientEndColor: "#3AA9FFFF",
    gradientAngleDegrees: 135,
    outerShadow: {
      enabled: false,
      offsetX: 0,
      offsetY: 12,
      blurRadius: 26,
      spread: 1,
      color: "#1E2A5A38",
    },
    stroke: {
      width: 0,
      color: "#FFFFFF42",
    },
    gloss: glossConfig({
      enabled: true,
      width: 1,
      strength: 1,
      lightColor: "#FFFFFFFF",
      topStrength: 0.8,
      bottomStrength: 0.6,
      topDecay: 25,
      bottomDecay: 10,
    }),
    sourceSaturationPercent: 100,
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

/** 全关：不改变源图，只是把画面裁到画布范围内。 */
const BLANK: PresetDefinition = {
  id: "blank",
  name: "空白",
  description: "什么都不改：无背板、圆角、描边、阴影、光泽与画质增强",
  config: {
    ...DEFAULT_RENDER_CONFIG,
    shape: IconShape.Rectangle,
    cornerRadius: 0,
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
    gloss: glossConfig({ enabled: false }),
    hqRender: hqConfig({ enabled: false }),
    sourceSaturationPercent: 100,
  },
};

export const BUILT_IN_PRESETS: readonly PresetDefinition[] = Object.freeze([
  Object.freeze(MACOS27),
  Object.freeze(HQ_RENDER),
  Object.freeze(BLANK),
]);

export function getPresetById(id: string): PresetDefinition | undefined {
  return BUILT_IN_PRESETS.find((p) => p.id === id);
}
