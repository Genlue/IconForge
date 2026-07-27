import { useEffect } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useIconForgeStore } from "../store/useIconForgeStore";

export function useGlobalFileDrop(): void {
  const importPaths = useIconForgeStore((s) => s.importPaths);
  const setDragActive = useIconForgeStore((s) => s.setDragActive);

  useEffect(() => {
    let cancelled = false;
    const webview = getCurrentWebviewWindow();

    const preventDefaults = (e: Event) => {
      e.preventDefault();
    };
    window.addEventListener("dragover", preventDefaults);
    window.addEventListener("drop", preventDefaults);

    const setup = async () => {
      const unlisten = await webview.onDragDropEvent((event) => {
        if (cancelled) return;
        const type = event.payload.type;
        if (type === "enter" || type === "over") {
          setDragActive(true);
        } else if (type === "leave") {
          setDragActive(false);
        } else if (type === "drop") {
          setDragActive(false);
          importPaths(event.payload.paths);
        }
      });
      if (cancelled) {
        unlisten();
      } else {
        return unlisten;
      }
    };

    const unlistenPromise = setup();

    return () => {
      cancelled = true;
      window.removeEventListener("dragover", preventDefaults);
      window.removeEventListener("drop", preventDefaults);
      unlistenPromise.then((unlisten) => unlisten?.());
    };
  }, [importPaths, setDragActive]);
}
