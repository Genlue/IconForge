import { useIconForgeStore } from "../../store/useIconForgeStore";
import { BatchItem } from "./BatchItem";
import { BatchToolbar } from "./BatchToolbar";

export function BatchTray(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const selectItem = useIconForgeStore((s) => s.selectItem);
  const removeItem = useIconForgeStore((s) => s.removeItem);
  const clearItems = useIconForgeStore((s) => s.clearItems);

  return (
    <div className="glass-panel flex h-full flex-col overflow-hidden p-3">
      <BatchToolbar itemCount={items.length} onClear={clearItems} />
      <div className="mt-2 flex flex-1 gap-2 overflow-x-auto">
        {items.length === 0 && (
          <div className="flex w-full items-center justify-center">
            <span className="text-xs text-[var(--text-secondary)]">
              暂无导入项 — 拖入文件开始
            </span>
          </div>
        )}
        {items.map((item) => (
          <BatchItem
            key={item.id}
            item={item}
            selected={item.id === selectedId}
            onSelect={selectItem}
            onRemove={removeItem}
          />
        ))}
      </div>
    </div>
  );
}
