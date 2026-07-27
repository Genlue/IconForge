import type { RenderConfig } from "../../types/domain";
import { ForegroundFit } from "../../types/domain";
import { RangeField } from "./RangeField";
import { SegmentedControl } from "./SegmentedControl";

export interface ForegroundSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

export function ForegroundSection(props: ForegroundSectionProps): JSX.Element {
  const { config, onChange } = props;
  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">前景</h3>
      <SegmentedControl
        id="foreground-fit"
        value={config.foregroundFit}
        options={[
          { value: ForegroundFit.Contain, label: "适应" },
          { value: ForegroundFit.Cover, label: "填满" },
        ]}
        onChange={(v) => onChange({ foregroundFit: v })}
      />
      <RangeField
        id="fg-scale"
        label="缩放"
        value={config.foregroundScalePercent}
        min={10}
        max={300}
        step={1}
        unit="%"
        onChange={(v) => onChange({ foregroundScalePercent: v })}
      />
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        以源图的非透明有效内容为基准；“填满”会铺满形状并裁切，超过 100% 可继续放大。
      </p>
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
