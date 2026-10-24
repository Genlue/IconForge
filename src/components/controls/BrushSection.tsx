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
    <div className="flex flex-col gap-2.5">
      <span className="hint">
        已绘制 {config?.brushStrokes.length ?? 0} 笔
      </span>
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled}
          onClick={() => { updateBrushSettings({ mode: "paint" }); setPreviewTool(tool === "brush" ? "none" : "brush"); }}
          className={`btn btn--sm${tool === "brush" ? " btn--active" : ""}`}
        >
          画笔
        </button>
        <button
          type="button"
          disabled={disabled}
          onClick={() => { updateBrushSettings({ mode: "erase" }); setPreviewTool(tool === "eraser" ? "none" : "eraser"); }}
          className={`btn btn--sm${tool === "eraser" ? " btn--active" : ""}`}
        >
          涂抹擦除
        </button>
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
      <div className="field">
        <span className="field__label">裁切到图标</span>
        <div className="field__controls justify-end">
          <input
            type="checkbox"
            checked={config?.brushClipToMask ?? true}
            disabled={disabled}
            aria-label="笔迹不超出当前图标外围"
            onChange={(event) => updateBrushSettings({ clipToMask: event.target.checked })}
          />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled || (config?.brushStrokes.length ?? 0) === 0}
          onClick={undoBrushStroke}
          className="btn btn--sm"
        >
          撤销一笔
        </button>
        <button
          type="button"
          disabled={disabled || (config?.brushStrokes.length ?? 0) === 0}
          onClick={clearBrushStrokes}
          className="btn btn--sm btn--danger"
        >
          清空画笔
        </button>
      </div>
      <p className="hint">
        画笔和擦除笔迹属于当前图标，并会进入预览、PNG 和最终 ICO。
      </p>
    </div>
  );
}
