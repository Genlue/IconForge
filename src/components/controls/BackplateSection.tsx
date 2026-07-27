import type { RenderConfig } from "../../types/domain";
import { BackplateType } from "../../types/domain";
import { SegmentedControl } from "./SegmentedControl";
import { ColorField } from "./ColorField";
import { RangeField } from "./RangeField";

export interface BackplateSectionProps {
  config: RenderConfig;
  onChange(patch: Partial<RenderConfig>): void;
}

const BACKPLATE_OPTIONS = [
  { value: BackplateType.None, label: "无" },
  { value: BackplateType.Solid, label: "纯色" },
  { value: BackplateType.Gradient, label: "渐变" },
];

export function BackplateSection(props: BackplateSectionProps): JSX.Element {
  const { config, onChange } = props;

  return (
    <section className="space-y-3">
      <h3 className="text-xs font-semibold text-[var(--text-primary)]">背板</h3>
      <SegmentedControl
        id="backplate"
        value={config.backplateType}
        options={BACKPLATE_OPTIONS}
        onChange={(v) => onChange({ backplateType: v })}
      />
      {config.backplateType === BackplateType.Solid && (
        <ColorField
          id="bp-color"
          label="背板颜色"
          value={config.backplateColor}
          onChange={(v) => onChange({ backplateColor: v })}
        />
      )}
      {config.backplateType === BackplateType.Gradient && (
        <>
          <ColorField
            id="grad-start"
            label="渐变起点"
            value={config.gradientStartColor}
            onChange={(v) => onChange({ gradientStartColor: v })}
          />
          <ColorField
            id="grad-end"
            label="渐变终点"
            value={config.gradientEndColor}
            onChange={(v) => onChange({ gradientEndColor: v })}
          />
          <RangeField
            id="grad-angle"
            label="渐变角度"
            value={config.gradientAngleDegrees}
            min={0}
            max={360}
            step={1}
            unit="°"
            onChange={(v) => onChange({ gradientAngleDegrees: v })}
          />
        </>
      )}
    </section>
  );
}
