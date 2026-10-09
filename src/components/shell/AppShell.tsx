import { useIconForgeStore } from "../../store/useIconForgeStore";
import { useSettingsStore } from "../../store/useSettingsStore";
import { useWindowDecorations } from "../../hooks/useWindowDecorations";
import { StatusBar } from "./StatusBar";
import { TitleBar } from "./TitleBar";
import { PreviewPane } from "../preview/PreviewPane";
import { ParameterPanel } from "../controls/ParameterPanel";
import { BatchTray } from "../batch/BatchTray";
import { ErrorBanner } from "../common/ErrorBanner";
import { SettingsDialog } from "./SettingsDialog";

export function AppShell(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const isImporting = useIconForgeStore((s) => s.isImporting);
  const isExporting = useIconForgeStore((s) => s.isExporting);
  const error = useIconForgeStore((s) => s.error);
  const clearError = useIconForgeStore((s) => s.clearError);
  const titleBarStyle = useSettingsStore((s) => s.titleBarStyle);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);

  const decorationStatus = useWindowDecorations();

  const macOSStyle = titleBarStyle === "macos";
  // Windows 原生样式下系统自己画标题栏，自定义顶栏行不再占用高度。
  const titlebarRow = macOSStyle ? "36px" : "0px";
  // 无边框模式下由 web 层裁出 10px 圆角；原生模式交给系统绘制，
  // 这里再裁会把内容四角切掉并露出底色。
  const radius = macOSStyle ? 10 : 0;
  const nativeFrameFailed = !macOSStyle && decorationStatus === "unsupported";

  return (
    <div
      className="app-shell"
      style={{
        display: "grid",
        gridTemplateColumns: "minmax(520px, 1fr) 360px",
        gridTemplateRows: `${titlebarRow} minmax(0, 1fr) 184px 24px`,
        gridTemplateAreas: `
          "titlebar titlebar"
          "preview controls"
          "batch controls"
          "status status"
        `,
        width: "100vw",
        height: "100vh",
        minWidth: 960,
        minHeight: 700,
        overflow: "hidden",
        borderRadius: radius,
        background: "var(--surface-app)",
      }}
    >
      {macOSStyle && (
        <div style={{ gridArea: "titlebar" }}>
          <TitleBar />
        </div>
      )}
      <div
        style={{
          gridArea: "preview",
          minHeight: 0,
          overflow: "hidden",
          display: "flex",
          flexDirection: "column",
        }}
      >
        {/* 原生模式下没有自定义顶栏，单独给一行入口，保证设置随时可达 */}
        {!macOSStyle && (
          <div className="native-settings-rail">
            <span className="text-[11px] text-[var(--text-secondary)]">
              IconForge
            </span>
            <button
              type="button"
              onClick={() => setSettingsOpen(true)}
              className="flex items-center gap-1 rounded-md px-2 py-0.5 text-[11px] text-[var(--text-secondary)] transition-colors hover:bg-white/60 hover:text-[var(--text-primary)]"
              title="软件设置"
              aria-label="软件设置"
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="3.2" />
                <path d="M12 2.6v2.2M12 19.2v2.2M4.4 12H2.2M21.8 12h-2.2M6.3 6.3 4.8 4.8M19.2 19.2l-1.5-1.5M17.7 6.3l1.5-1.5M4.8 19.2l1.5-1.5" />
              </svg>
              设置
            </button>
          </div>
        )}
        {nativeFrameFailed && (
          <div className="mx-3 mb-1 rounded-lg border border-[var(--warning-border)] bg-[var(--warning-surface)] px-3 py-1.5 text-[11px] leading-4 text-[var(--warning-text)]">
            未能切换到系统标题栏，窗口当前没有标题栏。请在设置中改回「macOS 红绿灯」。
          </div>
        )}
        <div style={{ minHeight: 0, flex: 1 }}>
          <PreviewPane />
        </div>
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
      <SettingsDialog nativeFrameFailed={nativeFrameFailed} />
    </div>
  );
}
