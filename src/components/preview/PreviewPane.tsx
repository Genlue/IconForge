import { useIconForgeStore } from "../../store/useIconForgeStore";
import { Checkerboard } from "./Checkerboard";
import { EmptyDropZone } from "./EmptyDropZone";
import { Spinner } from "../common/Spinner";
import { useDebouncedPreview } from "../../hooks/useDebouncedPreview";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../../constants/defaults";

export function PreviewPane(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const dragActive = useIconForgeStore((s) => s.dragActive);

  const selectedItem = items.find((i) => i.id === selectedId) ?? items[0] ?? null;
  const renderConfig = itemConfig?.renderConfig ?? DEFAULT_RENDER_CONFIG;
  const upscaleConfig = itemConfig?.upscaleConfig ?? DEFAULT_UPSCALE_CONFIG;
  const previewSizePx = Math.min(Math.max(256, 420), 420); // clamp 256-420
  const preview = useDebouncedPreview(
    selectedItem?.sourcePath ?? null,
    renderConfig,
    upscaleConfig,
    420,
  );

  return (
    <div className="relative flex h-full flex-col items-center justify-center gap-3 p-8">
      <div className="glass-panel relative flex aspect-square items-center justify-center overflow-hidden"
        style={{
          width: previewSizePx,
          maxWidth: "100%",
          maxHeight: "100%",
        }}
      >
        <Checkerboard />
        {selectedItem ? (
          <>
            {preview.pngDataUrl && (
              <img
                src={preview.pngDataUrl}
                alt={`${selectedItem.displayName} 最终预览`}
                className="relative z-10 h-full w-full object-contain"
                draggable={false}
              />
            )}
            {preview.loading && (
              <div className="absolute inset-0 z-20 flex items-center justify-center bg-white/55 backdrop-blur-sm">
                <Spinner label={upscaleConfig.enabled ? "Real-CUGAN 处理中..." : "正在渲染..."} />
              </div>
            )}
            {preview.error && (
              <div className="absolute inset-x-5 bottom-5 z-20 rounded-xl border border-red-200 bg-red-50/95 p-3 text-xs leading-5 text-red-700">
                {preview.error.message}
              </div>
            )}
          </>
        ) : (
          <EmptyDropZone dragActive={dragActive} />
        )}
      </div>
      {selectedItem && (
        <div className="flex max-w-full flex-wrap items-center justify-center gap-x-3 gap-y-1 text-[11px] text-[var(--text-secondary)]">
          <span>导入源图 {selectedItem.sourceWidth}×{selectedItem.sourceHeight}</span>
          {preview.processedSourceWidth && preview.processedSourceHeight && (
            <span>
              {upscaleConfig.enabled ? "增强源图" : "处理源图"} {preview.processedSourceWidth}×{preview.processedSourceHeight}
            </span>
          )}
          <span>主图 256×256</span>
          <span>ICO 256 / 128 / 64 / 48 / 32 / 24 / 20 / 16</span>
        </div>
      )}
    </div>
  );
}
