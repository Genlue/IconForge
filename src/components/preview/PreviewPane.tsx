import { useCallback, useEffect, useRef, useState } from "react";
import { useIconForgeStore } from "../../store/useIconForgeStore";
import type { BrushPoint, BrushStroke } from "../../types/domain";
import { Checkerboard } from "./Checkerboard";
import { EmptyDropZone } from "./EmptyDropZone";
import { Spinner } from "../common/Spinner";
import { useDebouncedPreview } from "../../hooks/useDebouncedPreview";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../../constants/defaults";
import { drawSelectionOverlay } from "../../lib/selectionOverlay";
import { SourceEditor } from "./SourceEditor";

export function PreviewPane(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const previewTool = useIconForgeStore((s) => s.previewTool);
  const addBrushStroke = useIconForgeStore((s) => s.addBrushStroke);
  const applyPickedColor = useIconForgeStore((s) => s.applyPickedColor);
  const wandSelection = useIconForgeStore((s) => s.wandSelection);
  const deleteWandSelection = useIconForgeStore((s) => s.deleteWandSelection);
  const dragActive = useIconForgeStore((s) => s.dragActive);
  const sourceEditorOpen = useIconForgeStore((s) => s.sourceEditorOpen);
  const setSourceEditorOpen = useIconForgeStore((s) => s.setSourceEditorOpen);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [drawingPoints, setDrawingPoints] = useState<BrushPoint[]>([]);

  const selectedItem = items.find((i) => i.id === selectedId) ?? items[0] ?? null;
  const renderConfig = itemConfig?.renderConfig ?? DEFAULT_RENDER_CONFIG;
  const upscaleConfig = itemConfig?.upscaleConfig ?? DEFAULT_UPSCALE_CONFIG;
  const brushStrokes = itemConfig?.brushStrokes ?? [];
  const wandStrokes = itemConfig?.wandStrokes ?? [];
  const eraserStrokes = itemConfig?.eraserStrokes ?? [];
  const preview = useDebouncedPreview(
    selectedItem?.sourcePath ?? null,
    renderConfig,
    upscaleConfig,
    brushStrokes,
    wandStrokes,
    eraserStrokes,
    420,
  );

  useEffect(() => {
    if (!preview.pngDataUrl) {
      imageRef.current = null;
      return;
    }
    const image = new Image();
    image.onload = () => {
      imageRef.current = image;
      redrawCanvas(canvasRef.current, image, null);
    };
    image.src = preview.pngDataUrl;
  }, [preview.pngDataUrl]);

  // Marching-ants overlay for a pending wand selection (only meaningful while
  // the source editor is open; keep it when closing so the user can review).
  useEffect(() => {
    if (!wandSelection) {
      drawSelectionOverlay(overlayRef.current, null, 0);
      return;
    }
    const image = new Image();
    let phase = 0;
    let timer: number | undefined;
    image.onload = () => {
      timer = window.setInterval(() => {
        phase = (phase + 4) % 16;
        drawSelectionOverlay(overlayRef.current, image, phase);
      }, 90);
      drawSelectionOverlay(overlayRef.current, image, phase);
    };
    image.src = `data:image/png;base64,${wandSelection.maskPngBase64}`;
    return () => {
      if (timer !== undefined) window.clearInterval(timer);
    };
  }, [wandSelection]);

  const pointFromEvent = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: ((event.clientX - rect.left) / rect.width) * 256,
      y: ((event.clientY - rect.top) / rect.height) * 256,
    };
  }, []);

  const handlePointerDown = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if (!imageRef.current || !itemConfig) return;
    const point = pointFromEvent(event);
    if (previewTool === "eyedropper") {
      const context = event.currentTarget.getContext("2d", { willReadFrequently: true });
      if (!context) return;
      const rect = event.currentTarget.getBoundingClientRect();
      const x = Math.max(0, Math.min(419, Math.floor(((event.clientX - rect.left) / rect.width) * 420)));
      const y = Math.max(0, Math.min(419, Math.floor(((event.clientY - rect.top) / rect.height) * 420)));
      const pixel = context.getImageData(x, y, 1, 1).data;
      applyPickedColor(`#${toHex(pixel[0] ?? 0)}${toHex(pixel[1] ?? 0)}${toHex(pixel[2] ?? 0)}`);
      return;
    }
    if (previewTool === "magic-wand") {
      // Wand editing happens in the source editor where coordinates match the
      // original image; selecting here would drift.
      setSourceEditorOpen(true);
      return;
    }
    if (previewTool === "source-eraser") {
      setSourceEditorOpen(true);
      return;
    }
    if (previewTool !== "brush" && previewTool !== "eraser") return;
    event.currentTarget.setPointerCapture(event.pointerId);
    setDrawingPoints([point]);
    redrawCanvas(event.currentTarget, imageRef.current, {
      points: [point],
      color: itemConfig.brushColor,
      size: itemConfig.brushSize,
      opacity: 1,
      mode: previewTool === "eraser" ? "erase" : "paint",
      clipToMask: itemConfig.brushClipToMask,
    });
  }, [applyPickedColor, itemConfig, pointFromEvent, previewTool, setSourceEditorOpen]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if ((previewTool !== "brush" && previewTool !== "eraser") || drawingPoints.length === 0 || !imageRef.current || !itemConfig) {
      return;
    }
    const points = [...drawingPoints, pointFromEvent(event)];
    setDrawingPoints(points);
    redrawCanvas(event.currentTarget, imageRef.current, {
      points,
      color: itemConfig.brushColor,
      size: itemConfig.brushSize,
      opacity: 1,
      mode: previewTool === "eraser" ? "erase" : "paint",
      clipToMask: itemConfig.brushClipToMask,
    });
  }, [drawingPoints, itemConfig, pointFromEvent, previewTool]);

  const finishStroke = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if ((previewTool !== "brush" && previewTool !== "eraser") || drawingPoints.length === 0 || !itemConfig) return;
    const points = drawingPoints.length === 1
      ? [drawingPoints[0]!, pointFromEvent(event)]
      : drawingPoints;
    addBrushStroke({
      points,
      color: itemConfig.brushColor,
      size: itemConfig.brushSize,
      opacity: 1,
      mode: previewTool === "eraser" ? "erase" : "paint",
      clipToMask: itemConfig.brushClipToMask,
    });
    setDrawingPoints([]);
  }, [addBrushStroke, drawingPoints, itemConfig, pointFromEvent, previewTool]);

  const cursor = previewTool === "eyedropper"
    ? "crosshair"
    : previewTool === "brush"
      ? "crosshair"
      : previewTool === "eraser"
        ? "cell"
        : previewTool === "magic-wand" || previewTool === "source-eraser"
          ? "crosshair"
          : "default";

  return (
    <div className="relative flex h-full flex-col items-center justify-center gap-3 p-8">
      <div
        className="glass-panel relative flex aspect-square items-center justify-center overflow-hidden"
        style={{ height: 420, maxWidth: "100%" }}
      >
        <div className="pointer-events-none absolute inset-0 z-0">
          <Checkerboard />
        </div>
        {selectedItem ? (
          <>
            <canvas
              ref={canvasRef}
              width={420}
              height={420}
              className="relative z-10 h-full w-full touch-none"
              style={{ cursor }}
              onPointerDown={handlePointerDown}
              onPointerMove={handlePointerMove}
              onPointerUp={finishStroke}
              onPointerCancel={() => setDrawingPoints([])}
            />
            {(previewTool === "magic-wand") && (
              <canvas
                ref={overlayRef}
                width={420}
                height={420}
                className="pointer-events-none absolute inset-0 z-20 h-full w-full"
              />
            )}
            {preview.loading && (
              <div className="absolute inset-0 z-20 flex items-center justify-center bg-white/55 backdrop-blur-sm">
                <Spinner label={upscaleConfig.enabled ? "Real-CUGAN 处理中..." : "正在渲染..."} />
              </div>
            )}
            {preview.error && (
              <div className="absolute inset-x-5 bottom-5 z-20 rounded-xl border border-red-200 bg-red-50/95 p-3 text-xs leading-5 text-red-700">
                {preview.error.message}
              </div>
            )}
          </>
        ) : (
          <div className="relative z-10"><EmptyDropZone dragActive={dragActive} /></div>
        )}
      </div>
      {selectedItem && (
        <div className="flex max-w-full shrink-0 flex-wrap items-center justify-center gap-x-3 gap-y-1 text-[11px] text-[var(--text-secondary)]">
          <span>导入源图 {selectedItem.sourceWidth}×{selectedItem.sourceHeight}</span>
          {preview.processedSourceWidth && preview.processedSourceHeight && (
            <span>{upscaleConfig.enabled ? "增强源图" : "处理源图"} {preview.processedSourceWidth}×{preview.processedSourceHeight}</span>
          )}
          <span>主图 256×256</span>
          {previewTool === "eyedropper" && <span className="text-[var(--accent)]">点击预览取色</span>}
          {previewTool === "brush" && <span className="text-[var(--accent)]">在预览上拖动绘制</span>}
          {previewTool === "eraser" && <span className="text-[var(--accent)]">在预览上拖动擦除</span>}
          {(previewTool === "magic-wand" || previewTool === "source-eraser") && (
            <span className="text-[var(--accent)]">原图修改请在原图编辑界面中进行</span>
          )}
          <button
            type="button"
            onClick={() => setSourceEditorOpen(true)}
            className="rounded-lg border border-[var(--accent)] bg-[var(--accent)]/10 px-3 py-1 text-xs text-[var(--accent)] hover:bg-[var(--accent)]/15"
          >
            {wandStrokes.length + eraserStrokes.length > 0
              ? `编辑原图（${wandStrokes.length + eraserStrokes.length} 处修改）`
              : "编辑原图"}
          </button>
          {wandSelection && previewTool === "magic-wand" && (
            <button
              type="button"
              onClick={deleteWandSelection}
              className="rounded-lg border border-red-200 bg-red-50 px-2 py-1 text-xs text-red-600 hover:bg-red-100"
            >
              删除选中区域 (Del)
            </button>
          )}
        </div>
      )}
      {sourceEditorOpen && <SourceEditor />}
    </div>
  );
}

function redrawCanvas(
  canvas: HTMLCanvasElement | null,
  image: HTMLImageElement,
  stroke: BrushStroke | null,
): void {
  const context = canvas?.getContext("2d");
  if (!canvas || !context) return;
  context.clearRect(0, 0, canvas.width, canvas.height);
  context.drawImage(image, 0, 0, canvas.width, canvas.height);
  if (!stroke || stroke.points.length === 0) return;
  context.save();
  context.globalCompositeOperation = stroke.mode === "erase" ? "destination-out" : "source-over";
  context.strokeStyle = stroke.mode === "erase" ? "rgba(0,0,0,1)" : stroke.color.slice(0, 7);
  const colorAlpha = Number.parseInt(stroke.color.slice(7, 9) || "FF", 16) / 255;
  context.globalAlpha = stroke.opacity * colorAlpha;
  context.lineWidth = stroke.size * canvas.width / 256;
  context.lineCap = "round";
  context.lineJoin = "round";
  context.beginPath();
  context.moveTo(stroke.points[0]!.x * canvas.width / 256, stroke.points[0]!.y * canvas.height / 256);
  for (const point of stroke.points.slice(1)) {
    context.lineTo(point.x * canvas.width / 256, point.y * canvas.height / 256);
  }
  context.stroke();
  context.restore();
  if (stroke.mode === "paint" && stroke.clipToMask) {
    context.save();
    context.globalCompositeOperation = "destination-in";
    context.drawImage(image, 0, 0, canvas.width, canvas.height);
    context.restore();
  }
}

function toHex(value: number): string {
  return value.toString(16).padStart(2, "0").toUpperCase();
}