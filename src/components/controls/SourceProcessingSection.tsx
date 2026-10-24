import type { AutoCutoutConfig } from "../../types/domain";
import { CollapsibleSection } from "./CollapsibleSection";
import { CutoutSection } from "./CutoutSection";
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
      <p className="hint">以下操作直接作用于导入的原图。</p>
      <div className={disabled ? "pointer-events-none flex flex-col gap-2.5 opacity-40" : "flex flex-col gap-2.5"}>
        <CutoutSection value={cutout} onChange={onCutoutChange} />
        <WandSection />
        <EraserSection />
      </div>
    </CollapsibleSection>
  );
}
