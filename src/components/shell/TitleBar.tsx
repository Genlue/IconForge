import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "../../store/useSettingsStore";

// 模块级单例：放在组件外，避免每次渲染重建 Window 对象导致回调 identity 变化
const win = getCurrentWindow();

export function TitleBar(): JSX.Element {
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);

  useEffect(() => {
    let disposed = false;
    // unlisten 函数只有在 promise resolve 后才能拿到。若组件在此之前卸载，
    // resolve 出来的 unlisten 会 push 进已废弃的数组而永远不被调用 —— 监听器泄漏。
    // 因此分两处收集：已 resolve 的立即记录，未 resolve 的在 then 里补调。
    const settledUnlisteners: Array<() => void> = [];

    const track = (p: Promise<() => void>): void => {
      void p.then(
        (unlisten) => {
          if (disposed) unlisten();
          else settledUnlisteners.push(unlisten);
        },
        () => {
          // 注册失败：没有监听器需要清理
        },
      );
    };

    const syncMaximized = (): void => {
      void win.isMaximized().then((val) => {
        if (!disposed) setMaximized(val);
      });
    };
    syncMaximized();

    void win.isFocused().then((val) => {
      if (!disposed) setFocused(val);
    });

    // 拖拽缩放期间 onResized 高频触发，leading+trailing 节流以限制 isMaximized() IPC 频率
    const RESIZE_THROTTLE_MS = 150;
    let lastSync = 0;
    let trailingTimer: number | undefined;
    const throttledSyncMaximized = (): void => {
      const now = Date.now();
      if (now - lastSync >= RESIZE_THROTTLE_MS) {
        lastSync = now;
        syncMaximized();
      } else if (trailingTimer === undefined) {
        trailingTimer = window.setTimeout(() => {
          trailingTimer = undefined;
          lastSync = Date.now();
          syncMaximized();
        }, RESIZE_THROTTLE_MS - (now - lastSync));
      }
    };

    track(win.onResized(throttledSyncMaximized));
    track(
      win.onFocusChanged(({ payload }) => {
        if (!disposed) setFocused(payload);
      }),
    );

    return () => {
      disposed = true;
      if (trailingTimer !== undefined) window.clearTimeout(trailingTimer);
      for (const unlisten of settledUnlisteners) unlisten();
    };
  }, []);

  return (
    <header
      className={`titlebar${focused ? "" : " titlebar--unfocused"}`}
      data-tauri-drag-region
    >
      <div className="traffic-lights" data-tauri-drag-region>
        {/* 1. 关闭按钮 */}
        <button
          type="button"
          className="traffic-light traffic-light--close"
          title="关闭"
          aria-label="关闭窗口"
          onClick={() => void win.close()}
        >
          <svg
            className="traffic-glyph glyph-close"
            width="12"
            height="12"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.15"
            strokeLinecap="round"
          >
            <path d="M3.7 3.7 L8.3 8.3 M8.3 3.7 L3.7 8.3" />
          </svg>
        </button>

        {/* 2. 最小化按钮 */}
        <button
          type="button"
          className="traffic-light traffic-light--minimize"
          title="最小化"
          aria-label="最小化窗口"
          onClick={() => void win.minimize()}
        >
          <svg
            className="traffic-glyph"
            width="12"
            height="12"
            viewBox="0 0 12 12"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.2"
            strokeLinecap="round"
          >
            <path d="M2.9 6 h6.2" />
          </svg>
        </button>

        {/* 3. 最大化 / 还原按钮 */}
        <button
          type="button"
          className="traffic-light traffic-light--maximize"
          title={maximized ? "还原" : "最大化"}
          aria-label={maximized ? "还原窗口" : "最大化窗口"}
          onClick={() => void win.toggleMaximize()}
        >
          {maximized ? (
            /* 已最大化：向心内缩对顶双三角，点击还原窗口 */
            <svg
              className="traffic-glyph glyph-restore"
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="currentColor"
            >
              <path d="M 2.2 5.5 L 5.0 5.5 Q 5.5 5.5 5.5 5.0 L 5.5 2.2 Z" />
              <path d="M 9.8 6.5 L 7.0 6.5 Q 6.5 6.5 6.5 7.0 L 6.5 9.8 Z" />
            </svg>
          ) : (
            /* 未最大化：外向扩张对角双三角（左上 ◤ 与 右下 ◢），点击最大化 */
            <svg
              className="traffic-glyph glyph-maximize"
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="currentColor"
            >
              <path d="M 3.8 3.1 L 7.9 3.1 L 3.1 7.9 L 3.1 3.8 Q 3.1 3.1 3.8 3.1 Z" />
              <path d="M 8.2 8.9 L 4.1 8.9 L 8.9 4.1 L 8.9 8.2 Q 8.9 8.9 8.2 8.9 Z" />
            </svg>
          )}
        </button>
      </div>

      <div className="titlebar__drag-space" data-tauri-drag-region />

      <button
        type="button"
        className="titlebar__action"
        title="软件设置"
        aria-label="软件设置"
        onClick={() => setSettingsOpen(true)}
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <circle cx="12" cy="12" r="3.2" />
          <path d="M12 2.6v2.2M12 19.2v2.2M4.4 12H2.2M21.8 12h-2.2M6.3 6.3 4.8 4.8M19.2 19.2l-1.5-1.5M17.7 6.3l1.5-1.5M4.8 19.2l1.5-1.5" />
        </svg>
      </button>
    </header>
  );
}
