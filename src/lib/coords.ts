export interface NormRect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface PxRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export function clamp01(v: number): number {
  if (Number.isNaN(v)) return 0;
  if (v < 0) return 0;
  if (v > 1) return 1;
  return v;
}

export function clampRect(r: NormRect): NormRect {
  const w = Math.min(1, Math.max(0.05, r.w));
  const h = Math.min(1, Math.max(0.05, r.h));
  const x = Math.min(1 - w, Math.max(0, r.x));
  const y = Math.min(1 - h, Math.max(0, r.y));
  return { x, y, w, h };
}

export function isValidRect(r: NormRect): boolean {
  return (
    [r.x, r.y, r.w, r.h].every((v) => typeof v === "number" && !Number.isNaN(v)) &&
    r.x >= 0 &&
    r.y >= 0 &&
    r.w > 0 &&
    r.h > 0 &&
    r.x + r.w <= 1 + 1e-9 &&
    r.y + r.h <= 1 + 1e-9
  );
}

export function normalizedToPreview(r: NormRect, previewW: number, previewH: number): PxRect {
  const c = clampRect({ x: r.x, y: r.y, w: r.w, h: r.h });
  return {
    x: c.x * previewW,
    y: c.y * previewH,
    width: c.w * previewW,
    height: c.h * previewH,
  };
}

export function previewToNormalized(p: PxRect, previewW: number, previewH: number): NormRect {
  if (previewW <= 0 || previewH <= 0) return { x: 0, y: 0, w: 0.5, h: 0.5 };
  return clampRect({
    x: p.x / previewW,
    y: p.y / previewH,
    w: p.width / previewW,
    h: p.height / previewH,
  });
}

export function nativeToNormalized(
  x: number,
  y: number,
  w: number,
  h: number,
  workX: number,
  workY: number,
  workW: number,
  workH: number,
): NormRect {
  if (!(workW > 0) || !(workH > 0)) return { x: 0.05, y: 0.05, w: 0.6, h: 0.6 };
  return clampRect({
    x: (x - workX) / workW,
    y: (y - workY) / workH,
    w: w / workW,
    h: h / workH,
  });
}

export function rectsOverlap(a: NormRect, b: NormRect): boolean {
  return a.x < b.x + b.w && a.x + a.w > b.x && a.y < b.y + b.h && a.y + a.h > b.y;
}

export function isOffscreen(r: NormRect): boolean {
  return r.x >= 1 || r.y >= 1 || r.x + r.w <= 0 || r.y + r.h <= 0;
}

export function snapValue(v: number, step = 0.05): number {
  if (step <= 0) return v;
  return Math.round(v / step) * step;
}

export function snapRect(r: NormRect, step = 0.05): NormRect {
  return clampRect({
    x: snapValue(r.x, step),
    y: snapValue(r.y, step),
    w: snapValue(r.w, step),
    h: snapValue(r.h, step),
  });
}

export function previewSizeForMonitor(
  monitorW: number,
  monitorH: number,
  maxW = 720,
): { width: number; height: number } {
  if (monitorW <= 0 || monitorH <= 0) return { width: maxW, height: 405 };
  const h = Math.round((maxW * monitorH) / monitorW);
  return { width: maxW, height: Math.max(200, h) };
}
