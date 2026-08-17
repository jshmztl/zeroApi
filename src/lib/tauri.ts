// Tauri IPC 封装（V2）
import { invoke } from "@tauri-apps/api/core";
import type {
  Request,
  ResponseSnapshot,
  ExecutionListItem,
  RequestExecution,
  Favorite,
  Collection,
  Folder,
  Project,
  Environment,
  Settings,
  ImportPayload,
  ExportPayload,
} from "@/types";

export const tauri = {
  // ---- Request ----
  sendRequest: (request: Request, clientId?: string) =>
    invoke<ResponseSnapshot>("send_request", { request, clientId }),
  cancelRequest: (clientId: string) =>
    invoke<boolean>("cancel_request", { clientId }),
  saveRequest: (request: Request) =>
    invoke<string>("save_request", { request }),
  deleteRequest: (id: string) =>
    invoke<void>("delete_request", { id }),
  listRequests: (collectionId?: string, folderId?: string) =>
    invoke<Request[]>("list_requests", { collectionId, folderId }),
  getRequest: (id: string) =>
    invoke<Request | null>("get_request", { id }),

  // ---- History (RequestExecution) ----
  listHistory: (limit = 100) =>
    invoke<ExecutionListItem[]>("list_history", { limit }),
  getExecution: (id: string) =>
    invoke<RequestExecution | null>("get_execution", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  deleteHistory: (id: string) => invoke<void>("delete_history", { id }),

  // ---- Favorites ----
  listFavorites: () => invoke<Favorite[]>("list_favorites"),
  addFavorite: (request: Request) =>
    invoke<string>("add_favorite", { request }),
  removeFavorite: (id: string) =>
    invoke<void>("remove_favorite", { id }),

  // ---- Project ----
  listProjects: () => invoke<Project[]>("list_projects"),
  getProject: (id: string) =>
    invoke<Project | null>("get_project", { id }),
  saveProject: (project: Project) =>
    invoke<string>("save_project", { project }),
  deleteProject: (id: string) =>
    invoke<void>("delete_project", { id }),
  /** 导出项目到目录（Git-friendly 文件格式） */
  exportProject: (projectId: string, dir: string) =>
    invoke<void>("export_project", { projectId, dir }),
  /** 从目录导入项目，返回新项目 id */
  importProject: (dir: string) =>
    invoke<string>("import_project", { dir }),

  // ---- Collection / Folder ----
  listCollections: (projectId?: string) =>
    invoke<Collection[]>("list_collections", { projectId }),
  getCollection: (id: string) =>
    invoke<Collection | null>("get_collection", { id }),
  saveCollection: (collection: Collection) =>
    invoke<string>("save_collection", { collection }),
  deleteCollection: (id: string) =>
    invoke<void>("delete_collection", { id }),
  listFolders: (collectionId?: string) =>
    invoke<Folder[]>("list_folders", { collectionId }),
  saveFolder: (folder: Folder) =>
    invoke<string>("save_folder", { folder }),
  deleteFolder: (id: string) =>
    invoke<void>("delete_folder", { id }),

  // ---- Environment ----
  listEnvironments: () => invoke<Environment[]>("list_environments"),
  getEnvironment: (id: string) =>
    invoke<Environment | null>("get_environment", { id }),
  saveEnvironment: (env: Environment) =>
    invoke<string>("save_environment", { env }),
  deleteEnvironment: (id: string) =>
    invoke<void>("delete_environment", { id }),

  // ---- Import / Export ----
  importCurl: (command: string) =>
    invoke<Request>("import_curl", { command }),
  importJson: (content: string) =>
    invoke<ImportPayload>("import_json", { args: { content } }),
  exportJson: () => invoke<string>("export_json"),
  /** 导入 OpenAPI 3.0/3.1（YAML/JSON）；返回导入结果 */
  importOpenapi: (content: string, projectId?: string) =>
    invoke<OpenApiImportResult>("import_openapi", { content, projectId }),
  /** 从 URL 抓取并导入 OpenAPI 文档 */
  importOpenapiUrl: (url: string, projectId?: string) =>
    invoke<OpenApiImportResult>("import_openapi_url", { url, projectId }),
  /** Request → cURL 命令 */
  exportCurl: (request: Request) =>
    invoke<string>("export_curl", { request }),

  // ---- Settings ----
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) =>
    invoke<void>("save_settings", { settings }),

  clearAllData: () => invoke<void>("clear_all_data"),
  appVersion: () => invoke<string>("app_version"),

  // ---- Cookie Session（按 Project 隔离） ----
  getSessionCookies: (projectId: string, url: string) =>
    invoke<string>("get_session_cookies", { projectId, url }),
  clearSessionCookies: (projectId: string) =>
    invoke<void>("clear_session_cookies", { projectId }),
};

export type { ExportPayload };

/** OpenAPI 导入结果 */
export interface OpenApiImportResult {
  project_id: string;
  project_name: string;
  collections: number;
  requests: number;
}
