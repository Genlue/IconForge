import { useIconForgeStore } from "../../store/useIconForgeStore";
import { CollapsibleSection } from "./CollapsibleSection";
import { RangeField } from "./RangeField";

export function EraserSection(): JSX.Element {
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const config = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const tool = useIconForgeStore((s) => s.previewTool);
  const setPreviewTool = useIconForgeStore((s) => s.setPreviewTool);
  const setSourceEditorOpen = useIconForgeStore((s) => s.setSourceEditorOpen);
  const updateEraserSettings = useIconForgeStore((s) => s.updateEraserSettings);
  const undoEraserStroke = useIconForgeStore((s) => s.undoEraserStroke);
  const clearEraserStrokes = useIconForgeStore((s) => s.clearEraserStrokes);
  const disabled = !selectedId || !config;
  const strokeCount = config?.eraserStrokes.length ?? 0;

  return (
    <CollapsibleSection title="橡皮擦（原图）" variant="sub">
      <button
        type="button"
        disabled={disabled}
        onClick={() => {
          if (tool === "source-eraser") {
            setPreviewTool("none");
          } else {
            setPreviewTool("source-eraser");
            setSourceEditorOpen(true);
          }
        }}
        className={`w-full rounded-lg border px-3 py-2 text-xs ${tool === "source-eraser" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40"}`}
      >
        {tool === "source-eraser" ? "橡皮擦使用中：在原图编辑界面涂抹" : "橡皮擦：进入原图编辑并涂抹"}
      </button>
      <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
        在独立原图编辑界面涂抹：涂抹处变为全透明，并进入预览、PNG 和最终 ICO。
      </p>
      <RangeField
        id="eraser-size"
        label="大小"
        value={config?.eraserSize ?? 16}
        min={1}
        max={64}
        step={1}
        disabled={disabled}
        onChange={(size) => updateEraserSettings({ size })}
      />
      <RangeField
        id="eraser-hardness"
        label="硬度"
        value={config?.eraserHardness ?? 50}
        min={0}
        max={100}
        step={1}
        disabled={disabled}
        onChange={(hardness) => updateEraserSettings({ hardness })}
      />
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={undoEraserStroke}
          className="rounded-lg border border-[var(--border-hairline)] bg-white/40 px-2 py-1.5 text-xs disabled:opacity-40"
        >
          撤销一笔 ({strokeCount})
        </button>
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={clearEraserStrokes}
          className="rounded-lg border border-red-200 bg-red-50 px-2 py-1.5 text-xs text-red-600 disabled:opacity-40"
        >
          清空擦除
        </button>
      </div>
    </CollapsibleSection>
  );
}