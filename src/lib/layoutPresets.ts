import type { WorkspaceApp } from "@/types";
import { clampRect } from "@/lib/coords";

export type PresetKind =
  | "two-col"
  | "three-col"
  | "main-sidebar"
  | "main-bottom"
  | "quadrants"
  | "fullscreen"
  | "custom";

export const PRESET_LABELS: Record<PresetKind, string> = {
  "two-col": "Two columns",
  "three-col": "Three columns",
  "main-sidebar": "Main + sidebar",
  "main-bottom": "Main + bottom",
  quadrants: "Quadrants",
  fullscreen: "Fullscreen",
  custom: "Custom",
};

export const PRESETS: PresetKind[] = [
  "two-col",
  "three-col",
  "main-sidebar",
  "main-bottom",
  "quadrants",
  "fullscreen",
  "custom",
];

interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export function rectsForPreset(count: number, preset: PresetKind): Rect[] {
  if (count <= 0) return [];
  if (preset === "custom") {
    return Array.from({ length: count }, () => ({ x: 0, y: 0, w: 0.5, h: 0.5 }));
  }
  if (preset === "fullscreen") {
    return Array.from({ length: count }, () => ({ x: 0, y: 0, w: 1, h: 1 }));
  }
  if (preset === "two-col") {
    const rows = Math.ceil(count / 2);
    const h = 1 / rows;
    return Array.from({ length: count }, (_, i) => ({
      x: (i % 2) * 0.5,
      y: Math.floor(i / 2) * h,
      w: 0.5,
      h,
    }));
  }
  if (preset === "three-col") {
    const rows = Math.ceil(count / 3);
    const w = 1 / 3;
    const h = 1 / rows;
    return Array.from({ length: count }, (_, i) => ({
      x: (i % 3) * w,
      y: Math.floor(i / 3) * h,
      w,
      h,
    }));
  }
  if (preset === "main-sidebar") {
    if (count === 1) return [{ x: 0, y: 0, w: 1, h: 1 }];
    const rest = count - 1;
    const out: Rect[] = [{ x: 0, y: 0, w: 0.7, h: 1 }];
    for (let i = 0; i < rest; i += 1) {
      out.push({ x: 0.7, y: i * (1 / rest), w: 0.3, h: 1 / rest });
    }
    return out;
  }
  if (preset === "main-bottom") {
    if (count === 1) return [{ x: 0, y: 0, w: 1, h: 1 }];
    const rest = count - 1;
    const out: Rect[] = [{ x: 0, y: 0, w: 1, h: 0.7 }];
    for (let i = 0; i < rest; i += 1) {
      out.push({ x: i * (1 / rest), y: 0.7, w: 1 / rest, h: 0.3 });
    }
    return out;
  }
  // quadrants
  const quad: Rect[] = [
    { x: 0, y: 0, w: 0.5, h: 0.5 },
    { x: 0.5, y: 0, w: 0.5, h: 0.5 },
    { x: 0, y: 0.5, w: 0.5, h: 0.5 },
    { x: 0.5, y: 0.5, w: 0.5, h: 0.5 },
  ];
  if (count <= 4) {
    if (count === 1) return [{ x: 0, y: 0, w: 1, h: 1 }];
    if (count === 2) {
      const left: Rect = { x: 0, y: 0, w: 0.5, h: 1 };
      const right: Rect = { x: 0.5, y: 0, w: 0.5, h: 1 };
      return [left, right];
    }
    if (count === 3) {
      return [
        { x: 0, y: 0, w: 0.5, h: 1 },
        { x: 0.5, y: 0, w: 0.5, h: 0.5 },
        { x: 0.5, y: 0.5, w: 0.5, h: 0.5 },
      ];
    }
    return quad.slice(0, 4);
  }
  const cols = Math.ceil(Math.sqrt(count));
  const rows = Math.ceil(count / cols);
  return Array.from({ length: count }, (_, i) => ({
    x: (i % cols) * (1 / cols),
    y: Math.floor(i / cols) * (1 / rows),
    w: 1 / cols,
    h: 1 / rows,
  }));
}

export function applyPreset(apps: WorkspaceApp[], preset: PresetKind): WorkspaceApp[] {
  if (preset === "custom") return apps.map((a) => ({ ...a }));
  const rects = rectsForPreset(apps.length, preset);
  return apps.map((app, i) => {
    const r = rects[i] ?? { x: 0, y: 0, w: 0.5, h: 0.5 };
    const c = clampRect(r);
    return { ...app, x: c.x, y: c.y, w: c.w, h: c.h };
  });
}
