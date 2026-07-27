import { useState, useEffect, useRef } from "react";
import type { BrushStroke, RenderConfig, UpscaleConfig } from "../types/domain";
import type { CommandError } from "../types/commands";
import { commands } from "../lib/tauri";
import { normalizeInvokeError } from "../types/errors";

export interface DebouncedPreviewState {
  pngDataUrl: string | null;
  processedSourceWidth: number | null;
  processedSourceHeight: number | null;
  loading: boolean;
  error: CommandError | null;
}

export function useDebouncedPreview(
  sourcePath: string | null,
  config: RenderConfig,
  upscaleConfig: UpscaleConfig,
  brushStrokes: BrushStroke[],
  previewSize: number,
  delayMs: number = 250,
): DebouncedPreviewState {
  const [state, setState] = useState<DebouncedPreviewState>({
    pngDataUrl: null,
    processedSourceWidth: null,
    processedSourceHeight: null,
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
          upscaleConfig,
          brushStrokes,
          previewSize,
        });
        if (generationRef.current === generation) {
          setState({
            pngDataUrl: `data:image/png;base64,${response.pngBase64}`,
            processedSourceWidth: response.processedSourceWidth,
            processedSourceHeight: response.processedSourceHeight,
            loading: false,
            error: null,
          });
        }
      } catch (err) {
        if (generationRef.current === generation) {
          setState({
            pngDataUrl: null,
            processedSourceWidth: null,
            processedSourceHeight: null,
            loading: false,
            error: normalizeInvokeError(err),
          });
        }
      }
    }, delayMs);
    return () => clearTimeout(timer);
  }, [sourcePath, config, upscaleConfig, brushStrokes, previewSize, delayMs]);

  return state;
}
