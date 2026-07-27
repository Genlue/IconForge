import { useIconForgeStore } from "../../store/useIconForgeStore";
import { TitleBar } from "./TitleBar";
import { StatusBar } from "./StatusBar";
import { PreviewPane } from "../preview/PreviewPane";
import { ParameterPanel } from "../controls/ParameterPanel";
import { BatchTray } from "../batch/BatchTray";
import { ErrorBanner } from "../common/ErrorBanner";

export function AppShell(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const isImporting = useIconForgeStore((s) => s.isImporting);
  const isExporting = useIconForgeStore((s) => s.isExporting);
  const error = useIconForgeStore((s) => s.error);
  const clearError = useIconForgeStore((s) => s.clearError);

  return (
    <div
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(520px, 1fr) 360px",
        gridTemplateRows: "52px 1fr 184px 24px",
        gridTemplateAreas: `
          "title title"
          "preview controls"
          "batch controls"
          "status status"
        `,
        width: "100vw",
        height: "100vh",
        minWidth: 960,
        minHeight: 700,
        overflow: "hidden",
        background: "var(--surface-app)",
      }}
    >
      <div style={{ gridArea: "title" }}>
        <TitleBar />
      </div>
      <div style={{ gridArea: "preview" }}>
        <PreviewPane />
      </div>
      <div style={{ gridArea: "controls" }}>
        <ParameterPanel />
      </div>
      <div style={{ gridArea: "batch" }}>
        <BatchTray />
      </div>
      <ErrorBanner error={error} onDismiss={clearError} />
      <div style={{ gridArea: "status" }}>
        <StatusBar
          itemCount={items.length}
          busy={isImporting || isExporting}
          message={isImporting ? "导入中..." : isExporting ? "导出中..." : null}
        />
      </div>
    </div>
  );
}
