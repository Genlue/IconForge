import { useCallback, useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open } from "@tauri-apps/plugin-dialog";
import { useIconForgeStore } from "../../store/useIconForgeStore";
import { useSettingsStore } from "../../store/useSettingsStore";

// 模块级单例：放在组件外，避免每次渲染重建 Window 对象导致回调 identity 变化
const win = getCurrentWindow();

const IMPORT_FILTERS = [{
  name: "支持的文件",
  extensions: ["png","jpg","jpeg","webp","bmp","gif","tif","tiff","ico","exe","dll","lnk"]
}];

/**
 * 统一工具栏（命令层）：红绿灯居左、窗口标题居中、
 * 导入 / 导出 / 设置靠右按任务分组。内容与导航保持平静，
 * 玻璃只出现在这层系统铬上。
 */
export function TitleBar(): JSX.Element {
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);
  const setSettingsOpen = useSettingsStore((s) => s.setSettingsOpen);
  const itemCount = useIconForgeStore((s) => s.items.length);
  const isExporting = useIconForgeStore((s) => s.isExporting);
  const exportAsIco = useIconForgeStore((s) => s.exportAsIco);

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

  const handleImport = useCallback(async () => {
    const selected = await open({ multiple: true, directory: false, filters: IMPORT_FILTERS });
    if (selected?.length) await useIconForgeStore.getState().importPaths(selected);
  }, []);

  return (
    <header
      className={`toolbar${focused ? "" : " toolbar--unfocused"}`}
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

      <div className="toolbar__spacer" data-tauri-drag-region />
      <span className="toolbar__title">IconForge</span>

      <div className="toolbar__group">
        <button
          type="button"
          onClick={() => void handleImport()}
          className="btn btn--ghost"
          title="导入图片、EXE、DLL、LNK (Ctrl+O)"
        >
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="17 8 12 3 7 8" />
            <line x1="12" y1="3" x2="12" y2="15" />
          </svg>
          导入
        </button>

        <span className="toolbar__divider" />

        <button
          type="button"
          onClick={() => void exportAsIco()}
          disabled={itemCount === 0 || isExporting}
          className="btn btn--primary"
          title="导出当前选中图标为 ICO"
        >
          <svg
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          {isExporting ? "导出中..." : "导出 ICO"}
        </button>

        <span className="toolbar__divider" />

        <button
          type="button"
          className="toolbar__action"
          title="软件设置"
          aria-label="软件设置"
          onClick={() => setSettingsOpen(true)}
        >
          <svg
            width="15"
            height="15"
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
      </div>
    </header>
  );
}
