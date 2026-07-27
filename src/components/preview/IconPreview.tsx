import { useMemo } from "react";
import type { InputItem, RenderConfig } from "../../types/domain";
import { IconShape, BackplateType } from "../../types/domain";
import { BASE_SHAPE_INSET } from "../../constants/defaults";
import { ShapeSvgDefs } from "./ShapeSvgDefs";

export interface IconPreviewProps {
  item: InputItem;
  config: RenderConfig;
  sizePx: number;
}

export function IconPreview(props: IconPreviewProps): JSX.Element {
  const { item, config, sizePx } = props;
  const ratio = sizePx / 256;

  const cssVars = useMemo(() => {
    const shapeInset = BASE_SHAPE_INSET * ratio;
    return {
      "--shape-inset": `${shapeInset}px`,
      "--fg-offset-x": `${config.foregroundOffsetX * ratio}px`,
      "--fg-offset-y": `${config.foregroundOffsetY * ratio}px`,
      "--fg-scale": String(config.foregroundScalePercent / 100),
      "--fg-rotation": `${config.foregroundRotationDegrees}deg`,
      "--shadow-x": `${config.outerShadow.offsetX * ratio}px`,
      "--shadow-y": `${config.outerShadow.offsetY * ratio}px`,
      "--shadow-blur": `${config.outerShadow.blurRadius * ratio}px`,
      "--backplate-color": config.backplateColor,
      "--gradient-angle": `${config.gradientAngleDegrees}deg`,
      "--gradient-start": config.gradientStartColor,
      "--gradient-end": config.gradientEndColor,
      "--shadow-color": config.outerShadow.color,
      "--stroke-width": `${config.stroke.width * ratio}px`,
      "--stroke-color": config.stroke.color,
      "--corner-radius": `${config.cornerRadius * ratio}px`,
    } as React.CSSProperties;
  }, [config, ratio]);

  const thumbSrc = useMemo(
    () => `data:image/png;base64,${item.thumbnailPngBase64}`,
    [item.thumbnailPngBase64],
  );

  // Shape clip-path
  const shapeClip = useMemo(() => {
    const inset = BASE_SHAPE_INSET * ratio;
    if (config.shape === IconShape.RoundedRectangle) {
      const r = Math.min(config.cornerRadius * ratio, (sizePx - 2 * inset) / 2);
      return `inset(${inset}px round ${r}px)`;
    }
    if (config.shape === IconShape.Squircle) {
      return `url(#squircle-clip)`;
    }
    return `inset(${inset}px)`;
  }, [config.shape, config.cornerRadius, ratio, sizePx]);

  // Squircle SVG clipPath
  const useSvgClip = config.shape === IconShape.Squircle;

  // Backplate style
  const backplateStyle = useMemo(() => {
    const base: React.CSSProperties = {
      position: "absolute",
      inset: `var(--shape-inset)`,
      overflow: "hidden",
      clipPath: shapeClip,
    };
    if (config.backplateType === BackplateType.None) {
      base.background = "transparent";
    } else if (config.backplateType === BackplateType.Solid) {
      base.background = "var(--backplate-color)";
    } else if (config.backplateType === BackplateType.Gradient) {
      base.background = `linear-gradient(calc(90deg + var(--gradient-angle)), var(--gradient-start), var(--gradient-end))`;
    }
    return base;
  }, [config.backplateType, shapeClip]);

  return (
    <div
      className="relative"
      style={{
        width: sizePx,
        height: sizePx,
        aspectRatio: "1 / 1",
        isolation: "isolate",
        ...cssVars,
      }}
    >
      {useSvgClip && (
        <ShapeSvgDefs
          shape={config.shape}
          cornerRadius={config.cornerRadius}
          squircleExponent={config.squircleExponent}
          idPrefix="icon-preview"
        />
      )}

      {/* Shadow layer */}
      {config.outerShadow.enabled && (
        <div
          style={{
            position: "absolute",
            inset: `var(--shape-inset)`,
            background: "var(--shadow-color)",
            clipPath: shapeClip,
            transform: `translate(var(--shadow-x), var(--shadow-y))`,
            filter: `blur(var(--shadow-blur))`,
          }}
        />
      )}

      {/* Backplate layer */}
      <div style={backplateStyle} />

      {/* Foreground layer */}
      <div
        className="foreground-layer"
        style={{
          position: "absolute",
          overflow: "hidden",
          display: "grid",
          placeItems: "center",
          clipPath: shapeClip,
          inset: `var(--shape-inset)`,
        }}
      >
        <img
          src={thumbSrc}
          alt={item.displayName}
          style={{
            width: "100%",
            height: "100%",
            objectFit: "contain",
            transform: `
              translate(var(--fg-offset-x), var(--fg-offset-y))
              rotate(var(--fg-rotation))
              scale(var(--fg-scale))
            `,
            transformOrigin: "center",
          }}
          draggable={false}
        />
      </div>

      {/* Stroke layer */}
      {config.stroke.width > 0 && (
        <div
          style={{
            position: "absolute",
            inset: `var(--shape-inset)`,
            pointerEvents: "none",
            border: `${config.stroke.width * ratio}px solid var(--stroke-color)`,
            borderRadius:
              config.shape === IconShape.RoundedRectangle
                ? `${Math.min(config.cornerRadius * ratio, (sizePx - 2 * BASE_SHAPE_INSET * ratio) / 2)}px`
                : undefined,
            clipPath: config.shape === IconShape.Squircle ? shapeClip : undefined,
          }}
        />
      )}
    </div>
  );
}
