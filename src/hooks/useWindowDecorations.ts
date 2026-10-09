import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettingsStore } from "../store/useSettingsStore";

const win = getCurrentWindow();

export type DecorationStatus = "ok" | "unsupported";

/**
 * Keeps the native window frame in sync with the selected title bar style.
 *
 * "macos" renders its own traffic lights, so the native frame must be off.
 * "windows" hands the frame back to the OS, so the custom bar is not rendered
 * at all (see AppShell) and this effect turns decorations on.
 *
 * Returns whether the last switch actually took effect, so the UI can warn
 * instead of leaving the user with no title bar at all.
 */
export function useWindowDecorations(): DecorationStatus {
  const titleBarStyle = useSettingsStore((s) => s.titleBarStyle);
  const [status, setStatus] = useState<DecorationStatus>("ok");

  useEffect(() => {
    const wantDecorations = titleBarStyle === "windows";
    let disposed = false;

    void (async () => {
      try {
        const before = await win.isDecorated();
        if (before !== wantDecorations) {
          await win.setDecorations(wantDecorations);
        }
        // Verify rather than trust: some platform/window combinations accept
        // the call but never repaint the frame.
        const after = await win.isDecorated();
        if (disposed) return;
        setStatus(after === wantDecorations ? "ok" : "unsupported");
        if (after !== wantDecorations) {
          console.error(
            `[shell] setDecorations(${wantDecorations}) did not take effect; window still reports ${after}`,
          );
        }
      } catch (err) {
        console.error("[shell] failed to change window decorations", err);
        if (!disposed) setStatus("unsupported");
      }
    })();

    return () => {
      disposed = true;
    };
  }, [titleBarStyle]);

  return status;
}
