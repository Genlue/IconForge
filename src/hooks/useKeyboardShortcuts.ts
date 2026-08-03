import { useEffect } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useIconForgeStore } from "../store/useIconForgeStore";

export function useKeyboardShortcuts(): void {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "o") {
        e.preventDefault();
        (async () => {
          const selected = await open({
            multiple: true,
            directory: false,
            filters: [{
              name: "支持的文件",
              extensions: ["png","jpg","jpeg","webp","bmp","gif","tif","tiff","ico","exe","dll","lnk"]
            }]
          });
          if (selected && selected.length > 0) {
            await useIconForgeStore.getState().importPaths(selected);
          }
        })();
        return;
      }

      // Placeholder: handle keyboard shortcuts in later stages
      if (e.key === "Delete" || e.key === "Backspace") {
        // removeItem will be handled here later
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
