import { useIconForgeStore } from "../../store/useIconForgeStore";
import { BatchItem } from "./BatchItem";

/**
 * 全高来源列表（导航层）：唯一的圆角选择填充属于选中行，
 * 未选行保持安静；导入/导出动作在工具栏，这里只放列表与清空。
 */
export function BatchTray(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const selectItem = useIconForgeStore((s) => s.selectItem);
  const removeItem = useIconForgeStore((s) => s.removeItem);
  const clearItems = useIconForgeStore((s) => s.clearItems);

  return (
    <aside className="pane-sidebar">
      <div className="source-header">
        <span>导入项{items.length > 0 ? ` · ${items.length}` : ""}</span>
        {items.length > 0 && (
          <button
            type="button"
            onClick={clearItems}
            className="text-[11px] text-[var(--text-secondary)] transition-colors hover:text-[var(--danger)]"
            title="移除全部导入项"
          >
            清空
          </button>
        )}
      </div>
      <div className="source-list">
        {items.length === 0 ? (
          <div className="flex h-full items-center justify-center px-4">
            <p className="hint text-center leading-5">
              拖入文件到窗口，<br />或点工具栏「导入」
            </p>
          </div>
        ) : (
          items.map((item) => (
            <BatchItem
              key={item.id}
              item={item}
              selected={item.id === selectedId}
              onSelect={selectItem}
              onRemove={removeItem}
            />
          ))
        )}
      </div>
    </aside>
  );
}
