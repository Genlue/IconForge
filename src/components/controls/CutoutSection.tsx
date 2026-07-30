import type { AutoCutoutConfig } from "../../types/domain";
import { RangeField } from "./RangeField";

export function CutoutSection(props: {
  value: AutoCutoutConfig;
  onChange(patch: Partial<AutoCutoutConfig>): void;
}): JSX.Element {
  const { value, onChange } = props;
  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold text-[var(--text-primary)]">自动抠图</h3>
        <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
          <input type="checkbox" checked={value.enabled} onChange={(e) => onChange({ enabled: e.target.checked })} />
          启用
        </label>
      </div>
      <div className={value.enabled ? "space-y-3" : "pointer-events-none space-y-3 opacity-40"}>
        <RangeField id="cutout-tolerance" label="背景容差" value={value.tolerance} min={0} max={100} step={1} onChange={(tolerance) => onChange({ tolerance })} />
        <RangeField id="cutout-feather" label="边缘羽化" value={value.feather} min={0} max={32} step={1} onChange={(feather) => onChange({ feather })} />
      </div>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">从图像边界识别连通背景，适合白底、纯色底和近纯色底图标。</p>
    </section>
  );
}
