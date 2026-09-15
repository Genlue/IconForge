import { useEffect, useState, useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface TitleBarProps {
  isDirty?: boolean;
}

export function TitleBar({ isDirty = false }: TitleBarProps): JSX.Element {
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);
  const [altPressed, setAltPressed] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    const syncMaximized = (): void => {
      void win.isMaximized().then((val) => {
        if (!disposed) setMaximized(val);
      });
    };
    syncMaximized();

    void win.isFocused().then((val) => {
      if (!disposed) setFocused(val);
    });

    void win.onResized(syncMaximized).then((u) => unlisteners.push(u));
    void win.onFocusChanged(({ payload }) => {
      if (!disposed) setFocused(payload);
    }).then((u) => unlisteners.push(u));

    const handleKeyDown = (e: KeyboardEvent): void => {
      if (e.key === "Alt" || e.altKey) setAltPressed(true);
    };
    const handleKeyUp = (e: KeyboardEvent): void => {
      if (e.key === "Alt" || !e.altKey) setAltPressed(false);
    };
    const handleWindowBlur = (): void => {
      setAltPressed(false);
    };

    window.addEventListener("keydown", handleKeyDown);
    window.addEventListener("keyup", handleKeyUp);
    window.addEventListener("blur", handleWindowBlur);

    return () => {
      disposed = true;
      for (const u of unlisteners) u();
      window.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("keyup", handleKeyUp);
      window.removeEventListener("blur", handleWindowBlur);
    };
  }, []);

  const win = getCurrentWindow();

  const handleClose = useCallback(() => {
    void win.close();
  }, [win]);

  const handleMinimize = useCallback(() => {
    void win.minimize();
  }, [win]);

  const handleMaximize = useCallback(() => {
    void win.toggleMaximize();
  }, [win]);

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
          onClick={handleClose}
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
          {isDirty && <span className="traffic-dirty-dot" />}
        </button>

        {/* 2. 最小化按钮 */}
        <button
          type="button"
          className="traffic-light traffic-light--minimize"
          title="最小化"
          aria-label="最小化窗口"
          onClick={handleMinimize}
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

        {/* 3. 全屏 / 最大化 / 缩放按钮 */}
        <button
          type="button"
          className="traffic-light traffic-light--maximize"
          title={altPressed ? "缩放" : maximized ? "还原" : "全屏"}
          aria-label={altPressed ? "缩放窗口" : maximized ? "还原窗口" : "全屏窗口"}
          onClick={handleMaximize}
        >
          {altPressed ? (
            <svg
              className="traffic-glyph"
              width="12"
              height="12"
              viewBox="0 0 12 12"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.15"
              strokeLinecap="round"
            >
              <path d="M6 3.4 v5.2 M3.4 6 h5.2" />
            </svg>
          ) : maximized ? (
            /* 窗口已最大化态（图 2）：向心内缩对顶双三角，点击还原窗口 */
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
            /* 窗口未最大化态（图 1）：外向扩张对角双三角（左上 ◤ 与 右下 ◢），点击最大化/全屏 */
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
    </header>
  );
}
