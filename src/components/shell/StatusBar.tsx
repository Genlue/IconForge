import { useSettingsStore } from "../../store/useSettingsStore";

export interface StatusBarProps {
  itemCount: number;
  busy: boolean;
  message: string | null;
}

export function StatusBar(props: StatusBarProps): JSX.Element {
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);

  return (
    <div className="flex h-6 items-center justify-between px-3 text-xs text-[var(--text-secondary)]">
      <span>{props.busy ? "处理中..." : props.message ?? `${props.itemCount} 项`}</span>
      {/* 常驻入口：切到 Windows 原生顶栏后自定义顶栏消失，仍能从这里打开设置 */}
      <button
        type="button"
        title="软件设置"
        aria-label="软件设置"
        onClick={() => setSettingsOpen(true)}
        className="flex items-center gap-1 rounded px-1 py-0.5 transition-colors hover:bg-white/60 hover:text-[var(--text-primary)]"
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
        <span>设置</span>
      </button>
    </div>
  );
}
