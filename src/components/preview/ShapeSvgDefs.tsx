import { useMemo } from "react";
import { IconShape } from "../../types/domain";
import { sampleSquirclePath, pointsToSvgPath } from "../../lib/squircle";
import { BASE_SHAPE_INSET } from "../../constants/defaults";

export interface ShapeSvgDefsProps {
  shape: IconShape;
  cornerRadius: number;
  squircleExponent: number;
  idPrefix: string;
}

export function ShapeSvgDefs(props: ShapeSvgDefsProps): JSX.Element {
  const { shape, squircleExponent, idPrefix } = props;

  const squirclePath = useMemo(() => {
    if (shape !== IconShape.Squircle) return "";
    const pts = sampleSquirclePath(squircleExponent, 256);
    // Normalize from [-1,1] to [inset, 1-inset] in objectBoundingBox coords
    const inset = BASE_SHAPE_INSET / 256;
    const scale = 1 - 2 * inset;
    const normalized = pts.map((p) => ({
      x: (p.x * scale + 1) / 2,
      y: (p.y * scale + 1) / 2,
    }));
    return pointsToSvgPath(normalized);
  }, [shape, squircleExponent]);

  if (shape !== IconShape.Squircle) {
    return <svg width="0" height="0" className="absolute"><defs /></svg>;
  }

  return (
    <svg width="0" height="0" className="absolute">
      <defs>
        <clipPath id={`${idPrefix}-shape`} clipPathUnits="objectBoundingBox">
          <path d={squirclePath} />
        </clipPath>
      </defs>
    </svg>
  );
}
