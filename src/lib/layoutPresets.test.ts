import { describe, expect, it } from "vitest";
import { PRESETS, applyPreset, rectsForPreset } from "@/lib/layoutPresets";
import { isValidRect } from "@/lib/coords";
import type { WorkspaceApp } from "@/types";

function makeApps(n: number): WorkspaceApp[] {
  return Array.from({ length: n }, (_, i) => ({
    id: `app-${i}`,
    workspace_id: "ws-1",
    name: `App ${i}`,
    exe_path: "C:\\app.exe",
    args: "",
    cwd: "",
    url: "",
    delay_ms: 0,
    monitor_id: "",
    x: 0,
    y: 0,
    w: 0.5,
    h: 0.5,
    state: "normal" as const,
    policy: "if_not_running" as const,
    match_rules: {},
    sort_order: i,
  }));
}

describe("layoutPresets", () => {
  for (const preset of PRESETS) {
    it(`preset "${preset}" assigns valid normalized rects for 1..6 apps`, () => {
      for (let n = 1; n <= 6; n += 1) {
        const rects = rectsForPreset(n, preset);
        expect(rects).toHaveLength(n);
        for (const r of rects) {
          expect(isValidRect(r), `${preset} n=${n} rect ${JSON.stringify(r)}`).toBe(true);
        }
        const out = applyPreset(makeApps(n), preset);
        expect(out).toHaveLength(n);
        for (const a of out) {
          expect(
            isValidRect({ x: a.x, y: a.y, w: a.w, h: a.h }),
            `${preset} app rect invalid`,
          ).toBe(true);
        }
      }
    });
  }

  it("custom preset preserves app identity without mutating input", () => {
    const apps = makeApps(3);
    const out = applyPreset(apps, "custom");
    expect(out.map((a) => a.id)).toEqual(["app-0", "app-1", "app-2"]);
    expect(out).not.toBe(apps);
  });

  it("two-col splits first two apps into halves", () => {
    const out = applyPreset(makeApps(2), "two-col");
    const a0 = out[0];
    const a1 = out[1];
    expect(a0).toBeDefined();
    expect(a1).toBeDefined();
    if (!a0 || !a1) return;
    expect(a0.x).toBeCloseTo(0);
    expect(a0.w).toBeCloseTo(0.5);
    expect(a1.x).toBeCloseTo(0.5);
    expect(a1.w).toBeCloseTo(0.5);
  });

  it("fullscreen covers the whole area", () => {
    const out = applyPreset(makeApps(2), "fullscreen");
    for (const a of out) {
      expect(a.x).toBeCloseTo(0);
      expect(a.y).toBeCloseTo(0);
      expect(a.w).toBeCloseTo(1);
      expect(a.h).toBeCloseTo(1);
    }
  });
});
