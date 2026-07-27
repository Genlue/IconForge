import type { RenderConfig } from "../../types/domain";
import { IconShape } from "../../types/domain";
import { RangeField } from "./RangeField";
import { SegmentedControl } from "./SegmentedControl";

export interface ShapeSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

const SHAPE_OPTIONS = [
  { value: IconShape.Rectangle, label: "矩形" },
  { value: IconShape.RoundedRectangle, label: "圆角" },
  { value: IconShape.Squircle, label: "超椭圆" },
];

export function ShapeSection(props: ShapeSectionProps): JSX.Element {
  const { config, onChange } = props;
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">形状</h3>
      <SegmentedControl
        id="shape"
        value={config.shape}
        options={SHAPE_OPTIONS}
        onChange={(v) => onChange({ shape: v })}
      />
      <RangeField
        id="corner-radius"
        label="圆角半径"
        value={config.cornerRadius}
        min={0}
        max={128}
        step={1}
        disabled={config.shape !== IconShape.RoundedRectangle}
        onChange={(v) => onChange({ cornerRadius: v })}
      />
      <RangeField
        id="squircle-exp"
        label="超椭圆指数"
        value={config.squircleExponent}
        min={2}
        max={8}
        step={0.1}
        disabled={config.shape !== IconShape.Squircle}
        onChange={(v) => onChange({ squircleExponent: v })}
      />
    </section>
  );
}
