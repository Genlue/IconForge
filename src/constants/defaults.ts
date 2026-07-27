import { IconShape, BackplateType } from "../types/domain";
import type { RenderConfig } from "../types/domain";

export const BASE_SHAPE_INSET = 16;

export const DEFAULT_RENDER_CONFIG: Readonly<RenderConfig> = {
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
};
