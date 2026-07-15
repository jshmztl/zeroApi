// Tauri IPC 封装
import { invoke } from "@tauri-apps/api/core";
import type {
  Request,
  ResponseSnapshot,
  HistoryItem,
  Favorite,
  Collection,
  Environment,
  Settings,
  ImportPayload,
  ExportPayload,
} from "@/types";

export const tauri = {
  sendRequest: (request: Request, clientId?: string) =>
    invoke<ResponseSnapshot>("send_request", { request, clientId }),
  cancelRequest: (clientId: string) =>
    invoke<boolean>("cancel_request", { clientId }),
  saveRequest: (request: Request) =>
    invoke<string>("save_request", { request }),
  deleteRequest: (id: string) =>
    invoke<void>("delete_request", { id }),

  listHistory: (limit = 100) =>
    invoke<HistoryItem[]>("list_history", { limit }),
  clearHistory: () => invoke<void>("clear_history"),
  deleteHistory: (id: string) => invoke<void>("delete_history", { id }),

  listFavorites: () => invoke<Favorite[]>("list_favorites"),
  addFavorite: (request: Request) =>
    invoke<string>("add_favorite", { request }),
  removeFavorite: (id: string) =>
    invoke<void>("remove_favorite", { id }),

  listCollections: () => invoke<Collection[]>("list_collections"),
  saveCollection: (collection: Collection) =>
    invoke<string>("save_collection", { collection }),
  updateCollection: (id: string, patch: { name?: string; description?: string; request_ids?: string[] }) =>
    invoke<void>("update_collection", { id, patch }),
  deleteCollection: (id: string) =>
    invoke<void>("delete_collection", { id }),
  saveSavedRequest: (request: Request, collectionId?: string) =>
    invoke<string>("save_saved_request", { request, collectionId }),
  listSavedRequests: (collectionId?: string) =>
    invoke<Request[]>("list_saved_requests", { collectionId }),

  listEnvironments: () => invoke<Environment[]>("list_environments"),
  saveEnvironment: (env: Environment) =>
    invoke<string>("save_environment", { env }),
  deleteEnvironment: (id: string) =>
    invoke<void>("delete_environment", { id }),

  importCurl: (command: string) =>
    invoke<Request>("import_curl", { command }),

  importJson: (content: string) =>
    invoke<ImportPayload>("import_json", { args: { content } }),
  exportJson: (collections?: Collection[], environments?: Environment[]) =>
    invoke<string>("export_json", { collections, environments }),

  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) =>
    invoke<void>("save_settings", { settings }),

  clearAllData: () => invoke<void>("clear_all_data"),
  appVersion: () => invoke<string>("app_version"),
};

export type { ExportPayload };
