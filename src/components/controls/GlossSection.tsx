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
        <RangeField
          id="gloss-angle"
          label="光源角度"
          value={value.angleDegrees}
          min={0}
          max={360}
          step={1}
          onChange={(angleDegrees) => onChange({ angleDegrees })}
        />
        <RangeField id="gloss-width" label="边缘衰减" value={value.width} min={0.5} max={16} step={0.5} onChange={(width) => onChange({ width })} />
        <RangeField id="gloss-strength" label="光泽强度" value={value.strength} min={0} max={1} step={0.05} onChange={(strength) => onChange({ strength })} />
        <RangeField
          id="gloss-top"
          label="受光侧亮度"
          value={value.topStrength}
          min={0}
          max={1}
          step={0.01}
          onChange={(topStrength) => onChange({ topStrength })}
        />
        <RangeField
          id="gloss-bottom"
          label="背光侧亮度"
          value={value.bottomStrength}
          min={0}
          max={1}
          step={0.01}
          onChange={(bottomStrength) => onChange({ bottomStrength })}
        />
        <RangeField
          id="gloss-top-decay"
          label="受光侧延伸"
          value={value.topDecay}
          min={2}
          max={128}
          step={1}
          onChange={(topDecay) => onChange({ topDecay })}
        />
        <RangeField
          id="gloss-bottom-decay"
          label="背光侧延伸"
          value={value.bottomDecay}
          min={2}
          max={128}
          step={1}
          onChange={(bottomDecay) => onChange({ bottomDecay })}
        />
        <ColorField id="gloss-light" label="高光" value={value.lightColor} pickTarget="glossLight" onChange={(lightColor) => onChange({ lightColor })} />
      </div>
      <p className="hint">
        光泽只跟随图标外形，沿边缘向内指数衰减；同时沿一条可旋转的光轴叠加受光侧与背光侧两道柔光。0° 表示光从正上方照下，数值越大光源顺时针旋转。原图内容始终可见，不额外涂色。
      </p>
    </div>
  );
}
