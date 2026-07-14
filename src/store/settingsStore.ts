import { create } from "zustand";
import { tauri } from "@/lib/tauri";
import type { Settings } from "@/types";

interface SettingsState {
  settings: Settings;
  loaded: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => Promise<void>;
}

const DEFAULT_SETTINGS: Settings = {
  timeout_ms: 30000,
  proxy_url: "",
  theme: "light",
  auto_save_history: true,
  history_limit: 100,
  verify_ssl: true,
  follow_redirects: true,
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,
  load: async () => {
    try {
      const s = await tauri.getSettings();
      set({ settings: { ...DEFAULT_SETTINGS, ...s }, loaded: true });
    } catch (e) {
      console.error("加载设置失败", e);
      set({ loaded: true });
    }
  },
  update: async (patch) => {
    const next = { ...get().settings, ...patch };
    set({ settings: next });
    try {
      await tauri.saveSettings(next);
    } catch (e) {
      console.error("保存设置失败", e);
    }
  },
}));
