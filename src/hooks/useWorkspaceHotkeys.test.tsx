import { describe, expect, it, vi } from "vitest";
import { renderHook } from "@testing-library/react";
import { useWorkspaceHotkeys } from "@/hooks/useWorkspaceHotkeys";
import { useWorkspaces } from "@/stores/useWorkspaces";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockRejectedValue(new Error("no backend")),
}));

const register = vi.fn().mockResolvedValue(undefined);
const unregisterAll = vi.fn().mockResolvedValue(undefined);

vi.mock("@tauri-apps/plugin-global-shortcut", () => ({
  register,
  unregister: vi.fn().mockResolvedValue(undefined),
  unregisterAll,
  isRegistered: vi.fn().mockResolvedValue(false),
}));

function seed(): void {
  useWorkspaces.setState({
    workspaces: [
      {
        id: "ws-1",
        name: "Dev",
        description: "",
        icon: "",
        color: "#0078d4",
        hotkey: "Ctrl+Alt+D",
        favorite: false,
        created_at: "",
        updated_at: "",
        last_launched_at: null,
        apps: [],
      },
      {
        id: "ws-2",
        name: "No hotkey",
        description: "",
        icon: "",
        color: "#0078d4",
        hotkey: "",
        favorite: false,
        created_at: "",
        updated_at: "",
        last_launched_at: null,
        apps: [],
      },
    ],
    selectedId: null,
    loading: false,
    error: null,
  });
}

describe("useWorkspaceHotkeys", () => {
  it("registers workspace hotkeys and cleans up", async () => {
    seed();
    const { unmount } = renderHook(() => useWorkspaceHotkeys());
    await vi.waitFor(() => expect(register).toHaveBeenCalledWith("Ctrl+Alt+D", expect.any(Function)));
    expect(register).toHaveBeenCalledTimes(1);
    unmount();
    await vi.waitFor(() => expect(unregisterAll).toHaveBeenCalled());
  });
});
