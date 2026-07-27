import { IconShape, BackplateType, ForegroundFit } from "../types/domain";
import type { RenderConfig, UpscaleConfig } from "../types/domain";

export const DEFAULT_RENDER_CONFIG: Readonly<RenderConfig> = {
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
};

export const DEFAULT_UPSCALE_CONFIG: Readonly<UpscaleConfig> = {
  enabled: false,
  scale: 2,
  denoise: 1,
  model: "se",
};
