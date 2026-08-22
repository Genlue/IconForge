import { useIconForgeStore } from "../../store/useIconForgeStore";
import { BUILT_IN_PRESETS } from "../../constants/presets";
import { PresetSelector } from "./PresetSelector";
import { ForegroundSection } from "./ForegroundSection";
import { ShapeSection } from "./ShapeSection";
import { BackplateSection } from "./BackplateSection";
import { ShadowSection } from "./ShadowSection";
import { StrokeSection } from "./StrokeSection";
import { UpscaleSection } from "./UpscaleSection";
import { CollapsibleSection } from "./CollapsibleSection";
import { ExportActions } from "../export/ExportActions";
import { BrushSection } from "./BrushSection";
import { GlossSection } from "./GlossSection";
import { SourceProcessingSection } from "./SourceProcessingSection";
import { HqRenderSection } from "./HqRenderSection";
import { GlassSection } from "./GlassSection";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../../constants/defaults";

const HQ_PRESET_IDS = new Set(["hq-render"]);
const GLASS_PRESET_IDS = new Set(["glass"]);

export function ParameterPanel(): JSX.Element {
  const selectedItemId = useIconForgeStore((s) => s.selectedItemId);
  const itemCount = useIconForgeStore((s) => s.items.length);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
const applyPreset = useIconForgeStore((s) => s.applyPreset);
  const applyCurrentConfigToAll = useIconForgeStore((s) => s.applyCurrentConfigToAll);
  const customPresets = useIconForgeStore((s) => s.customPresets);
  const saveCurrentAsPreset = useIconForgeStore((s) => s.saveCurrentAsPreset);
  const renamePreset = useIconForgeStore((s) => s.renamePreset);
  const deletePreset = useIconForgeStore((s) => s.deletePreset);
  const updateRenderConfig = useIconForgeStore((s) => s.updateRenderConfig);
  const updateOuterShadow = useIconForgeStore((s) => s.updateOuterShadow);
  const updateStroke = useIconForgeStore((s) => s.updateStroke);
  const updateUpscaleConfig = useIconForgeStore((s) => s.updateUpscaleConfig);
  const renderConfig = itemConfig?.renderConfig ?? DEFAULT_RENDER_CONFIG;
  const upscaleConfig = itemConfig?.upscaleConfig ?? DEFAULT_UPSCALE_CONFIG;
  const activePresetId = itemConfig?.activePresetId ?? null;
  const hasSelection = selectedItemId !== null && itemConfig !== undefined;
  const isHqPreset = HQ_PRESET_IDS.has(activePresetId ?? "");
  const isGlassPreset = GLASS_PRESET_IDS.has(activePresetId ?? "");
  // Stay visible while the item is in HQ/glass mode even after the user tweaks
  // a parameter (which clears activePresetId); only leaving for another preset
  // turns the adaptive renderer off.
  const glassActive = isGlassPreset || renderConfig.glassRender.enabled;
  const hqActive = (isHqPreset || renderConfig.hqRender.enabled) && !glassActive;

  return (
    <aside className="h-full overflow-y-auto p-4" style={{ minWidth: 0 }}>
      <div className="glass-panel space-y-1 p-4">
        {/* Presets */}
        <CollapsibleSection title="预设">
          <PresetSelector
            presets={[...BUILT_IN_PRESETS, ...customPresets]}
            activePresetId={activePresetId}
            onSelect={applyPreset}
            onRename={renamePreset}
            onDelete={deletePreset}
          />
          <button
            type="button"
            onClick={saveCurrentAsPreset}
            disabled={!hasSelection}
            className="mt-2 w-full rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-3 py-2 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)]/15 disabled:cursor-not-allowed disabled:opacity-40"
            title="保存渲染风格设置（不含原图处理、画质增强与画笔）"
          >
            保存当前设置为预设
          </button>
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
        </CollapsibleSection>

        {/* Source enhancement */}
        <CollapsibleSection title="画质增强">
          <UpscaleSection
            value={upscaleConfig}
            onChange={updateUpscaleConfig}
          />
        </CollapsibleSection>

        {/* Original image editing (auto cutout + magic wand + eraser) */}
        <SourceProcessingSection
          cutout={renderConfig.autoCutout}
          onCutoutChange={(autoCutout) => updateRenderConfig({ autoCutout: { ...renderConfig.autoCutout, ...autoCutout } })}
          disabled={!hasSelection}
        />

        {glassActive && (
          <CollapsibleSection title="玻璃质感参数">
            <GlassSection
              value={renderConfig.glassRender}
              onChange={(patch) => updateRenderConfig({ glassRender: { ...renderConfig.glassRender, ...patch } })}
            />
          </CollapsibleSection>
        )}

        {hqActive && (
          <CollapsibleSection title="高质量渲染参数">
            <HqRenderSection
              value={renderConfig.hqRender}
              gloss={renderConfig.gloss}
              onChange={(patch) => updateRenderConfig({ hqRender: { ...renderConfig.hqRender, ...patch } })}
              onGlossChange={(patch) => updateRenderConfig({ gloss: { ...renderConfig.gloss, ...patch } })}
            />
          </CollapsibleSection>
        )}

        {!hqActive && (
          <>
            <CollapsibleSection title="前景">
              <ForegroundSection
                config={renderConfig}
                onChange={updateRenderConfig}
              />
            </CollapsibleSection>

            <CollapsibleSection title="形状">
              <ShapeSection
                config={renderConfig}
                onChange={updateRenderConfig}
              />
            </CollapsibleSection>

            <CollapsibleSection title="背板">
              <BackplateSection
                config={renderConfig}
                onChange={updateRenderConfig}
              />
            </CollapsibleSection>

            <CollapsibleSection title="外阴影">
              <ShadowSection
                value={renderConfig.outerShadow}
                onChange={updateOuterShadow}
              />
            </CollapsibleSection>

            <CollapsibleSection title="描边">
              <StrokeSection
                value={renderConfig.stroke}
                onChange={updateStroke}
              />
            </CollapsibleSection>

            <CollapsibleSection title="边缘光泽">
              <GlossSection value={renderConfig.gloss} onChange={(gloss) => updateRenderConfig({ gloss: { ...renderConfig.gloss, ...gloss } })} />
            </CollapsibleSection>
          </>
        )}

        <CollapsibleSection title="画笔">
          <BrushSection />
        </CollapsibleSection>

        <CollapsibleSection title="导出">
          <ExportActions />
        </CollapsibleSection>
      </div>
    </aside>
  );
}