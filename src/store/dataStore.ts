import { create } from "zustand";
import { tauri } from "@/lib/tauri";
import type {
  HistoryItem,
  Favorite,
  Collection,
  Environment,
} from "@/types";

interface DataState {
  history: HistoryItem[];
  favorites: Favorite[];
  collections: Collection[];
  environments: Environment[];
  activeEnvId: string | null;

  loadAll: () => Promise<void>;
  loadHistory: () => Promise<void>;
  loadFavorites: () => Promise<void>;
  loadCollections: () => Promise<void>;
  loadEnvironments: () => Promise<void>;

  clearHistory: () => Promise<void>;
  removeFavorite: (id: string) => Promise<void>;
  removeCollection: (id: string) => Promise<void>;
  removeEnvironment: (id: string) => Promise<void>;
  setActiveEnv: (id: string | null) => void;

  attachToCollection: (collectionId: string, requestId: string) => Promise<void>;
  detachFromCollection: (collectionId: string, requestId: string) => Promise<void>;
  updateCollection: (id: string, patch: Partial<Pick<Collection, "name" | "description" | "request_ids">>) => Promise<void>;
}

export const useDataStore = create<DataState>((set, get) => ({
  history: [],
  favorites: [],
  collections: [],
  environments: [],
  activeEnvId: null,

  loadAll: async () => {
    await Promise.all([
      get().loadHistory(),
      get().loadFavorites(),
      get().loadCollections(),
      get().loadEnvironments(),
    ]);
  },
  loadHistory: async () => {
    try {
      const h = await tauri.listHistory(100);
      set({ history: h });
    } catch (e) {
      console.error(e);
    }
  },
  loadFavorites: async () => {
    try {
      const f = await tauri.listFavorites();
      set({ favorites: f });
    } catch (e) {
      console.error(e);
    }
  },
  loadCollections: async () => {
    try {
      const c = await tauri.listCollections();
      set({ collections: c });
    } catch (e) {
      console.error(e);
    }
  },
  loadEnvironments: async () => {
    try {
      const e = await tauri.listEnvironments();
      const active = e.find((x) => x.active);
      set({ environments: e, activeEnvId: active?.id ?? null });
    } catch (err) {
      console.error(err);
    }
  },
  clearHistory: async () => {
    await tauri.clearHistory();
    set({ history: [] });
  },
  removeFavorite: async (id) => {
    await tauri.removeFavorite(id);
    set({ favorites: get().favorites.filter((f) => f.id !== id) });
  },
  removeCollection: async (id) => {
    await tauri.deleteCollection(id);
    set({ collections: get().collections.filter((c) => c.id !== id) });
  },
  removeEnvironment: async (id) => {
    await tauri.deleteEnvironment(id);
    set({
      environments: get().environments.filter((e) => e.id !== id),
      activeEnvId:
        get().activeEnvId === id ? null : get().activeEnvId,
    });
  },
  setActiveEnv: (id) => set({ activeEnvId: id }),

  attachToCollection: async (collectionId, requestId) => {
    const col = get().collections.find((c) => c.id === collectionId);
    if (!col) return;
    const request_ids = col.request_ids.includes(requestId)
      ? col.request_ids
      : [...col.request_ids, requestId];
    await tauri.updateCollection(collectionId, { request_ids });
    await get().loadCollections();
  },

  detachFromCollection: async (collectionId, requestId) => {
    const col = get().collections.find((c) => c.id === collectionId);
    if (!col) return;
    const request_ids = col.request_ids.filter((id) => id !== requestId);
    await tauri.updateCollection(collectionId, { request_ids });
    await get().loadCollections();
  },

  updateCollection: async (id, patch) => {
    await tauri.updateCollection(id, patch);
    await get().loadCollections();
  },
}));
