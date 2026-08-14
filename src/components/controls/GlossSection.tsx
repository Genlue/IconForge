import type { GlossConfig } from "../../types/domain";
import { ColorField } from "./ColorField";
import { RangeField } from "./RangeField";

export function GlossSection(props: {
  value: GlossConfig;
  onChange(patch: Partial<GlossConfig>): void;
}): JSX.Element {
  const { value, onChange } = props;
  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-xs text-[var(--text-secondary)]">启用</span>
        <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
          <input type="checkbox" checked={value.enabled} onChange={(e) => onChange({ enabled: e.target.checked })} />
          启用
        </label>
      </div>
      <div className={value.enabled ? "space-y-3" : "pointer-events-none space-y-3 opacity-40"}>
        <RangeField id="gloss-width" label="光泽宽度" value={value.width} min={1} max={16} step={0.5} onChange={(width) => onChange({ width })} />
        <RangeField id="gloss-strength" label="光泽强度" value={value.strength} min={0} max={1} step={0.05} onChange={(strength) => onChange({ strength })} />
        <RangeField id="gloss-feather" label="内边缘羽化" value={value.featherBlur} min={0} max={32} step={1} onChange={(featherBlur) => onChange({ featherBlur })} />
        <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
          羽化值使光泽向内平滑衰减，加强光泽与图标主体的过渡。
        </p>
        <ColorField id="gloss-light" label="高光" value={value.lightColor} pickTarget="glossLight" onChange={(lightColor) => onChange({ lightColor })} />
        <ColorField id="gloss-base" label="边缘基底色" value={value.baseColor} pickTarget="glossBase" onChange={(baseColor) => onChange({ baseColor })} />
      </div>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        沿边缘的一圈基底色，其上叠加方向性高光：左上高光沿左、上边缘，右下角同色高光并向末端渐隐。
      </p>
    </section>
  );
}
