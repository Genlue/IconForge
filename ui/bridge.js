/* --------------------------------------------------------------------------
   IconForge ↔ Tauri 宿主桥
   —— 只碰 Tauri 的 window / webview 事件，不碰任何业务逻辑。
   wasm 侧（ui/src/bridge.rs）通过 window.__iconforge 调这里。

   两个约定，都是踩过的坑：

   1) **句柄必须懒取**。`withGlobalTauri` 注入的 window.__TAURI__ 和这份普通
      脚本谁先执行没有保证；在加载时抓一次 `getCurrentWindow()`，抓空就永久
      失效 —— 窗口按钮、装饰切换、拖放会一起静默失灵，而且不报错。
      所以每个函数进来先取一次。

   2) **失败要带原因**。返回 `{ ok: false, error }` 而不是 `false`，
      上层才能把「权限没开」「API 不存在」「窗口还没建好」区分开。
   -------------------------------------------------------------------------- */
(() => {
  const noop = () => {};
  const resolved = (value) => Promise.resolve(value);

  function currentWindow() {
    try {
      return window.__TAURI__?.window?.getCurrentWindow?.() ?? null;
    } catch {
      return null;
    }
  }

  function currentWebview() {
    try {
      return window.__TAURI__?.webviewWindow?.getCurrentWebviewWindow?.() ?? null;
    } catch {
      return null;
    }
  }

  const api = {
    windowAction(action) {
      const win = currentWindow();
      if (!win) return resolved();
      if (action === "close") return win.close();
      if (action === "minimize") return win.minimize();
      if (action === "toggle-maximize") return win.toggleMaximize();
      return resolved();
    },

    isMaximized() {
      const win = currentWindow();
      return win ? win.isMaximized() : resolved(false);
    },

    isFocused() {
      const win = currentWindow();
      return win ? win.isFocused() : resolved(true);
    },

    /* 窗口最大化 / 焦点变化。
       焦点用**事件**推，不用 isFocused() 轮询：
       挂载那一刻查一次 isFocused() 很容易赶在系统激活窗口之前，拿到 false 之后
       就再也没机会纠正（激活事件可能早于监听注册）。这里以「默认有焦点」起步，
       只认真正的 blur/focus；DOM 的 window.focus/blur 与 Tauri 的 onFocusChanged
       双保险，谁先到都算。 */
    onWindowState(cb) {
      let focused = true;
      let maximized = false;
      const push = () => cb(maximized, focused);

      const win = currentWindow();
      const onDomFocus = () => {
        focused = true;
        push();
      };
      const onDomBlur = () => {
        focused = false;
        push();
      };
      window.addEventListener("focus", onDomFocus);
      window.addEventListener("blur", onDomBlur);
      document.addEventListener("visibilitychange", () => {
        focused = document.visibilityState === "visible";
        push();
      });

      let unlistenFocus = null;
      let unlistenResize = null;

      if (win) {
        win.onFocusChanged((event) => {
          focused = event?.payload !== false;
          push();
        })
          .then((un) => {
            unlistenFocus = un;
          })
          .catch(() => {});

        win.onResized(() => {
          win.isMaximized()
            .then((value) => {
              maximized = value;
              push();
            })
            .catch(() => {});
        })
          .then((un) => {
            unlistenResize = un;
          })
          .catch(() => {});

        win.isMaximized()
          .then((value) => {
            maximized = value;
            push();
          })
          .catch(() => {});
      }

      push();

      return () => {
        window.removeEventListener("focus", onDomFocus);
        window.removeEventListener("blur", onDomBlur);
        if (unlistenFocus) unlistenFocus();
        if (unlistenResize) unlistenResize();
      };
    },

    /* 系统级拖放。Tauri 会拦截 HTML5 drop，所以必须走原生事件。 */
    onDragDrop(cb) {
      const webview = currentWebview();
      if (!webview) return noop;
      let unlisten = null;
      webview
        .onDragDropEvent((event) => {
          const payload = event.payload || {};
          if (payload.type === "enter" || payload.type === "over") {
            cb("over", []);
          } else if (payload.type === "leave") {
            cb("leave", []);
          } else if (payload.type === "drop") {
            cb("drop", payload.paths || []);
          }
        })
        .then((un) => {
          unlisten = un;
        })
        .catch((err) => console.error("[bridge] onDragDropEvent failed", err));
      return () => {
        if (unlisten) unlisten();
      };
    },
  };

  window.__iconforge = api;
})();
