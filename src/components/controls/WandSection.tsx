import { useIconForgeStore } from "../../store/useIconForgeStore";
import { CollapsibleSection } from "./CollapsibleSection";
import { RangeField } from "./RangeField";

export function WandSection(): JSX.Element {
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const config = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const tool = useIconForgeStore((s) => s.previewTool);
  const setPreviewTool = useIconForgeStore((s) => s.setPreviewTool);
  const setSourceEditorOpen = useIconForgeStore((s) => s.setSourceEditorOpen);
  const setWandTolerance = useIconForgeStore((s) => s.setWandTolerance);
  const undoWandStroke = useIconForgeStore((s) => s.undoWandStroke);
  const clearWandStrokes = useIconForgeStore((s) => s.clearWandStrokes);
  const deleteWandSelection = useIconForgeStore((s) => s.deleteWandSelection);
  const clearWandSelection = useIconForgeStore((s) => s.clearWandSelection);
  const wandSelection = useIconForgeStore((s) => s.wandSelection);
  const disabled = !selectedId || !config;
  const strokeCount = config?.wandStrokes.length ?? 0;

  return (
    <CollapsibleSection title="魔棒" variant="sub">
      <button
        type="button"
        disabled={disabled}
        onClick={() => {
          if (tool === "magic-wand") {
            setPreviewTool("none");
            clearWandSelection();
          } else {
            setPreviewTool("magic-wand");
            setSourceEditorOpen(true);
          }
        }}
        className={`btn btn--block btn--sm${tool === "magic-wand" ? " btn--active" : ""}`}
      >
        {tool === "magic-wand" ? "魔棒使用中：在原图编辑界面点击选中" : "魔棒：进入原图编辑并选中相似区域"}
      </button>
      <p className="hint">
        在独立原图编辑界面中圈选相似区域（蚂蚁线框出），按 Delete 或点击下方按钮删除，被选区域在原图上变为全透明，并进入预览、PNG 和最终 ICO。
      </p>
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
          disabled={!wandSelection}
          onClick={deleteWandSelection}
          className="btn btn--sm btn--danger"
        >
          删除选中区域 (Del)
        </button>
        <button
          type="button"
          disabled={!wandSelection}
          onClick={clearWandSelection}
          className="btn btn--sm"
        >
          取消选区
        </button>
      </div>
      <div className="grid grid-cols-2 gap-2">
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={undoWandStroke}
          className="btn btn--sm"
        >
          撤销删除 ({strokeCount})
        </button>
        <button
          type="button"
          disabled={disabled || strokeCount === 0}
          onClick={clearWandStrokes}
          className="btn btn--sm btn--danger"
        >
          清空删除
        </button>
      </div>
    </CollapsibleSection>
  );
}
