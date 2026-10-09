import { useEffect } from "react";
import { useSettingsStore } from "../../store/useSettingsStore";
import type { TitleBarStyle } from "../../store/useSettingsStore";
import { SegmentedControl } from "../controls/SegmentedControl";

const TITLE_BAR_OPTIONS: ReadonlyArray<{ value: TitleBarStyle; label: string }> = [
  { value: "macos", label: "macOS 红绿灯" },
  { value: "windows", label: "Windows 原生" },
];

const TITLE_BAR_HINTS: Record<TitleBarStyle, string> = {
  macos: "自制 36px 顶栏，左侧 macOS 红绿灯，右侧内嵌设置入口。",
  windows: "交由系统绘制标题栏与最小化/最大化/关闭按钮，不占用窗口高度。",
};

export interface SettingsDialogProps {
  /** True when the native frame could not be applied, so switching away is advised. */
  nativeFrameFailed?: boolean;
}

export function SettingsDialog(props: SettingsDialogProps): JSX.Element | null {
  const open = useSettingsStore((s) => s.settingsOpen);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);
  const titleBarStyle = useSettingsStore((s) => s.titleBarStyle);
  const setTitleBarStyle = useSettingsStore((s) => s.setTitleBarStyle);

  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent): void => {
      if (e.key === "Escape") setSettingsOpen(false);
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, setSettingsOpen]);

  if (!open) return null;

  return (
    <div
      className="settings-overlay"
      role="presentation"
      onClick={() => setSettingsOpen(false)}
    >
      <div
        className="settings-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="软件设置"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">
            软件设置
          </h2>
          <button
            type="button"
            aria-label="关闭设置"
            title="关闭设置"
            onClick={() => setSettingsOpen(false)}
            className="flex h-6 w-6 items-center justify-center rounded-md text-[var(--text-secondary)] hover:bg-white/60 hover:text-[var(--text-primary)]"
          >
            ×
          </button>
        </div>

        <div className="mt-4 space-y-2">
          <span className="text-xs text-[var(--text-secondary)]">顶栏样式</span>
          <SegmentedControl
            id="settings-title-bar-style"
            value={titleBarStyle}
            options={TITLE_BAR_OPTIONS}
            onChange={setTitleBarStyle}
          />
          <p className="text-[11px] leading-4 text-[var(--text-secondary)]">
            {TITLE_BAR_HINTS[titleBarStyle]}
          </p>
          {props.nativeFrameFailed && (
            <p className="text-[11px] leading-4 text-[var(--warning-text)]">
              当前窗口未能应用系统标题栏，建议改回「macOS 红绿灯」。
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
