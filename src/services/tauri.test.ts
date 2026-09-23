import { describe, expect, it, vi } from "vitest";
import { parseWorkspaceImport, serializeWorkspace, validateWorkspaceImport } from "@/services/tauri";
import type { Workspace } from "@/types";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockRejectedValue(new Error("no backend")),
}));

function sample(): Workspace {
  return {
    id: "ws-1",
    name: "Dev",
    description: "d",
    icon: "",
    color: "#0078d4",
    hotkey: "Ctrl+Alt+D",
    favorite: true,
    created_at: "2026-01-01T00:00:00.000Z",
    updated_at: "2026-01-02T00:00:00.000Z",
    last_launched_at: null,
    apps: [
      {
        id: "a1",
        workspace_id: "ws-1",
        name: "Code",
        exe_path: "C:\\code.exe",
        args: "",
        cwd: "",
        url: "",
        delay_ms: 0,
        monitor_id: "m1",
        x: 0,
        y: 0,
        w: 0.5,
        h: 1,
        state: "normal",
        policy: "if_not_running",
        match_rules: { title_contains: "Code" },
        sort_order: 0,
      },
    ],
  };
}

describe("tauri import/export validation", () => {
  it("serializes and re-validates a workspace", () => {
    const json = serializeWorkspace(sample());
    const data = JSON.parse(json) as unknown;
    expect(validateWorkspaceImport(data).ok).toBe(true);
    const parsed = parseWorkspaceImport(json);
    expect(parsed.name).toBe("Dev");
    expect(parsed.apps).toHaveLength(1);
  });

  it("rejects shapes with missing names and bad rects", () => {
    expect(validateWorkspaceImport(null).ok).toBe(false);
    expect(validateWorkspaceImport({ workspace: { name: "" } }).ok).toBe(false);
    expect(
      validateWorkspaceImport({ workspace: { name: "x", apps: [{ name: "a", x: 5 }] } }).ok,
    ).toBe(false);
    expect(() => parseWorkspaceImport("not json")).toThrow();
  });

  it("falls back gracefully when backend is missing", async () => {
    const { getWorkspaces, getMonitors, getSessions } = await import("@/services/tauri");
    await expect(getWorkspaces()).resolves.toEqual([]);
    await expect(getMonitors()).resolves.toEqual([]);
    await expect(getSessions()).resolves.toEqual([]);
  });

  it("forwards the import confirmation flag to the backend", async () => {
    const core = await import("@tauri-apps/api/core");
    const { importWorkspace } = await import("@/services/tauri");
    const json = serializeWorkspace(sample());
    await importWorkspace(json, true);
    expect(vi.mocked(core.invoke)).toHaveBeenCalledWith(
      "import_workspace",
      expect.objectContaining({ json, confirmed: true }),
    );
    await importWorkspace(json);
    expect(vi.mocked(core.invoke)).toHaveBeenCalledWith(
      "import_workspace",
      expect.objectContaining({ confirmed: false }),
    );
  });
});
