import { useIconForgeStore } from "../../store/useIconForgeStore";
import { ExportResultDialog } from "./ExportResultDialog";
import { RangeField } from "../controls/RangeField";

export function ExportActions(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const isExporting = useIconForgeStore((s) => s.isExporting);
  const lastExportResult = useIconForgeStore((s) => s.lastExportResult);
  const selectedItemId = useIconForgeStore((s) => s.selectedItemId);
  const renderConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId]?.renderConfig : undefined,
  );
  const updateRenderConfig = useIconForgeStore((s) => s.updateRenderConfig);
  const exportAsIco = useIconForgeStore((s) => s.exportAsIco);
  const exportAllAsIco = useIconForgeStore((s) => s.exportAllAsIco);
  const exportAsPng = useIconForgeStore((s) => s.exportAsPng);
  const exportAllAsPng = useIconForgeStore((s) => s.exportAllAsPng);
  const extractIcoAsPng = useIconForgeStore((s) => s.extractIcoAsPng);
  const applyToSelectedLnks = useIconForgeStore((s) => s.applyToSelectedLnks);
  const hasLnks = items.some((i) => i.fileType === "Lnk");
  const hasIcoCapable = items.length > 0;
  const hasIcoSources = items.some((item) => item.sourcePath.toLowerCase().endsWith(".ico"));
  const contentScale = renderConfig?.contentScalePercent ?? 100;

  return (
    <>
      <div className="flex flex-col gap-2">
        <div className="grid grid-cols-2 gap-2">
          <button
            onClick={exportAsIco}
            disabled={!hasIcoCapable || isExporting}
            className="rounded-lg bg-[var(--accent)] px-2 py-2 text-xs text-white transition-opacity hover:bg-[var(--accent)]/90 disabled:opacity-40"
          >
            {isExporting ? "导出中..." : "导出当前 ICO"}
          </button>
          <button
            onClick={exportAllAsIco}
            disabled={!hasIcoCapable || isExporting}
            className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-2 py-2 text-xs text-[var(--accent)] transition-colors hover:bg-[var(--accent)]/15 disabled:opacity-40"
          >
            {isExporting ? "导出中..." : "全部导出 ICO"}
          </button>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <button onClick={exportAsPng} disabled={!hasIcoCapable || isExporting} className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-2 py-2 text-xs text-[var(--accent)] disabled:opacity-40">导出当前 PNG</button>
          <button onClick={exportAllAsPng} disabled={!hasIcoCapable || isExporting} className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-2 py-2 text-xs text-[var(--accent)] disabled:opacity-40">全部导出 PNG</button>
        </div>
        <RangeField
          id="export-content-scale"
          label="图标整体缩放"
          value={contentScale}
          min={0}
          max={100}
          step={1}
          unit="%"
          disabled={!renderConfig || !selectedItemId}
          onChange={(contentScalePercent) => updateRenderConfig({ contentScalePercent })}
        />
        <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
          保持 256×256 总画布，把最终图标内容整体缩小靠中（默认 100%）。
        </p>
        <button onClick={extractIcoAsPng} disabled={!hasIcoSources || isExporting} className="w-full rounded-lg border border-[var(--border-hairline)] bg-white/40 px-3 py-2 text-xs text-[var(--text-primary)] disabled:opacity-40">批量提取 ICO 最大帧为 PNG</button>
        <button
          onClick={applyToSelectedLnks}
          disabled={!hasLnks || isExporting}
          className="w-full rounded-lg border border-[var(--border-hairline)] bg-white/40 px-3 py-2 text-sm text-[var(--text-primary)] transition-colors hover:bg-white/60 disabled:opacity-40"
        >
          {isExporting ? "应用中..." : "应用到 LNK 快捷方式"}
        </button>
      </div>
      <ExportResultDialog result={lastExportResult} onClose={() => {
        useIconForgeStore.setState({ lastExportResult: null });
      }} />
    </>
  );
}