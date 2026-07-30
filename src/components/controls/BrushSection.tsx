import { useIconForgeStore } from "../../store/useIconForgeStore";
import { ColorField } from "./ColorField";
import { RangeField } from "./RangeField";

export function BrushSection(): JSX.Element {
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const config = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const tool = useIconForgeStore((s) => s.previewTool);
  const setPreviewTool = useIconForgeStore((s) => s.setPreviewTool);
  const updateBrushSettings = useIconForgeStore((s) => s.updateBrushSettings);
  const undoBrushStroke = useIconForgeStore((s) => s.undoBrushStroke);
  const clearBrushStrokes = useIconForgeStore((s) => s.clearBrushStrokes);
  const disabled = !selectedId || !config;

  return (
    <section className="space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-semibold text-[var(--text-primary)]">画笔</h3>
        <span className="text-[10px] text-[var(--text-secondary)]">
          {config?.brushStrokes.length ?? 0} 笔
        </span>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <button type="button" disabled={disabled} onClick={() => { updateBrushSettings({ mode: "paint" }); setPreviewTool(tool === "brush" ? "none" : "brush"); }} className={`rounded-lg border px-3 py-2 text-xs ${tool === "brush" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40"}`}>画笔</button>
        <button type="button" disabled={disabled} onClick={() => { updateBrushSettings({ mode: "erase" }); setPreviewTool(tool === "eraser" ? "none" : "eraser"); }} className={`rounded-lg border px-3 py-2 text-xs ${tool === "eraser" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40"}`}>涂抹擦除</button>
      </div>
      <ColorField
        id="brush-color"
        label="画笔颜色"
        value={config?.brushColor ?? "#FF3B30FF"}
        disabled={disabled}
        pickTarget="brush"
        onChange={(color) => updateBrushSettings({ color })}
      />
      <RangeField
        id="brush-size"
        label="画笔大小"
        value={config?.brushSize ?? 8}
        min={1}
        max={64}
        step={1}
        disabled={disabled}
        onChange={(size) => updateBrushSettings({ size })}
      />
      <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
        <input type="checkbox" checked={config?.brushClipToMask ?? true} disabled={disabled} onChange={(event) => updateBrushSettings({ clipToMask: event.target.checked })} />
        笔迹不超出当前图标外围
      </label>
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled || (config?.brushStrokes.length ?? 0) === 0}
          onClick={undoBrushStroke}
          className="rounded-lg border border-[var(--border-hairline)] bg-white/40 px-2 py-1.5 text-xs disabled:opacity-40"
        >
          撤销一笔
        </button>
        <button
          type="button"
          disabled={disabled || (config?.brushStrokes.length ?? 0) === 0}
          onClick={clearBrushStrokes}
          className="rounded-lg border border-red-200 bg-red-50 px-2 py-1.5 text-xs text-red-600 disabled:opacity-40"
        >
          清空画笔
        </button>
      </div>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        画笔和擦除笔迹属于当前图标，并会进入预览、PNG 和最终 ICO。
      </p>
    </section>
  );
}
