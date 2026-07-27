import { useIconForgeStore } from "../../store/useIconForgeStore";
import { BUILT_IN_PRESETS } from "../../constants/presets";
import { PresetSelector } from "./PresetSelector";
import { ForegroundSection } from "./ForegroundSection";
import { ShapeSection } from "./ShapeSection";
import { BackplateSection } from "./BackplateSection";
import { ShadowSection } from "./ShadowSection";
import { StrokeSection } from "./StrokeSection";
import { UpscaleSection } from "./UpscaleSection";
import { ExportActions } from "../export/ExportActions";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../../constants/defaults";

export function ParameterPanel(): JSX.Element {
  const selectedItemId = useIconForgeStore((s) => s.selectedItemId);
  const itemCount = useIconForgeStore((s) => s.items.length);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const applyPreset = useIconForgeStore((s) => s.applyPreset);
  const applyCurrentConfigToAll = useIconForgeStore((s) => s.applyCurrentConfigToAll);
  const updateRenderConfig = useIconForgeStore((s) => s.updateRenderConfig);
  const updateOuterShadow = useIconForgeStore((s) => s.updateOuterShadow);
  const updateStroke = useIconForgeStore((s) => s.updateStroke);
  const updateUpscaleConfig = useIconForgeStore((s) => s.updateUpscaleConfig);
  const renderConfig = itemConfig?.renderConfig ?? DEFAULT_RENDER_CONFIG;
  const upscaleConfig = itemConfig?.upscaleConfig ?? DEFAULT_UPSCALE_CONFIG;
  const activePresetId = itemConfig?.activePresetId ?? null;
  const hasSelection = selectedItemId !== null && itemConfig !== undefined;

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
          <button
            type="button"
            onClick={applyCurrentConfigToAll}
            disabled={!hasSelection || itemCount < 2}
            className="mt-3 w-full rounded-lg border border-[var(--border-hairline)] bg-white/40 px-3 py-2 text-xs text-[var(--text-primary)] transition-colors hover:bg-white/60 disabled:cursor-not-allowed disabled:opacity-40"
            title="复制当前图标的所有参数，包括 Real-CUGAN 设置"
          >
            应用当前配置到全部图标
          </button>
          {itemCount > 1 && (
            <p className="mt-2 text-[10px] leading-4 text-[var(--text-secondary)]">
              右侧参数仅修改当前图标；需要统一时再使用上方按钮。
            </p>
          )}
        </section>

        <hr className="border-[var(--border-hairline)]" />

        {/* Source enhancement */}
        <UpscaleSection
          value={upscaleConfig}
          onChange={updateUpscaleConfig}
        />

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
