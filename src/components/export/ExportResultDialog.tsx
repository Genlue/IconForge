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
      className="sheet-overlay p-6"
      role="presentation"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) props.onClose();
      }}
    >
      <div
        className="sheet w-full max-w-[520px]"
        role="dialog"
        aria-modal="true"
        aria-labelledby="export-result-title"
      >
        <div className="sheet__header">
          <div>
            <h2 id="export-result-title" className="sheet__title">
              {issueCount > 0 ? "导出完成（有提示）" : "导出完成"}
            </h2>
            <p className="sheet__subtitle mt-0.5">
              成功 {successCount} 项{issueCount > 0 ? `，提示 ${issueCount} 项` : ""}
            </p>
          </div>
        </div>

        <div className="sheet__body">
          <div className="result-list">
            {isIcoResult ? (
              <>
                {result.files.map((file) => (
                  <ResultRow key={file.outputPath} label="文件已保存" path={file.outputPath} />
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
          </div>

          {successCount === 0 && issueCount === 0 && (
            <p className="hint">没有生成任何文件。</p>
          )}
        </div>

        <div className="sheet__footer">
          <button autoFocus onClick={props.onClose} className="btn btn--primary px-5">
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
    <div className={`result-list__row${props.warning ? " result-list__row--warning" : ""}`}>
      <span className="result-list__label">{props.label}</span>
      <span className="result-list__path">{props.path}</span>
    </div>
  );
}
