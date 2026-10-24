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
    <div className="flex flex-col gap-2.5">
      <div className="field">
        <span className="field__label">启用阴影</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={value.enabled}
            aria-label="启用阴影"
            onChange={(e) => onChange({ enabled: e.target.checked })}
          />
        </div>
      </div>
      <p className="hint">
        外阴影仍受 256×256 ICO 画布限制；阴影被裁切时，请在“形状”中增加画布边距。
      </p>
      <div className={`flex flex-col gap-2.5${value.enabled ? "" : " pointer-events-none opacity-40"}`}>
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
          pickTarget="shadow"
          onChange={(v) => onChange({ color: v })}
        />
      </div>
    </div>
  );
}
