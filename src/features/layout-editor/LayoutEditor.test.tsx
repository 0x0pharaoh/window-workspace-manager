import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockRejectedValue(new Error("no backend")),
}));

import { useWorkspaces } from "@/stores/useWorkspaces";
import { useMonitors } from "@/stores/useMonitors";
import { LayoutEditorPage } from "@/features/layout-editor/LayoutEditorPage";

function seed(): void {
  useWorkspaces.setState({
    workspaces: [
      {
        id: "ws-1",
        name: "Dev",
        description: "",
        icon: "",
        color: "#0078d4",
        hotkey: "",
        favorite: false,
        created_at: "2026-01-01T00:00:00.000Z",
        updated_at: "2026-01-02T00:00:00.000Z",
        last_launched_at: null,
        apps: [
          {
            id: "app-1",
            workspace_id: "ws-1",
            name: "Editor",
            exe_path: "C:\\editor.exe",
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
            match_rules: {},
            sort_order: 0,
          },
          {
            id: "app-2",
            workspace_id: "ws-1",
            name: "Browser",
            exe_path: "C:\\browser.exe",
            args: "",
            cwd: "",
            url: "",
            delay_ms: 0,
            monitor_id: "m1",
            x: 0.5,
            y: 0,
            w: 0.5,
            h: 1,
            state: "normal",
            policy: "if_not_running",
            match_rules: {},
            sort_order: 1,
          },
        ],
      },
    ],
    selectedId: "ws-1",
    loading: false,
    error: null,
  });
  useMonitors.setState({
    monitors: [
      {
        id: "m1",
        name: "Display 1",
        x: 0,
        y: 0,
        width: 1920,
        height: 1080,
        work_x: 0,
        work_y: 0,
        work_width: 1920,
        work_height: 1040,
        scale_factor: 1,
        is_primary: true,
      },
    ],
    loading: false,
    error: null,
  });
}

describe("LayoutEditor", () => {
  beforeEach(() => {
    seed();
  });

  it("renders draggable cards for each app", () => {
    render(<LayoutEditorPage />);
    expect(screen.getByTestId("layout-preview")).toBeInTheDocument();
    expect(screen.getByTestId("layout-card-app-1")).toBeInTheDocument();
    expect(screen.getByTestId("layout-card-app-2")).toBeInTheDocument();
    expect(screen.getAllByText("Editor").length).toBeGreaterThan(0);
  });

  it("applies a preset and updates coordinates", async () => {
    const user = userEvent.setup();
    render(<LayoutEditorPage />);
    await user.click(screen.getByRole("button", { name: /three columns/i }));
    const apps = useWorkspaces.getState().workspaces[0]?.apps ?? [];
    expect(apps).toHaveLength(2);
    for (const a of apps) {
      expect(a.x).toBeGreaterThanOrEqual(0);
      expect(a.w).toBeGreaterThan(0);
      expect(a.x + a.w).toBeLessThanOrEqual(1.0001);
    }
  });

  it("edits coordinates via manual percent inputs (resize path)", async () => {
    render(<LayoutEditorPage />);
    const input = screen.getByRole("spinbutton", { name: "Editor w percent" });
    fireEvent.change(input, { target: { value: "70" } });
    await waitFor(() => {
      const app = useWorkspaces.getState().workspaces[0]?.apps.find((a) => a.id === "app-1");
      expect(app?.w).toBeCloseTo(0.7, 2);
    });
  });
});
