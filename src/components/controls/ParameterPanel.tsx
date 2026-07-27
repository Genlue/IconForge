import { useIconForgeStore } from "../../store/useIconForgeStore";
import { BUILT_IN_PRESETS } from "../../constants/presets";
import { PresetSelector } from "./PresetSelector";
import { ForegroundSection } from "./ForegroundSection";
import { ShapeSection } from "./ShapeSection";
import { BackplateSection } from "./BackplateSection";
import { ShadowSection } from "./ShadowSection";
import { StrokeSection } from "./StrokeSection";
import { ExportActions } from "../export/ExportActions";

export function ParameterPanel(): JSX.Element {
  const renderConfig = useIconForgeStore((s) => s.renderConfig);
  const activePresetId = useIconForgeStore((s) => s.activePresetId);
  const applyPreset = useIconForgeStore((s) => s.applyPreset);
  const updateRenderConfig = useIconForgeStore((s) => s.updateRenderConfig);
  const updateOuterShadow = useIconForgeStore((s) => s.updateOuterShadow);
  const updateStroke = useIconForgeStore((s) => s.updateStroke);

  return (
    <aside className="h-full overflow-y-auto p-4" style={{ minWidth: 0 }}>
      <div className="glass-panel space-y-5 p-4">
        {/* Presets */}
        <section>
          <h3 className="mb-2 text-xs font-semibold text-[var(--text-primary)]">预设</h3>
          <PresetSelector
            presets={BUILT_IN_PRESETS}
            activePresetId={activePresetId}
            onSelect={applyPreset}
          />
        </section>

        <hr className="border-[var(--border-hairline)]" />

        {/* Foreground */}
        <ForegroundSection
          config={renderConfig}
          onChange={updateRenderConfig}
        />

        <hr className="border-[var(--border-hairline)]" />

        {/* Shape */}
        <ShapeSection
          config={renderConfig}
          onChange={updateRenderConfig}
        />

        <hr className="border-[var(--border-hairline)]" />

        {/* Backplate */}
        <BackplateSection
          config={renderConfig}
          onChange={updateRenderConfig}
        />

        <hr className="border-[var(--border-hairline)]" />

        {/* Shadow */}
        <ShadowSection
          value={renderConfig.outerShadow}
          onChange={updateOuterShadow}
        />

        <hr className="border-[var(--border-hairline)]" />

        {/* Stroke */}
        <StrokeSection
          value={renderConfig.stroke}
          onChange={updateStroke}
        />

        <hr className="border-[var(--border-hairline)]" />

        {/* Export */}
        <ExportActions />
      </div>
    </aside>
  );
}
