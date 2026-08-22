import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Maximize2, Minimize2, Minus, X } from "lucide-react";

/**
 * Custom window title bar (native decorations are disabled):
 * - background matches the app surface (`--surface-app`)
 * - macOS-style traffic light buttons in the top-left corner
 * - the bar itself is a Tauri drag region (`data-tauri-drag-region`);
 *   dragging and double-click-to-maximize are handled natively
 */
export function TitleBar(): JSX.Element {
  const [maximized, setMaximized] = useState(false);
  const [focused, setFocused] = useState(true);

  useEffect(() => {
    const win = getCurrentWindow();
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    const syncMaximized = (): void => {
      void win.isMaximized().then((value) => {
        if (!disposed) setMaximized(value);
      });
    };
    syncMaximized();

    void win.isFocused().then((value) => {
      if (!disposed) setFocused(value);
    });

    void win.onResized(syncMaximized).then((unlisten) => unlisteners.push(unlisten));
    void win
      .onFocusChanged(({ payload }) => {
        if (!disposed) setFocused(payload);
      })
      .then((unlisten) => unlisteners.push(unlisten));

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
    };
  }, []);

  const win = getCurrentWindow();

  return (
    <div
      className={`titlebar${focused ? "" : " titlebar--unfocused"}`}
      data-tauri-drag-region
    >
      <div className="traffic-lights" data-tauri-drag-region>
        <button
          type="button"
          className="traffic-light traffic-light--close"
          title="关闭"
          aria-label="关闭窗口"
          onClick={() => void win.close()}
        >
          <X size={8} strokeWidth={3} strokeLinecap="round" strokeLinejoin="round" />
        </button>
        <button
          type="button"
          className="traffic-light traffic-light--minimize"
          title="最小化"
          aria-label="最小化窗口"
          onClick={() => void win.minimize()}
        >
          <Minus size={8} strokeWidth={3} strokeLinecap="round" />
        </button>
        <button
          type="button"
          className="traffic-light traffic-light--maximize"
          title={maximized ? "还原" : "最大化"}
          aria-label={maximized ? "还原窗口" : "最大化窗口"}
          onClick={() => void win.toggleMaximize()}
        >
          {maximized ? (
            <Minimize2 size={7.5} strokeWidth={2.75} strokeLinecap="round" strokeLinejoin="round" />
          ) : (
            <Maximize2 size={7.5} strokeWidth={2.75} strokeLinecap="round" strokeLinejoin="round" />
          )}
        </button>
      </div>
    </div>
  );
}
