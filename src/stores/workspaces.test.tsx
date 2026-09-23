import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Workspace } from "@/types";

const { mocks } = vi.hoisted(() => ({
  mocks: {
    getWorkspaces: vi.fn(),
    createWorkspace: vi.fn(),
    updateWorkspace: vi.fn(),
    deleteWorkspace: vi.fn(),
    duplicateWorkspace: vi.fn(),
    toggleFavorite: vi.fn(),
    createApp: vi.fn(),
    updateApp: vi.fn(),
    deleteApp: vi.fn(),
    reorderApps: vi.fn(),
  },
}));

vi.mock("@/services/tauri", () => ({
  getWorkspaces: mocks.getWorkspaces,
  createWorkspace: mocks.createWorkspace,
  updateWorkspace: mocks.updateWorkspace,
  deleteWorkspace: mocks.deleteWorkspace,
  duplicateWorkspace: mocks.duplicateWorkspace,
  toggleFavorite: mocks.toggleFavorite,
  createApp: mocks.createApp,
  updateApp: mocks.updateApp,
  deleteApp: mocks.deleteApp,
  reorderApps: mocks.reorderApps,
  getMonitors: vi.fn().mockResolvedValue([]),
  getSessions: vi.fn().mockResolvedValue([]),
}));

import { useWorkspaces } from "@/stores/useWorkspaces";

function ws(id: string, name: string): Workspace {
  return {
    id,
    name,
    description: "",
    icon: "",
    color: "#0078d4",
    hotkey: "",
    favorite: false,
    created_at: "2026-01-01T00:00:00.000Z",
    updated_at: "2026-01-01T00:00:00.000Z",
    last_launched_at: null,
    apps: [],
  };
}

describe("workspaces store CRUD", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useWorkspaces.setState({ workspaces: [], selectedId: null, loading: false, error: null });
  });

  it("fetches and selects the first workspace", async () => {
    mocks.getWorkspaces.mockResolvedValue([ws("a", "Alpha"), ws("b", "Beta")]);
    await useWorkspaces.getState().fetchWorkspaces();
    const s = useWorkspaces.getState();
    expect(s.workspaces).toHaveLength(2);
    expect(s.selectedId).toBe("a");
  });

  it("creates, renames, favorites and deletes", async () => {
    const created = ws("n1", "Fresh");
    mocks.createWorkspace.mockResolvedValue(created);
    await useWorkspaces.getState().createWorkspace("Fresh");
    expect(useWorkspaces.getState().workspaces[0]?.id).toBe("n1");

    mocks.updateWorkspace.mockResolvedValue({ ...created, name: "Renamed" });
    await useWorkspaces.getState().renameWorkspace("n1", "Renamed");
    expect(useWorkspaces.getState().workspaces[0]?.name).toBe("Renamed");

    mocks.toggleFavorite.mockResolvedValue({ ...created, name: "Renamed", favorite: true });
    await useWorkspaces.getState().toggleFavorite("n1");
    expect(useWorkspaces.getState().workspaces[0]?.favorite).toBe(true);

    mocks.deleteWorkspace.mockResolvedValue(undefined);
    await useWorkspaces.getState().deleteWorkspace("n1");
    expect(useWorkspaces.getState().workspaces).toHaveLength(0);
  });

  it("creates and updates apps inside a workspace", async () => {
    useWorkspaces.setState({ workspaces: [ws("w1", "Dev")], selectedId: "w1" });
    mocks.createApp.mockResolvedValue({
      id: "app-1",
      workspace_id: "w1",
      name: "Code",
      exe_path: "C:\\code.exe",
      args: "",
      cwd: "",
      url: "",
      delay_ms: 0,
      monitor_id: "",
      x: 0,
      y: 0,
      w: 0.5,
      h: 1,
      state: "normal",
      policy: "if_not_running",
      match_rules: {},
      sort_order: 0,
    });
    await useWorkspaces.getState().createApp("w1", {
      name: "Code",
      exe_path: "C:\\code.exe",
      args: "",
      cwd: "",
      url: "",
      delay_ms: 0,
      monitor_id: "",
      x: 0,
      y: 0,
      w: 0.5,
      h: 1,
      state: "normal",
      policy: "if_not_running",
      match_rules: {},
    });
    expect(useWorkspaces.getState().workspaces[0]?.apps).toHaveLength(1);

    mocks.updateApp.mockResolvedValue({
      id: "app-1",
      workspace_id: "w1",
      name: "Code Updated",
      exe_path: "C:\\code.exe",
      args: "",
      cwd: "",
      url: "",
      delay_ms: 0,
      monitor_id: "",
      x: 0.5,
      y: 0,
      w: 0.5,
      h: 1,
      state: "normal",
      policy: "if_not_running",
      match_rules: {},
      sort_order: 0,
    });
    await useWorkspaces.getState().updateApp("app-1", { name: "Code Updated", x: 0.5 });
    expect(useWorkspaces.getState().workspaces[0]?.apps[0]?.name).toBe("Code Updated");
  });
});
