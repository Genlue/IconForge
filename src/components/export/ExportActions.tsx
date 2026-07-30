import { useIconForgeStore } from "../../store/useIconForgeStore";
import { ExportResultDialog } from "./ExportResultDialog";

export function ExportActions(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const isExporting = useIconForgeStore((s) => s.isExporting);
  const lastExportResult = useIconForgeStore((s) => s.lastExportResult);
  const exportAsIco = useIconForgeStore((s) => s.exportAsIco);
  const exportAllAsIco = useIconForgeStore((s) => s.exportAllAsIco);
  const exportAsPng = useIconForgeStore((s) => s.exportAsPng);
  const exportAllAsPng = useIconForgeStore((s) => s.exportAllAsPng);
  const extractIcoAsPng = useIconForgeStore((s) => s.extractIcoAsPng);
  const applyToSelectedLnks = useIconForgeStore((s) => s.applyToSelectedLnks);
  const hasLnks = items.some((i) => i.fileType === "Lnk");
  const hasIcoCapable = items.length > 0;
  const hasIcoSources = items.some((item) => item.sourcePath.toLowerCase().endsWith(".ico"));

  return (
    <>
      <div className="flex flex-col gap-2">
        <h3 className="text-xs font-semibold text-[var(--text-primary)]">导出</h3>
        <button
          onClick={exportAsIco}
          disabled={!hasIcoCapable || isExporting}
          className="w-full rounded-lg bg-[var(--accent)] px-3 py-2 text-sm text-white transition-opacity hover:bg-[var(--accent)]/90 disabled:opacity-40"
        >
          {isExporting ? "导出中..." : "导出当前 ICO"}
        </button>
        <button
          onClick={exportAllAsIco}
          disabled={!hasIcoCapable || isExporting}
          className="w-full rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-3 py-2 text-sm text-[var(--accent)] transition-colors hover:bg-[var(--accent)]/15 disabled:opacity-40"
        >
          {isExporting ? "导出中..." : "全部导出"}
        </button>
        <div className="grid grid-cols-2 gap-2">
          <button onClick={exportAsPng} disabled={!hasIcoCapable || isExporting} className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-2 py-2 text-xs text-[var(--accent)] disabled:opacity-40">导出当前 PNG</button>
          <button onClick={exportAllAsPng} disabled={!hasIcoCapable || isExporting} className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-2 py-2 text-xs text-[var(--accent)] disabled:opacity-40">全部导出 PNG</button>
        </div>
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
