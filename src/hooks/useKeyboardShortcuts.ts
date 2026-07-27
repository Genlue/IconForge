import { useEffect } from "react";

export function useKeyboardShortcuts(): void {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      // Placeholder: handle keyboard shortcuts in later stages
      if (e.key === "Delete" || e.key === "Backspace") {
        // removeItem will be handled here later
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
