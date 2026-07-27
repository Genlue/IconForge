import { useState, useEffect, useRef } from "react";
import type { RenderConfig } from "../types/domain";
import type { CommandError } from "../types/commands";
import { commands } from "../lib/tauri";
import { normalizeInvokeError } from "../types/errors";

export interface DebouncedPreviewState {
  pngDataUrl: string | null;
  loading: boolean;
  error: CommandError | null;
}

export function useDebouncedPreview(
  sourcePath: string | null,
  config: RenderConfig,
  previewSize: number,
  delayMs: number = 250,
): DebouncedPreviewState {
  const [state, setState] = useState<DebouncedPreviewState>({
    pngDataUrl: null,
    loading: false,
    error: null,
  });
  const generationRef = useRef(0);

  useEffect(() => {
    if (!sourcePath) return;
    const generation = ++generationRef.current;
    const timer = setTimeout(async () => {
      setState((prev) => ({ ...prev, loading: true }));
      try {
        const response = await commands.renderPreview({
          sourcePath,
          renderConfig: config,
          previewSize,
        });
        if (generationRef.current === generation) {
          setState({
            pngDataUrl: `data:image/png;base64,${response.pngBase64}`,
            loading: false,
            error: null,
          });
        }
      } catch (err) {
        if (generationRef.current === generation) {
          setState({
            pngDataUrl: null,
            loading: false,
            error: normalizeInvokeError(err),
          });
        }
      }
    }, delayMs);
    return () => clearTimeout(timer);
  }, [sourcePath, config, previewSize, delayMs]);

  return state;
}
