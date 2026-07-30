import { useCallback, useEffect, useRef, useState } from "react";
import { useIconForgeStore } from "../../store/useIconForgeStore";
import type { BrushPoint, BrushStroke } from "../../types/domain";
import { Checkerboard } from "./Checkerboard";
import { EmptyDropZone } from "./EmptyDropZone";
import { Spinner } from "../common/Spinner";
import { useDebouncedPreview } from "../../hooks/useDebouncedPreview";
import { DEFAULT_RENDER_CONFIG, DEFAULT_UPSCALE_CONFIG } from "../../constants/defaults";

export function PreviewPane(): JSX.Element {
  const items = useIconForgeStore((s) => s.items);
  const selectedId = useIconForgeStore((s) => s.selectedItemId);
  const itemConfig = useIconForgeStore((s) =>
    s.selectedItemId ? s.itemConfigs[s.selectedItemId] : undefined,
  );
  const previewTool = useIconForgeStore((s) => s.previewTool);
  const addBrushStroke = useIconForgeStore((s) => s.addBrushStroke);
  const applyPickedColor = useIconForgeStore((s) => s.applyPickedColor);
  const dragActive = useIconForgeStore((s) => s.dragActive);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const imageRef = useRef<HTMLImageElement | null>(null);
  const [drawingPoints, setDrawingPoints] = useState<BrushPoint[]>([]);

  const selectedItem = items.find((i) => i.id === selectedId) ?? items[0] ?? null;
  const renderConfig = itemConfig?.renderConfig ?? DEFAULT_RENDER_CONFIG;
  const upscaleConfig = itemConfig?.upscaleConfig ?? DEFAULT_UPSCALE_CONFIG;
  const brushStrokes = itemConfig?.brushStrokes ?? [];
  const preview = useDebouncedPreview(
    selectedItem?.sourcePath ?? null,
    renderConfig,
    upscaleConfig,
    brushStrokes,
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
  }, [applyPickedColor, itemConfig, pointFromEvent, previewTool]);

  const handlePointerMove = useCallback((event: React.PointerEvent<HTMLCanvasElement>) => {
    if ((previewTool !== "brush" && previewTool !== "eraser") || drawingPoints.length === 0 || !imageRef.current || !itemConfig) return;
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
      : "default";

  return (
    <div className="relative flex h-full flex-col items-center justify-center gap-3 p-8">
      <div
        className="glass-panel relative flex aspect-square items-center justify-center overflow-hidden"
        style={{ width: 420, maxWidth: "100%", maxHeight: "100%" }}
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
        <div className="flex max-w-full flex-wrap items-center justify-center gap-x-3 gap-y-1 text-[11px] text-[var(--text-secondary)]">
          <span>导入源图 {selectedItem.sourceWidth}×{selectedItem.sourceHeight}</span>
          {preview.processedSourceWidth && preview.processedSourceHeight && (
            <span>{upscaleConfig.enabled ? "增强源图" : "处理源图"} {preview.processedSourceWidth}×{preview.processedSourceHeight}</span>
          )}
          <span>主图 256×256</span>
          {previewTool === "eyedropper" && <span className="text-[var(--accent)]">点击预览取色</span>}
          {previewTool === "brush" && <span className="text-[var(--accent)]">在预览上拖动绘制</span>}
          {previewTool === "eraser" && <span className="text-[var(--accent)]">在预览上拖动擦除</span>}
        </div>
      )}
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
