import { create } from "zustand";
import type { Workspace, WorkspaceApp, WorkspaceAppDraft } from "@/types";
import * as api from "@/services/tauri";

interface WorkspacesState {
  workspaces: Workspace[];
  selectedId: string | null;
  loading: boolean;
  error: string | null;
  fetchWorkspaces: () => Promise<void>;
  selectWorkspace: (id: string | null) => void;
  createWorkspace: (name: string) => Promise<Workspace>;
  renameWorkspace: (id: string, name: string) => Promise<void>;
  updateWorkspaceMeta: (id: string, patch: Partial<Workspace>) => Promise<void>;
  duplicateWorkspace: (id: string) => Promise<void>;
  deleteWorkspace: (id: string) => Promise<void>;
  toggleFavorite: (id: string) => Promise<void>;
  createApp: (workspaceId: string, input: WorkspaceAppDraft) => Promise<void>;
  updateApp: (appId: string, patch: Partial<WorkspaceApp>) => Promise<void>;
  deleteApp: (appId: string) => Promise<void>;
  reorderApps: (workspaceId: string, orderedIds: string[]) => Promise<void>;
  importWorkspace: (json: string, confirmed?: boolean) => Promise<void>;
  refreshWorkspace: (id: string) => Promise<void>;
}

function withApps(ws: Workspace, apps: WorkspaceApp[]): Workspace {
  return { ...ws, apps: [...apps].sort((a, b) => a.sort_order - b.sort_order), updated_at: new Date().toISOString() };
}

export const useWorkspaces = create<WorkspacesState>()((set, get) => ({
  workspaces: [],
  selectedId: null,
  loading: false,
  error: null,

  fetchWorkspaces: async () => {
    set({ loading: true, error: null });
    try {
      const list = await api.getWorkspaces();
      const sorted = [...list].sort((a, b) => b.updated_at.localeCompare(a.updated_at));
      const sel = get().selectedId;
      const stillThere = sel != null && sorted.some((w) => w.id === sel);
      set({
        workspaces: sorted,
        selectedId: stillThere ? sel : (sorted[0]?.id ?? null),
        loading: false,
      });
    } catch (err) {
      set({ loading: false, error: err instanceof Error ? err.message : "Failed to load workspaces" });
    }
  },

  selectWorkspace: (id) => set({ selectedId: id }),

  createWorkspace: async (name) => {
    const ws = await api.createWorkspace(name.trim() || "Untitled workspace");
    set((s) => ({ workspaces: [ws, ...s.workspaces], selectedId: ws.id }));
    return ws;
  },

  renameWorkspace: async (id, name) => {
    const clean = name.trim() || "Untitled workspace";
    set((s) => ({
      workspaces: s.workspaces.map((w) => (w.id === id ? { ...w, name: clean } : w)),
    }));
    try {
      const updated = await api.updateWorkspace(id, { name: clean });
      set((s) => ({
        workspaces: s.workspaces.map((w) =>
          w.id === id ? { ...w, name: updated.name ?? clean, updated_at: updated.updated_at ?? w.updated_at } : w,
        ),
      }));
    } catch {
      // optimistic state already applied
    }
  },

  updateWorkspaceMeta: async (id, patch) => {
    const { apps: _apps, ...rest } = patch;
    void _apps;
    set((s) => ({
      workspaces: s.workspaces.map((w) => (w.id === id ? { ...w, ...rest } : w)),
    }));
    try {
      await api.updateWorkspace(id, patch);
    } catch {
      // optimistic state already applied
    }
  },

  duplicateWorkspace: async (id) => {
    const copy = await api.duplicateWorkspace(id);
    set((s) => ({ workspaces: [copy, ...s.workspaces], selectedId: copy.id }));
  },

  deleteWorkspace: async (id) => {
    await api.deleteWorkspace(id);
    set((s) => {
      const rest = s.workspaces.filter((w) => w.id !== id);
      return {
        workspaces: rest,
        selectedId: s.selectedId === id ? (rest[0]?.id ?? null) : s.selectedId,
      };
    });
  },

  toggleFavorite: async (id) => {
    const updated = await api.toggleFavorite(id);
    set((s) => ({
      workspaces: s.workspaces.map((w) =>
        w.id === id ? { ...w, favorite: updated.favorite } : w,
      ),
    }));
  },

  createApp: async (workspaceId, input) => {
    const app = await api.createApp(workspaceId, input);
    set((s) => ({
      workspaces: s.workspaces.map((w) =>
        w.id === workspaceId ? withApps(w, [...w.apps, { ...app, workspace_id: workspaceId }]) : w,
      ),
    }));
  },

  updateApp: async (appId, patch) => {
    set((s) => ({
      workspaces: s.workspaces.map((w) => {
        if (!w.apps.some((a) => a.id === appId)) return w;
        const apps = w.apps.map((a) => (a.id === appId ? { ...a, ...patch, id: appId } : a));
        return { ...w, apps };
      }),
    }));
    try {
      await api.updateApp(appId, patch);
    } catch {
      // optimistic state already applied
    }
  },

  deleteApp: async (appId) => {
    await api.deleteApp(appId);
    set((s) => ({
      workspaces: s.workspaces.map((w) => ({ ...w, apps: w.apps.filter((a) => a.id !== appId) })),
    }));
  },

  reorderApps: async (workspaceId, orderedIds) => {
    await api.reorderApps(workspaceId, orderedIds);
    set((s) => ({
      workspaces: s.workspaces.map((w) => {
        if (w.id !== workspaceId) return w;
        const order = new Map(orderedIds.map((id, i) => [id, i] as const));
        const apps = [...w.apps].sort(
          (a, b) => (order.get(a.id) ?? 9999) - (order.get(b.id) ?? 9999),
        ).map((a, i) => ({ ...a, sort_order: i }));
        return { ...w, apps };
      }),
    }));
  },

  importWorkspace: async (json, confirmed) => {
    const ws = await api.importWorkspace(json, confirmed ?? false);
    set((s) => ({ workspaces: [ws, ...s.workspaces], selectedId: ws.id }));
  },

  refreshWorkspace: async (id) => {
    const ws = await api.getWorkspace(id);
    if (ws) {
      set((s) => ({ workspaces: s.workspaces.map((w) => (w.id === id ? ws : w)) }));
    }
  },
}));
