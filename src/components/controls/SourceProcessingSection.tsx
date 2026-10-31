import type { AutoCutoutConfig } from "../../types/domain";
import { CollapsibleSection } from "./CollapsibleSection";
import { CutoutSection } from "./CutoutSection";
import { WandSection } from "./WandSection";
import { EraserSection } from "./EraserSection";
import { RangeField } from "./RangeField";

export function SourceProcessingSection(props: {
  cutout: AutoCutoutConfig;
  onCutoutChange(patch: Partial<AutoCutoutConfig>): void;
  saturation: number;
  onSaturationChange(saturation: number): void;
  disabled: boolean;
}): JSX.Element {
  const { cutout, onCutoutChange, saturation, onSaturationChange, disabled } = props;

  return (
    <CollapsibleSection title="原图处理">
      <p className="hint">以下操作直接作用于导入的原图。</p>
      <div className={disabled ? "pointer-events-none flex flex-col gap-2.5 opacity-40" : "flex flex-col gap-2.5"}>
        <RangeField
          id="source-saturation"
          label="饱和度"
          value={saturation}
          min={0}
          max={100}
          step={1}
          onChange={onSaturationChange}
        />
        <p className="hint">100 保持原图色彩，0 完全去色为黑白灰。</p>
        <CutoutSection value={cutout} onChange={onCutoutChange} />
        <WandSection />
        <EraserSection />
      </div>
    </CollapsibleSection>
  );
}
