import { IconShape } from "../../types/domain";

export interface ShapeSvgDefsProps {
  shape: IconShape;
  cornerRadius: number;
  squircleExponent: number;
  idPrefix: string;
}

export function ShapeSvgDefs(props: ShapeSvgDefsProps): JSX.Element {
  return (
    <svg width="0" height="0" className="absolute">
      <defs>
        <clipPath id={`${props.idPrefix}-shape`}>
          <rect x="0" y="0" width="1" height="1" />
        </clipPath>
      </defs>
    </svg>
  );
}
