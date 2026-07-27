export function parseHexRgba(
  value: string,
): { r: number; g: number; b: number; a: number } {
  const hex = value.replace("#", "");
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  const a = parseInt(hex.slice(6, 8), 16);
  return { r, g, b, a };
}

export function formatHexRgba(
  r: number,
  g: number,
  b: number,
  a: number,
): string {
  const toHex = (n: number) =>
    Math.round(Math.max(0, Math.min(255, n)))
      .toString(16)
      .padStart(2, "0");
  return `#${toHex(r)}${toHex(g)}${toHex(b)}${toHex(a)}`.toUpperCase();
}

export function replaceRgb(value: string, rgbHex: string): string {
  const cleanRgb = rgbHex.replace("#", "").slice(0, 6);
  const alpha = value.replace("#", "").slice(6, 8) || "FF";
  return `#${cleanRgb}${alpha}`.toUpperCase();
}

export function replaceAlpha(value: string, alpha: number): string {
  const base = value.replace("#", "").slice(0, 6);
  const a = Math.round(Math.max(0, Math.min(255, alpha)))
    .toString(16)
    .padStart(2, "0");
  return `#${base}${a}`.toUpperCase();
}
