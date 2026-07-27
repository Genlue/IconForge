import type { StrokeConfig } from "../../types/domain";
import { RangeField } from "./RangeField";
import { ColorField } from "./ColorField";

export interface StrokeSectionProps {
  value: StrokeConfig;
  onChange(patch: Partial<StrokeConfig>): void;
}

export function StrokeSection(props: StrokeSectionProps): JSX.Element {
  const { value, onChange } = props;
  const showColor = value.width > 0;

  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">描边</h3>
      <RangeField
        id="stroke-width"
        label="宽度"
        value={value.width}
        min={0}
        max={32}
        step={1}
        onChange={(v) => onChange({ width: v })}
      />
      <div className={showColor ? "" : "pointer-events-none opacity-40"}>
        <ColorField
          id="stroke-color"
          label="描边颜色"
          value={value.color}
          disabled={!showColor}
          onChange={(v) => onChange({ color: v })}
        />
      </div>
    </section>
  );
}
