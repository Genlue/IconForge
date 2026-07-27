export interface NormalizedPoint {
  x: number;
  y: number;
}

export function sampleSquirclePath(
  exponent: number,
  sampleCount: number = 256,
): NormalizedPoint[] {
  const points: NormalizedPoint[] = [];
  const n = exponent;

  for (let i = 0; i < sampleCount; i++) {
    const theta = (i / sampleCount) * 2 * Math.PI;
    const cosT = Math.cos(theta);
    const sinT = Math.sin(theta);
    const absCos = Math.abs(cosT);
    const absSin = Math.abs(sinT);
    const denom = Math.pow(
      Math.pow(absCos, n) + Math.pow(absSin, n),
      1 / n,
    );
    const r = denom === 0 ? 0 : 1 / denom;
    points.push({ x: r * cosT, y: r * sinT });
  }

  return points;
}

export function pointsToSvgPath(points: readonly NormalizedPoint[]): string {
  if (points.length === 0) return "";
  let d = `M ${points[0]?.x.toFixed(6)} ${points[0]?.y.toFixed(6)}`;
  for (let i = 1; i < points.length; i++) {
    const p = points[i];
    if (p) d += ` L ${p.x.toFixed(6)} ${p.y.toFixed(6)}`;
  }
  d += " Z";
  return d;
}
