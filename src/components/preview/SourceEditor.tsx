import { useCallback, useEffect, useRef, useState } from "react";
import { useIconForgeStore } from "../../store/useIconForgeStore";
import type { BrushPoint } from "../../types/domain";
import { Checkerboard } from "./Checkerboard";
import { Spinner } from "../common/Spinner";
import { RangeField } from "../controls/RangeField";
import { commands } from "../../lib/tauri";
import { drawSelectionOverlay, loadImage } from "../../lib/selectionOverlay";

/**
 * Full-screen original-image editor. Shows the source image (after upscale)
 * with any applied eraser strokes and wand deletions, over a checkerboard
 * background. Editing here operates directly on the original coordinates, so
 * the wand and eraser never drift from what the user sees.
 */
export function SourceEditor(): JSX.Element | null {
  const selectedItemId = useIconForgeStore((s) => s.selectedItemId);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const items = useIconForgeStore((s) => s.items);
  const setSourceEditorOpen = useIconForgeStore((s) => s.setSourceEditorOpen);
  const previewTool = useIconForgeStore((s) => s.previewTool);
  const setPreviewTool = useIconForgeStore((s) => s.setPreviewTool);
  const addEraserStroke = useIconForgeStore((s) => s.addEraserStroke);
  const wandSelection = useIconForgeStore((s) => s.wandSelection);
  const setWandSelection = useIconForgeStore((s) => s.setWandSelection);
  const clearWandSelection = useIconForgeStore((s) => s.clearWandSelection);
  const deleteWandSelection = useIconForgeStore((s) => s.deleteWandSelection);
  const updateRenderConfig = useIconForgeStore((s) => s.updateRenderConfig);
  const setWandTolerance = useIconForgeStore((s) => s.setWandTolerance);
  const updateEraserSettings = useIconForgeStore((s) => s.updateEraserSettings);

  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [srcUrl, setSrcUrl] = useState<string | null>(null);
  const [dimensions, setDimensions] = useState<{ w: number; h: number } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [drawingPoints, setDrawingPoints] = useState<BrushPoint[]>([]);
  const [selecting, setSelecting] = useState(false);

  const selectedItem = items.find((i) => i.id === selectedItemId) ?? null;

  // Fetch the current edited source image whenever the source edits change.
  useEffect(() => {
    if (!selectedItem) {
      setSrcUrl(null);
      return;
    }
    let cancelled = false;
    setLoading(true);
    setError(null);
    (async () => {
      try {
        const response = await commands.renderSourceImage({
          sourcePath: selectedItem.sourcePath,
          upscaleConfig: itemConfig?.upscaleConfig ?? { enabled: false, scale: 2, denoise: 1, model: "se" },
          wandStrokes: itemConfig?.wandStrokes ?? [],
          eraserStrokes: itemConfig?.eraserStrokes ?? [],
        });
        if (cancelled) return;
        setSrcUrl(`data:image/png;base64,${response.pngBase64}`);
        setDimensions({ w: response.width, h: response.height });
      } catch (err) {
        if (!cancelled) setError(String(err));
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [selectedItem?.id, itemConfig?.upscaleConfig, itemConfig?.wandStrokes, itemConfig?.eraserStrokes]);

  // Load the fetched image into the canvas.
  useEffect(() => {
    if (!srcUrl) return;
    (async () => {
      try {
        const image = await loadImage(srcUrl);
        imageRef.current = image;
        redrawSource(canvasRef.current, image, null);
      } catch {
        // ignore
      }
    })();
  }, [srcUrl]);

  // Marching ants for the pending wand selection.
  useEffect(() => {
    if (!wandSelection) {
      drawSelectionOverlay(overlayRef.current, null, 0);
      return;
    }
    (async () => {
      try {
        const image = await loadImage(`data:image/png;base64,${wandSelection.maskPngBase64}`);
        let phase = 0;
        drawSelectionOverlay(overlayRef.current, image, phase);
        const timer = window.setInterval(() => {
          phase = (phase + 4) % 16;
          drawSelectionOverlay(overlayRef.current, image, phase);
        }, 90);
        return () => window.clearInterval(timer);
      } catch {
        // ignore
      }
    })();
  }, [wandSelection]);

  const pointFromEvent = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    // The source image fills the canvas; map the click back to 256-space so
    // the backend applies it to the original image coordinates 1:1.
    return {
      x: ((event.clientX - rect.left) / rect.width) * 256,
      y: ((event.clientY - rect.top) / rect.height) * 256,
    };
  }, []);

  const handlePointerDown = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if (!imageRef.current || !itemConfig) return;
    const point = pointFromEvent(event);
    if (previewTool === "magic-wand") {
      setSelecting(true);
      setWandSelection(null);
      (async () => {
        try {
          const response = await commands.computeWandSelection({
            sourcePath: selectedItem!.sourcePath,
            upscaleConfig: itemConfig.upscaleConfig,
            point,
            tolerance: itemConfig.wandTolerance,
            wandStrokes: itemConfig.wandStrokes,
            eraserStrokes: itemConfig.eraserStrokes,
          });
          setWandSelection({
            x: point.x,
            y: point.y,
            tolerance: itemConfig.wandTolerance,
            maskPngBase64: response.maskPngBase64,
          });
        } catch {
          setWandSelection(null);
        }
        setSelecting(false);
      })();
      return;
    }
    if (previewTool !== "source-eraser") return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrawingPoints([point]);
    redrawSource(event.currentTarget, imageRef.current, {
      points: [point],
      size: itemConfig.eraserSize,
    });
  }, [itemConfig, pointFromEvent, previewTool, selectedItem, setWandSelection]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if (previewTool !== "source-eraser" || drawingPoints.length === 0 || !imageRef.current || !itemConfig) {
      return;
    }
    const points = [...drawingPoints, pointFromEvent(event)];
    setDrawingPoints(points);
    redrawSource(event.currentTarget, imageRef.current, {
      points,
      size: itemConfig.eraserSize,
    });
  }, [drawingPoints, itemConfig, pointFromEvent, previewTool]);

  const finishStroke = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if (previewTool !== "source-eraser" || drawingPoints.length === 0 || !itemConfig) return;
    const points = drawingPoints.length === 1
      ? [drawingPoints[0]!, pointFromEvent(event)]
      : drawingPoints;
    addEraserStroke({
      points,
      size: itemConfig.eraserSize,
      hardness: itemConfig.eraserHardness,
    });
    setDrawingPoints([]);
  }, [addEraserStroke, drawingPoints, itemConfig, pointFromEvent, previewTool]);

  if (!selectedItem || !itemConfig) return null;

  return (
    <div className="fixed inset-0 z-50 flex flex-col bg-white/90 backdrop-blur-sm dark:bg-black/80">
      <div className="flex items-center justify-between gap-4 border-b border-[var(--border-hairline)] px-5 py-3">
        <div className="flex items-center gap-3">
          <h2 className="text-sm font-semibold text-[var(--text-primary)]">原图编辑</h2>
          <span className="text-[11px] text-[var(--text-secondary)]">
            {selectedItem.displayName} · {dimensions ? `${dimensions.w}×${dimensions.h}` : ""}
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setPreviewTool("none")}
            className={`rounded-lg border px-3 py-1.5 text-xs ${previewTool === "none" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40 text-[var(--text-primary)]"}`}
          >
            选择
          </button>
          <button
            type="button"
            onClick={() => setPreviewTool(previewTool === "magic-wand" ? "none" : "magic-wand")}
            className={`rounded-lg border px-3 py-1.5 text-xs ${previewTool === "magic-wand" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40 text-[var(--text-primary)]"}`}
          >
            魔棒
          </button>
          <button
            type="button"
            onClick={() => setPreviewTool(previewTool === "source-eraser" ? "none" : "source-eraser")}
            className={`rounded-lg border px-3 py-1.5 text-xs ${previewTool === "source-eraser" ? "border-[var(--accent)] bg-[var(--accent)] text-white" : "border-[var(--border-hairline)] bg-white/40 text-[var(--text-primary)]"}`}
          >
            橡皮擦
          </button>
          {previewTool === "magic-wand" && (
            <>
              <button
                type="button"
                disabled={!wandSelection || selecting}
                onClick={deleteWandSelection}
                className="rounded-lg border border-red-200 bg-red-50 px-3 py-1.5 text-xs text-red-600 disabled:opacity-40"
              >
                删除选中 (Del)
              </button>
              <button
                type="button"
                disabled={!wandSelection}
                onClick={clearWandSelection}
                className="rounded-lg border border-[var(--border-hairline)] bg-white/40 px-3 py-1.5 text-xs text-[var(--text-primary)] disabled:opacity-40"
              >
                取消选区
              </button>
            </>
          )}
          <button
            type="button"
            onClick={() => {
              setPreviewTool("none");
              clearWandSelection();
              setSourceEditorOpen(false);
            }}
            className="rounded-lg bg-[var(--accent)] px-4 py-1.5 text-xs text-white"
          >
            完成
          </button>
        </div>
      </div>
      {/* Source-editing parameters live here so adjustments happen while looking
          at the original image. */}
      <div className="flex flex-wrap items-end gap-x-6 gap-y-2 border-b border-[var(--border-hairline)] px-5 py-3">
        <label className="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
          <input
            type="checkbox"
            checked={itemConfig.renderConfig.autoCutout.enabled}
            onChange={(e) => updateRenderConfig({ autoCutout: { ...itemConfig.renderConfig.autoCutout, enabled: e.target.checked } })}
            className="rounded"
          />
          自动抠图
        </label>
        <div className="w-32">
          <RangeField
            id="editor-cutout-tolerance"
            label="背景容差"
            value={itemConfig.renderConfig.autoCutout.tolerance}
            min={0}
            max={100}
            step={1}
            disabled={!itemConfig.renderConfig.autoCutout.enabled}
            onChange={(tolerance) => updateRenderConfig({ autoCutout: { ...itemConfig.renderConfig.autoCutout, tolerance } })}
          />
        </div>
        <div className="w-32">
          <RangeField
            id="editor-cutout-feather"
            label="边缘羽化"
            value={itemConfig.renderConfig.autoCutout.feather}
            min={0}
            max={32}
            step={1}
            disabled={!itemConfig.renderConfig.autoCutout.enabled}
            onChange={(feather) => updateRenderConfig({ autoCutout: { ...itemConfig.renderConfig.autoCutout, feather } })}
          />
        </div>
        <div className="w-32">
          <RangeField
            id="editor-wand-tolerance"
            label="魔棒容差"
            value={itemConfig.wandTolerance}
            min={0}
            max={100}
            step={1}
            onChange={(wandTolerance) => {
              setWandTolerance(wandTolerance);
              clearWandSelection();
            }}
          />
        </div>
        <div className="w-32">
          <RangeField
            id="editor-eraser-size"
            label="橡皮擦大小"
            value={itemConfig.eraserSize}
            min={1}
            max={64}
            step={1}
            onChange={(size) => updateEraserSettings({ size })}
          />
        </div>
        <div className="w-32">
          <RangeField
            id="editor-eraser-hardness"
            label="橡皮擦硬度"
            value={itemConfig.eraserHardness}
            min={0}
            max={100}
            step={1}
            onChange={(hardness) => updateEraserSettings({ hardness })}
          />
        </div>
      </div>
      <div className="flex flex-1 items-center justify-center overflow-auto p-6">
        <div className="relative max-h-full max-w-full overflow-hidden rounded-xl border border-[var(--border-hairline)] shadow-lg">
          <div className="pointer-events-none absolute inset-0 z-0">
            <Checkerboard />
          </div>
          <canvas
            ref={canvasRef}
            width={420}
            height={420}
            className="relative z-10 touch-none"
            style={{
              cursor: previewTool === "none" ? "default" : "crosshair",
              maxWidth: "min(80vw, 70vh)",
              maxHeight: "70vh",
              aspectRatio: dimensions ? `${dimensions.w} / ${dimensions.h}` : "1 / 1",
              width: "auto",
              height: "auto",
            }}
            onPointerDown={handlePointerDown}
            onPointerMove={handlePointerMove}
            onPointerUp={finishStroke}
            onPointerCancel={() => setDrawingPoints([])}
          />
          <canvas
            ref={overlayRef}
            width={420}
            height={420}
            className="pointer-events-none absolute inset-0 z-20 h-full w-full"
          />
          {loading && (
            <div className="absolute inset-0 z-20 flex items-center justify-center bg-white/55 backdrop-blur-sm">
              <Spinner label="加载原图..." />
            </div>
          )}
        </div>
      </div>
      <div className="border-t border-[var(--border-hairline)] px-5 py-2 text-center text-[11px] text-[var(--text-secondary)]">
        {error ? (
          <span className="text-red-600">{error}</span>
        ) : previewTool === "magic-wand"
          ? "点击选中相似区域（蚂蚁线），按 Delete 或点「删除选中」删除为透明；容差可在左侧「原图处理 → 魔棒」调整。"
          : previewTool === "source-eraser"
            ? "涂抹即擦除原图（变为透明），大小/硬度可在左侧「原图处理 → 橡皮擦」调整。"
            : "选择工具后在此原图上修改，背景棋盘格表示透明。修改结果会进入预览、PNG 和最终 ICO。"}
      </div>
    </div>
  );
}

function redrawSource(
  canvas: HTMLCanvasElement | null,
  image: HTMLImageElement,
  stroke: { points: BrushPoint[]; size: number } | null,
): void {
  const context = canvas?.getContext("2d");
  if (!canvas || !context) return;
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.drawImage(image, 0, 0, canvas.width, canvas.height);
  if (!stroke || stroke.points.length === 0) return;
  context.save();
  context.globalCompositeOperation = "destination-out";
  context.globalAlpha = 1;
  context.strokeStyle = "rgba(0,0,0,1)";
  context.lineWidth = (stroke.size * canvas.width) / 256;
  context.lineCap = "round";
  context.lineJoin = "round";
  context.beginPath();
  context.moveTo((stroke.points[0]!.x * canvas.width) / 256, (stroke.points[0]!.y * canvas.height) / 256);
  for (const point of stroke.points.slice(1)) {
    context.lineTo((point.x * canvas.width) / 256, (point.y * canvas.height) / 256);
  }
  context.stroke();
  context.restore();
}