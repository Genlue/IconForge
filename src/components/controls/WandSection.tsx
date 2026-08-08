import { useIconForgeStore } from "../../store/useIconForgeStore";
import { RangeField } from "./RangeField";

export function WandSection(): JSX.Element {
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const config = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const tool = useIconForgeStore((s) => s.previewTool);
  const setPreviewTool = useIconForgeStore((s) => s.setPreviewTool);
  const setWandTolerance = useIconForgeStore((s) => s.setWandTolerance);
  const undoWandStroke = useIconForgeStore((s) => s.undoWandStroke);
  const clearWandStrokes = useIconForgeStore((s) => s.clearWandStrokes);
  const disabled = !selectedId || !config;
  const strokeCount = config?.wandStrokes.length ?? 0;

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h4 className="text-[11px] font-semibold text-[var(--text-secondary)]">魔棒</h4>
        <span className="text-[10px] text-[var(--text-secondary)]">{strokeCount} 次</span>
      </div>
      <button
        type="button"
        disabled={disabled}
        onClick={() => setPreviewTool(tool === "magic-wand" ? "none" : "magic-wand")}
        className={`w-full rounded-lg border px-3 py-2 text-xs ${tool === "magic-wand" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40"}`}
      >
        魔棒：点击删除相似区域
      </button>
      <RangeField
        id="wand-tolerance"
        label="容差"
        value={config?.wandTolerance ?? 72}
        min={0}
        max={100}
        step={1}
        disabled={disabled}
        onChange={(tolerance) => setWandTolerance(tolerance)}
      />
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={undoWandStroke}
          className="rounded-lg border border-[var(--border-hairline)] bg-white/40 px-2 py-1.5 text-xs disabled:opacity-40"
        >
          撤销一次
        </button>
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={clearWandStrokes}
          className="rounded-lg border border-red-200 bg-red-50 px-2 py-1.5 text-xs text-red-600 disabled:opacity-40"
        >
          清空删除
        </button>
      </div>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        容差越大，一次选中的相似区域越多。被选中的区域在原图上变为全透明，并会进入预览、PNG 和最终 ICO。
      </p>
    </section>
  );
}