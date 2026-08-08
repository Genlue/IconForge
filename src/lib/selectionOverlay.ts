/**
 * Marching-ants overlay drawing shared by the preview and the source editor.
 * Draws a translucent red fill for the selection plus a rolling black/white
 * dashed boundary.
 */
export function drawSelectionOverlay(
  canvas: HTMLCanvasElement | null,
  maskImage: HTMLImageElement | null,
  phase: number,
): void {
  const context = canvas?.getContext("2d");
  if (!canvas || !context) {
    return;
  }
  context.clearRect(0, 0, canvas.width, canvas.height);
  if (!maskImage) return;

  const size = canvas.width;
  context.drawImage(maskImage, 0, 0, size, size);

  const pixels = context.getImageData(0, 0, size, size);
  const data = pixels.data;
  for (let i = 0; i < data.length; i += 4) {
    const value = data[i]!;
    if (value > 0) {
      data[i + 1] = 0;
      data[i + 2] = 0;
      data[i + 3] = Math.max(value, 60);
      data[i] = 255;
    }
  }
  context.putImageData(pixels, 0, 0);

  const edge = new Uint8ClampedArray(data.length);
  for (let y = 1; y < size - 1; y++) {
    for (let x = 1; x < size - 1; x++) {
      const idx = (y * size + x) * 4;
      const inside = (data[idx] ?? 0) >= 128;
      const neighborOffsets = [-4, 4, -size * 4, size * 4];
      let onEdge = false;
      for (const off of neighborOffsets) {
        const nInside = (data[idx + off] ?? 0) >= 128;
        if (inside !== nInside) {
          onEdge = true;
          break;
        }
      }
      if (!onEdge) continue;
      // Both the outer boundary and inner "holes" get the dashed border.
      if (inside) {
        const dashIndex = (x * 5 + y * 7 + Math.floor(phase / 2)) % 10;
        const color = dashIndex < 5 ? 0 : 255;
        const o = idx;
        edge[o] = color;
        edge[o + 1] = color;
        edge[o + 2] = color;
        edge[o + 3] = 255;
      } else {
        // Outline of a hole inside the selection: white dashes over the red
        // fill, so the inner rim stays visible.
        const dashIndex = (x * 5 + y * 7 + Math.floor(phase / 2)) % 10;
        const color = dashIndex < 5 ? 255 : 0;
        const o = idx;
        edge[o] = color;
        edge[o + 1] = color;
        edge[o + 2] = color;
        edge[o + 3] = 255;
      }
    }
  }
  context.putImageData(new ImageData(edge, size, size), 0, 0);
}

/** Loads a data URI into an HTMLImageElement and resolves on load. */
export function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((resolve, reject) => {
    const image = new Image();
    image.onload = () => resolve(image);
    image.onerror = () => reject(new Error("image decode failed"));
    image.src = src;
  });
}