import type { GlossConfig } from "../../types/domain";
import { ColorField } from "./ColorField";
import { RangeField } from "./RangeField";

export function GlossSection(props: {
  value: GlossConfig;
  onChange(patch: Partial<GlossConfig>): void;
}): JSX.Element {
  const { value, onChange } = props;
  return (
    <div className="flex flex-col gap-2.5">
      <div className="field">
        <span className="field__label">启用</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={value.enabled}
            aria-label="启用边缘光泽"
            onChange={(e) => onChange({ enabled: e.target.checked })}
          />
        </div>
      </div>
      <div className={`flex flex-col gap-2.5${value.enabled ? "" : " pointer-events-none opacity-40"}`}>
        <RangeField id="gloss-width" label="光泽宽度" value={value.width} min={1} max={16} step={0.5} onChange={(width) => onChange({ width })} />
        <RangeField id="gloss-strength" label="光泽强度" value={value.strength} min={0} max={1} step={0.05} onChange={(strength) => onChange({ strength })} />
        <RangeField id="gloss-feather" label="内边缘羽化" value={value.featherBlur} min={0} max={32} step={1} onChange={(featherBlur) => onChange({ featherBlur })} />
        <p className="hint">羽化值使光泽向内平滑衰减，加强光泽与图标主体的过渡。</p>
        <ColorField id="gloss-light" label="高光" value={value.lightColor} pickTarget="glossLight" onChange={(lightColor) => onChange({ lightColor })} />
        <ColorField id="gloss-base" label="边缘基底色" value={value.baseColor} pickTarget="glossBase" onChange={(baseColor) => onChange({ baseColor })} />
      </div>
      <p className="hint">
        光泽以蒙版方式作用于原图：边缘带作为蒙版，高光沿左、上边缘与右下角以滤色方式提亮原有像素；边缘基底色在未受光处以正片叠底轻微压暗，形成明暗对比。原图内容始终可见，不额外涂色。
      </p>
    </div>
  );
}
