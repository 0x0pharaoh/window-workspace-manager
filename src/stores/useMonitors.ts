import { create } from "zustand";
import type { MonitorInfo } from "@/types";
import { getMonitors } from "@/services/tauri";

interface MonitorsState {
  monitors: MonitorInfo[];
  loading: boolean;
  error: string | null;
  fetchMonitors: () => Promise<void>;
}

export const useMonitors = create<MonitorsState>()((set) => ({
  monitors: [],
  loading: false,
  error: null,
  fetchMonitors: async () => {
    set({ loading: true, error: null });
    try {
      const monitors = await getMonitors();
      set({ monitors, loading: false });
    } catch (err) {
      set({ loading: false, error: err instanceof Error ? err.message : "Monitor fetch failed" });
    }
  },
}));
