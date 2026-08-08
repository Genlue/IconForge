import type { AutoCutoutConfig } from "../../types/domain";
import { CollapsibleSection } from "./CollapsibleSection";
import { RangeField } from "./RangeField";
import { WandSection } from "./WandSection";
import { EraserSection } from "./EraserSection";

export function SourceProcessingSection(props: {
  cutout: AutoCutoutConfig;
  onCutoutChange(patch: Partial<AutoCutoutConfig>): void;
  disabled: boolean;
}): JSX.Element {
  const { cutout, onCutoutChange, disabled } = props;

  return (
    <CollapsibleSection title="原图处理">
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        以下操作直接作用于导入的原图。
      </p>
      <div className={disabled ? "pointer-events-none opacity-40" : "space-y-4"}>
        <CollapsibleSection title="自动抠图" variant="sub">
          <div className="flex items-center justify-between">
            <span className="text-xs text-[var(--text-secondary)]">从背景中提取主体</span>
            <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
              <input
                type="checkbox"
                checked={cutout.enabled}
                onChange={(e) => onCutoutChange({ enabled: e.target.checked })}
                className="rounded"
              />
              启用
            </label>
          </div>
          <div className={cutout.enabled ? "space-y-3" : "pointer-events-none space-y-3 opacity-40"}>
            <RangeField id="cutout-tolerance" label="背景容差" value={cutout.tolerance} min={0} max={100} step={1} onChange={(tolerance) => onCutoutChange({ tolerance })} />
            <RangeField id="cutout-feather" label="边缘羽化" value={cutout.feather} min={0} max={32} step={1} onChange={(feather) => onCutoutChange({ feather })} />
          </div>
          <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
            从图像边界识别连通背景，适合白底、纯色底和近纯色底图标。
          </p>
        </CollapsibleSection>
        <WandSection />
        <EraserSection />
      </div>
    </CollapsibleSection>
  );
}