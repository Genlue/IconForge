import { useIconForgeStore } from "../../store/useIconForgeStore";
import { Checkerboard } from "./Checkerboard";
import { EmptyDropZone } from "./EmptyDropZone";
import { IconPreview } from "./IconPreview";

export function PreviewPane(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const renderConfig = useIconForgeStore((s) => s.renderConfig);
  const dragActive = useIconForgeStore((s) => s.dragActive);

  const selectedItem = items.find((i) => i.id === selectedId) ?? items[0] ?? null;
  const previewSizePx = Math.min(Math.max(256, 420), 420); // clamp 256-420

  return (
    <div className="relative flex h-full items-center justify-center p-8">
      <div className="glass-panel relative flex aspect-square items-center justify-center overflow-hidden"
        style={{
          width: previewSizePx,
          maxWidth: "100%",
          maxHeight: "100%",
        }}
      >
        <Checkerboard />
        {selectedItem ? (
          <IconPreview item={selectedItem} config={renderConfig} sizePx={previewSizePx} />
        ) : (
          <EmptyDropZone dragActive={dragActive} />
        )}
      </div>
    </div>
  );
}
