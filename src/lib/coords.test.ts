import { describe, expect, it } from "vitest";
import {
  clamp01,
  clampRect,
  isOffscreen,
  isValidRect,
  normalizedToPreview,
  previewToNormalized,
  rectsOverlap,
} from "@/lib/coords";

describe("coords", () => {
  it("clamps values to 0..1", () => {
    expect(clamp01(-2)).toBe(0);
    expect(clamp01(2)).toBe(1);
    expect(clamp01(0.4)).toBeCloseTo(0.4);
    expect(clamp01(Number.NaN)).toBe(0);
  });

  it("clampRect keeps rects inside the unit square", () => {
    const c = clampRect({ x: -0.5, y: 0.9, w: 2, h: 0.01 });
    expect(isValidRect(c)).toBe(true);
    expect(c.x).toBeGreaterThanOrEqual(0);
    expect(c.y).toBeGreaterThanOrEqual(0);
  });

  it("scaling round-trips", () => {
    const cases = [
      { x: 0, y: 0, w: 1, h: 1 },
      { x: 0.25, y: 0.1, w: 0.5, h: 0.6 },
      { x: 0.7, y: 0.7, w: 0.3, h: 0.3 },
    ];
    for (const r of cases) {
      const px = normalizedToPreview(r, 720, 405);
      const back = previewToNormalized(px, 720, 405);
      expect(back.x).toBeCloseTo(r.x, 5);
      expect(back.y).toBeCloseTo(r.y, 5);
      expect(back.w).toBeCloseTo(r.w, 5);
      expect(back.h).toBeCloseTo(r.h, 5);
    }
  });

  it("detects overlap and offscreen", () => {
    expect(rectsOverlap({ x: 0, y: 0, w: 0.5, h: 0.5 }, { x: 0.25, y: 0.25, w: 0.5, h: 0.5 })).toBe(true);
    expect(rectsOverlap({ x: 0, y: 0, w: 0.5, h: 0.5 }, { x: 0.5, y: 0.5, w: 0.5, h: 0.5 })).toBe(false);
    expect(isOffscreen({ x: 1.2, y: 0, w: 0.2, h: 0.2 })).toBe(true);
    expect(isOffscreen({ x: 0.1, y: 0.1, w: 0.5, h: 0.5 })).toBe(false);
  });
});
