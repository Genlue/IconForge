import { useEffect } from "react";
import { createPortal } from "react-dom";
import type { ExportIcoResponse, ApplyToLnkResponse } from "../../types/commands";

export interface ExportResultDialogProps {
  result: ExportIcoResponse | ApplyToLnkResponse | null;
  onClose(): void;
}

export function ExportResultDialog(props: ExportResultDialogProps): JSX.Element | null {
  useEffect(() => {
    if (!props.result) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") props.onClose();
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [props.result, props.onClose]);

  if (!props.result) return null;

  const result = props.result;
  const isIcoResult = isExportIcoResponse(result);
  const successCount = isIcoResult
    ? result.files.length
    : result.applied.length;
  const issueCount = isIcoResult
    ? result.warnings.length
    : result.failed.length;

  return createPortal(
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/25 p-6 backdrop-blur-sm"
      role="dialog"
      aria-modal="true"
      aria-labelledby="export-result-title"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) props.onClose();
      }}
    >
      <div className="glass-panel flex max-h-[min(640px,calc(100vh-48px))] w-full max-w-[560px] flex-col overflow-hidden bg-[var(--surface-overlay)]">
        <div className="border-b border-[var(--border-hairline)] px-6 py-5">
          <h2 id="export-result-title" className="text-base font-semibold text-[var(--text-primary)]">
            {issueCount > 0 ? "导出完成（有提示）" : "导出完成"}
          </h2>
          <p className="mt-1 text-sm text-[var(--text-secondary)]">
            成功 {successCount} 项{issueCount > 0 ? `，提示 ${issueCount} 项` : ""}
          </p>
        </div>

        <div className="min-h-0 flex-1 space-y-4 overflow-y-auto px-6 py-5">
          {isIcoResult ? (
            <>
              {result.files.map((file) => (
                <ResultRow key={file.outputPath} label="ICO 已保存" path={file.outputPath} />
              ))}
              {result.warnings.map((warning, index) => (
                <ResultRow key={`${index}-${warning}`} label="提示" path={warning} warning />
              ))}
            </>
          ) : (
            <>
              {result.applied.map((item) => (
                <ResultRow key={item.lnkPath} label="快捷方式已更新" path={item.lnkPath} />
              ))}
              {result.failed.map((item) => (
                <ResultRow
                  key={item.path}
                  label={`失败：${item.code}`}
                  path={`${item.path}\n${item.message}`}
                  warning
                />
              ))}
            </>
          )}

          {successCount === 0 && issueCount === 0 && (
            <p className="text-sm text-[var(--text-secondary)]">没有生成任何文件。</p>
          )}
        </div>

        <div className="flex justify-end border-t border-[var(--border-hairline)] px-6 py-4">
          <button
            autoFocus
            onClick={props.onClose}
            className="rounded-lg bg-[var(--accent)] px-5 py-2 text-sm text-white"
          >
            确定
          </button>
        </div>
      </div>
    </div>,
    document.body,
  );
}

function isExportIcoResponse(
  result: ExportIcoResponse | ApplyToLnkResponse,
): result is ExportIcoResponse {
  return "files" in result;
}

function ResultRow(props: { label: string; path: string; warning?: boolean }): JSX.Element {
  return (
    <div className={`rounded-xl border p-3 ${
      props.warning
        ? "border-amber-200 bg-amber-50/80"
        : "border-[var(--border-hairline)] bg-[var(--surface-control)]"
    }`}>
      <div className="text-xs font-semibold text-[var(--text-primary)]">{props.label}</div>
      <div className="mt-1 whitespace-pre-wrap break-all text-xs leading-5 text-[var(--text-secondary)]">
        {props.path}
      </div>
    </div>
  );
}
