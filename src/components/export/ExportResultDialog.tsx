import type { ExportIcoResponse, ApplyToLnkResponse } from "../../types/commands";

export interface ExportResultDialogProps {
  result: ExportIcoResponse | ApplyToLnkResponse | null;
  onClose(): void;
}

export function ExportResultDialog(props: ExportResultDialogProps): JSX.Element | null {
  if (!props.result) return null;
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm">
      <div className="glass-panel max-h-[80vh] w-[480px] overflow-y-auto p-6">
        <h2 className="mb-4 text-sm font-semibold text-[var(--text-primary)]">
          导出结果
        </h2>
        <button
          onClick={props.onClose}
          className="mt-4 rounded-lg bg-[var(--accent)] px-4 py-2 text-sm text-white"
        >
          关闭
        </button>
      </div>
    </div>
  );
}
