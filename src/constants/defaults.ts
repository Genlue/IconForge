import { IconShape, BackplateType, ForegroundFit, HqPanelShape } from "../types/domain";
import type { HqRenderConfig, RenderConfig, UpscaleConfig } from "../types/domain";

export const DEFAULT_HQ_RENDER_CONFIG: Readonly<HqRenderConfig> = {
  enabled: false,
  thresh: 0.5,
  iconRatio: 0.66,
  bg: 0.1,
  lightMix: 0.9,
  darkMix: 0.72,
  gloss: 0.16,
  iconLight: 0.08,
  corner: 0.22,
  shape: HqPanelShape.Rect,
  offsetX: 0,
  offsetY: 0,
  shadowOpacity: 0.22,
  shadowBlurFactor: 0.022,
  shadowOffsetFactor: 0.012,
  shadowFade: 0.25,
  shadowMode: "icon",
};

export const DEFAULT_RENDER_CONFIG: Readonly<RenderConfig> = {
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
  gloss: {
    enabled: false,
    width: 3,
    strength: 0.7,
    lightColor: "#FFFFFFFF",
    darkColor: "#00000080",
    featherBlur: 0,
  },
  autoCutout: {
    enabled: false,
    tolerance: 20,
    feather: 8,
  },
  hqRender: { ...DEFAULT_HQ_RENDER_CONFIG },
};

export const DEFAULT_UPSCALE_CONFIG: Readonly<UpscaleConfig> = {
  enabled: false,
  scale: 2,
  denoise: 1,
  model: "se",
};
