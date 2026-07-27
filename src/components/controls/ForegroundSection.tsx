import type { RenderConfig } from "../../types/domain";
import { RangeField } from "./RangeField";

export interface ForegroundSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

export function ForegroundSection(props: ForegroundSectionProps): JSX.Element {
  const { config, onChange } = props;
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">前景</h3>
      <RangeField
        id="fg-scale"
        label="缩放"
        value={config.foregroundScalePercent}
        min={10}
        max={100}
        step={1}
        unit="%"
        onChange={(v) => onChange({ foregroundScalePercent: v })}
      />
      <RangeField
        id="fg-offset-x"
        label="偏移 X"
        value={config.foregroundOffsetX}
        min={-128}
        max={128}
        step={1}
        onChange={(v) => onChange({ foregroundOffsetX: v })}
      />
      <RangeField
        id="fg-offset-y"
        label="偏移 Y"
        value={config.foregroundOffsetY}
        min={-128}
        max={128}
        step={1}
        onChange={(v) => onChange({ foregroundOffsetY: v })}
      />
      <RangeField
        id="fg-rotate"
        label="旋转"
        value={config.foregroundRotationDegrees}
        min={-180}
        max={180}
        step={1}
        unit="°"
        onChange={(v) => onChange({ foregroundRotationDegrees: v })}
      />
    </section>
  );
}
