import { create } from "zustand";
import type { LaunchProgressItem, SessionInfo } from "@/types";
import * as api from "@/services/tauri";

interface SessionsState {
  sessions: SessionInfo[];
  progress: Record<string, LaunchProgressItem>;
  loading: boolean;
  launchWorkspace: (workspaceId: string) => Promise<string>;
  cancelLaunch: (workspaceId: string) => Promise<void>;
  closeSessionApp: (sessionId: string, appId: string) => Promise<void>;
  closeWorkspaceSession: (sessionId: string) => Promise<void>;
  fetchSessions: () => Promise<void>;
  setProgress: (item: LaunchProgressItem) => void;
  clearProgress: (workspaceId: string) => void;
}

export const useSessions = create<SessionsState>()((set) => ({
  sessions: [],
  progress: {},
  loading: false,

  fetchSessions: async () => {
    set({ loading: true });
    try {
      const sessions = await api.getSessions();
      set({ sessions, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  launchWorkspace: async (workspaceId) => {
    const sessionId = await api.launchWorkspace(workspaceId);
    await api.getSessions().then((sessions) => set({ sessions })).catch(() => undefined);
    return sessionId;
  },

  cancelLaunch: async (workspaceId) => {
    await api.cancelLaunch(workspaceId);
  },

  closeSessionApp: async (sessionId, appId) => {
    await api.closeSessionApp(sessionId, appId);
    set((s) => ({
      sessions: s.sessions.map((sess) =>
        sess.id === sessionId
          ? { ...sess, windows: sess.windows.filter((w) => w.app_id !== appId) }
          : sess,
      ),
    }));
  },

  closeWorkspaceSession: async (sessionId) => {
    await api.closeWorkspaceSession(sessionId);
    set((s) => ({ sessions: s.sessions.filter((sess) => sess.id !== sessionId) }));
  },

  setProgress: (item) =>
    set((s) => ({ progress: { ...s.progress, [item.app_id]: item } })),

  clearProgress: (workspaceId) =>
    set((s) => {
      const next = { ...s.progress };
      for (const k of Object.keys(next)) {
        if (next[k]?.workspace_id === workspaceId) delete next[k];
      }
      return { progress: next };
    }),
}));
