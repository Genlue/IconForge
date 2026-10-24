import type { AutoCutoutConfig } from "../../types/domain";
import { RangeField } from "./RangeField";

export function CutoutSection(props: {
  value: AutoCutoutConfig;
  onChange(patch: Partial<AutoCutoutConfig>): void;
}): JSX.Element {
  const { value, onChange } = props;
  return (
    <div className="flex flex-col gap-2.5">
      <div className="field">
        <span className="field__label">自动抠图</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={value.enabled}
            aria-label="启用自动抠图"
            onChange={(e) => onChange({ enabled: e.target.checked })}
          />
        </div>
      </div>
      <div className={`flex flex-col gap-2.5${value.enabled ? "" : " pointer-events-none opacity-40"}`}>
        <RangeField id="cutout-tolerance" label="背景容差" value={value.tolerance} min={0} max={100} step={1} onChange={(tolerance) => onChange({ tolerance })} />
        <RangeField id="cutout-feather" label="边缘羽化" value={value.feather} min={0} max={32} step={1} onChange={(feather) => onChange({ feather })} />
      </div>
      <p className="hint">从图像边界识别连通背景，适合白底、纯色底和近纯色底图标。</p>
    </div>
  );
}
