import { useEffect } from "react";
import { useSettingsStore } from "../../store/useSettingsStore";
import type { TitleBarStyle } from "../../store/useSettingsStore";
import { SegmentedControl } from "../controls/SegmentedControl";

const TITLE_BAR_OPTIONS: ReadonlyArray<{ value: TitleBarStyle; label: string }> = [
  { value: "macos", label: "macOS 红绿灯" },
  { value: "windows", label: "Windows 原生" },
];

const TITLE_BAR_HINTS: Record<TitleBarStyle, string> = {
  macos: "自制 48px 统一工具栏，左侧 macOS 红绿灯，标题居中，导入 / 导出 / 设置靠右。",
  windows: "交由系统绘制标题栏与最小化/最大化/关闭按钮，不占用窗口高度。",
};

export interface SettingsDialogProps {
  /** True when the native frame could not be applied, so switching away is advised. */
  nativeFrameFailed?: boolean;
}

/** 设置面板：瞬态表面，Liquid Glass 的正确去处。 */
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
      className="sheet-overlay"
      role="presentation"
      onClick={() => setSettingsOpen(false)}
    >
      <div
        className="sheet w-[340px] max-w-[calc(100vw-32px)]"
        role="dialog"
        aria-modal="true"
        aria-label="软件设置"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="sheet__header">
          <h2 className="sheet__title">软件设置</h2>
          <button
            type="button"
            aria-label="关闭设置"
            title="关闭设置"
            onClick={() => setSettingsOpen(false)}
            className="sheet__close"
          >
            ×
          </button>
        </div>
        <div className="sheet__body">
          <div className="field">
            <span className="field__label">顶栏样式</span>
            <div className="field__controls">
              <SegmentedControl
                id="settings-title-bar-style"
                value={titleBarStyle}
                options={TITLE_BAR_OPTIONS}
                onChange={setTitleBarStyle}
              />
            </div>
          </div>
          <p className="hint">
            {TITLE_BAR_HINTS[titleBarStyle]}
          </p>
          {props.nativeFrameFailed && (
            <p className="hint text-[var(--warning-text)]">
              当前窗口未能应用系统标题栏，建议改回「macOS 红绿灯」。
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
