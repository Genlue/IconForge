import type { OuterShadowConfig } from "../../types/domain";
import { RangeField } from "./RangeField";
import { ColorField } from "./ColorField";

export interface ShadowSectionProps {
  value: OuterShadowConfig;
  onChange(patch: Partial<OuterShadowConfig>): void;
}

export function ShadowSection(props: ShadowSectionProps): JSX.Element {
  const { value, onChange } = props;
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">外阴影</h3>
      <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
        <input
          type="checkbox"
          checked={value.enabled}
          onChange={(e) => onChange({ enabled: e.target.checked })}
          className="rounded"
        />
        启用阴影
      </label>
      <div className={value.enabled ? "" : "pointer-events-none opacity-40"}>
        <RangeField
          id="shadow-x"
          label="偏移 X"
          value={value.offsetX}
          min={-64}
          max={64}
          step={1}
          onChange={(v) => onChange({ offsetX: v })}
        />
        <RangeField
          id="shadow-y"
          label="偏移 Y"
          value={value.offsetY}
          min={-64}
          max={64}
          step={1}
          onChange={(v) => onChange({ offsetY: v })}
        />
        <RangeField
          id="shadow-blur"
          label="模糊"
          value={value.blurRadius}
          min={0}
          max={64}
          step={1}
          onChange={(v) => onChange({ blurRadius: v })}
        />
        <RangeField
          id="shadow-spread"
          label="扩散"
          value={value.spread}
          min={0}
          max={32}
          step={1}
          onChange={(v) => onChange({ spread: v })}
        />
        <ColorField
          id="shadow-color"
          label="阴影颜色"
          value={value.color}
          onChange={(v) => onChange({ color: v })}
        />
      </div>
    </section>
  );
}
