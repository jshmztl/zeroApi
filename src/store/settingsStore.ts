import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import type { Settings } from '@/types';

let saveTimer: ReturnType<typeof setTimeout> | null = null;

interface SettingsState {
  settings: Settings;
  loaded: boolean;
  load: () => Promise<void>;
  update: (patch: Partial<Settings>) => void;
  flush: () => Promise<void>;
}

const DEFAULT_SETTINGS: Settings = {
  timeout_ms: 30000,
  proxy_url: '',
  theme: 'light',
  auto_save_history: true,
  history_limit: 100,
  verify_ssl: true,
  follow_redirects: true,
};

const persist = async (settings: Settings) => {
  try {
    await tauri.saveSettings(settings);
  } catch (e) {
    console.error('保存设置失败', e);
  }
};

export const useSettingsStore = create<SettingsState>((set, get) => ({
  settings: DEFAULT_SETTINGS,
  loaded: false,
  load: async () => {
    try {
      const s = await tauri.getSettings();
      set({ settings: { ...DEFAULT_SETTINGS, ...s }, loaded: true });
    } catch (e) {
      console.error('加载设置失败', e);
      set({ loaded: true });
    }
  },
  update: (patch) => {
    const next = { ...get().settings, ...patch };
    set({ settings: next });

    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      persist(next);
    }, 350);
  },
  flush: async () => {
    if (saveTimer) {
      clearTimeout(saveTimer);
      saveTimer = null;
      await persist(get().settings);
    }
  },
}));
