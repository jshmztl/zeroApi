import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import type {
  ExecutionListItem,
  Favorite,
  Collection,
  Folder,
  Environment,
  Request,
  Project,
} from '@/types';

interface DataState {
  history: ExecutionListItem[];
  favorites: Favorite[];
  collections: Collection[];
  folders: Folder[];
  requests: Request[];
  environments: Environment[];
  projects: Project[];
  activeEnvId: string | null;

  loadAll: () => Promise<void>;
  loadHistory: () => Promise<void>;
  loadFavorites: () => Promise<void>;
  loadCollections: () => Promise<void>;
  loadFolders: (collectionId?: string) => Promise<void>;
  loadRequests: (collectionId?: string) => Promise<void>;
  loadEnvironments: () => Promise<void>;
  loadProjects: () => Promise<void>;

  clearHistory: () => Promise<void>;
  removeHistory: (id: string) => Promise<void>;
  removeFavorite: (id: string) => Promise<void>;
  removeCollection: (id: string) => Promise<void>;
  removeEnvironment: (id: string) => Promise<void>;
  setActiveEnv: (id: string | null) => void;
}

export const useDataStore = create<DataState>((set, get) => ({
  history: [],
  favorites: [],
  collections: [],
  folders: [],
  requests: [],
  environments: [],
  projects: [],
  activeEnvId: null,

  loadAll: async () => {
    await Promise.all([
      get().loadHistory(),
      get().loadFavorites(),
      get().loadCollections(),
      get().loadFolders(),
      get().loadRequests(),
      get().loadEnvironments(),
    ]);
  },
  loadHistory: async () => {
    try {
      const h = await tauri.listHistory(100);
      set({ history: h });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载历史记录失败: ${msg}`);
    }
  },
  loadFavorites: async () => {
    try {
      const f = await tauri.listFavorites();
      set({ favorites: f });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载收藏夹失败: ${msg}`);
    }
  },
  loadCollections: async () => {
    try {
      const c = await tauri.listCollections();
      set({ collections: c });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载集合失败: ${msg}`);
    }
  },
  loadFolders: async (collectionId?: string) => {
    try {
      const f = await tauri.listFolders(collectionId);
      set({ folders: f });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载文件夹失败: ${msg}`);
    }
  },
  loadRequests: async (collectionId?: string) => {
    try {
      const r = await tauri.listRequests(collectionId);
      set({ requests: r });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载请求失败: ${msg}`);
    }
  },
  loadEnvironments: async () => {
    try {
      const e = await tauri.listEnvironments();
      const active = e.find((x) => x.active);
      set({ environments: e, activeEnvId: active?.id ?? null });
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      console.error(err);
      toast.error(`加载环境变量失败: ${msg}`);
    }
  },
  loadProjects: async () => {
    try {
      const p = await tauri.listProjects();
      set({ projects: p });
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error(e);
      toast.error(`加载项目失败: ${msg}`);
    }
  },
  clearHistory: async () => {
    await tauri.clearHistory();
    set({ history: [] });
  },
  removeHistory: async (id) => {
    await tauri.deleteHistory(id);
    set({ history: get().history.filter((h) => h.execution.id !== id) });
  },
  removeFavorite: async (id) => {
    await tauri.removeFavorite(id);
    set({ favorites: get().favorites.filter((f) => f.id !== id) });
  },
  removeCollection: async (id) => {
    await tauri.deleteCollection(id);
    set({
      collections: get().collections.filter((c) => c.id !== id),
      requests: get().requests.filter((r) => r.collection_id !== id),
    });
  },
  removeEnvironment: async (id) => {
    await tauri.deleteEnvironment(id);
    set({
      environments: get().environments.filter((e) => e.id !== id),
      activeEnvId: get().activeEnvId === id ? null : get().activeEnvId,
    });
  },
  setActiveEnv: (id) => set({ activeEnvId: id }),
}));
