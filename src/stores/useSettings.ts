import { create } from "zustand";
import type { UserSettings } from "@/types";
import { DEFAULT_SETTINGS, getSettings, setSetting } from "@/services/tauri";

interface SettingsState {
  settings: UserSettings | null;
  loading: boolean;
  load: () => Promise<void>;
  updateSetting: <K extends keyof UserSettings>(key: K, value: UserSettings[K]) => Promise<void>;
}

export const useSettings = create<SettingsState>()((set, get) => ({
  settings: null,
  loading: false,

  load: async () => {
    set({ loading: true });
    try {
      const settings = await getSettings();
      set({ settings: { ...DEFAULT_SETTINGS, ...settings }, loading: false });
    } catch {
      set({ settings: { ...DEFAULT_SETTINGS }, loading: false });
    }
  },

  updateSetting: async (key, value) => {
    const prev = get().settings;
    set({ settings: prev ? { ...prev, [key]: value } : { ...DEFAULT_SETTINGS, [key]: value } });
    try {
      await setSetting(key, value);
    } catch (err) {
      console.warn("[settings] persist failed:", err);
    }
  },
}));
