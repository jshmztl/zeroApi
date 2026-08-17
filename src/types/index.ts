// 全局数据模型（V2，与 Rust 端 domain/ 对齐）

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

/** 请求 / 响应头条目（支持重复 Header，如 Set-Cookie） */
export interface HeaderEntry {
  name: string;
  value: string;
}

/** 键值对（query 参数 / form 字段） */
export interface KeyValue {
  name: string;
  value: string;
  enabled: boolean;
}

export type RequestBody =
  | { type: 'none' }
  | { type: 'form_data'; items: KeyValue[] }
  | { type: 'url_encoded'; items: KeyValue[] }
  | { type: 'raw'; content_type: string; content: string };

export type AuthConfig =
  | { type: 'none' }
  | { type: 'bearer'; token: string }
  | { type: 'basic'; username: string; password: string }
  | { type: 'api_key'; key: string; value: string; location: 'header' | 'query' };

/** 请求（V2：不再包含 last_response / status） */
export interface Request {
  id: string;
  collection_id: string;
  folder_id?: string | null;
  name: string;
  method: HttpMethod;
  url: string;
  headers: HeaderEntry[];
  query: KeyValue[];
  body?: RequestBody | null;
  auth?: AuthConfig | null;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export type NetworkErrorKind =
  | 'invalid_url'
  | 'dns'
  | 'connection'
  | 'timeout'
  | 'tls'
  | 'proxy'
  | 'protocol'
  | 'cancelled'
  | 'unknown';

export const NETWORK_ERROR_LABELS: Record<NetworkErrorKind, string> = {
  invalid_url: 'URL 无效',
  dns: 'DNS 解析失败',
  connection: '连接失败',
  timeout: '请求超时',
  tls: 'TLS 证书错误',
  proxy: '代理错误',
  protocol: '协议错误',
  cancelled: '请求已取消',
  unknown: '未知错误',
};

export interface NetworkError {
  kind: NetworkErrorKind;
  message: string;
  detail?: string;
}

export interface Timing {
  dns_ms?: number | null;
  tcp_ms?: number | null;
  tls_ms?: number | null;
  request_ms?: number | null;
  response_ms?: number | null;
  total_ms: number;
}

export type ResponseBody =
  | { type: 'text'; text: string }
  | { type: 'binary'; path: string; size: number };

export interface ResponseSnapshot {
  status: number;
  status_text: string;
  headers: HeaderEntry[];
  body: ResponseBody;
  size_bytes: number;
  content_type?: string | null;
  timing: Timing;
}

/** 一次请求执行记录（V2 历史） */
export interface RequestExecution {
  id: string;
  request_id: string;
  started_at: number;
  duration_ms: number;
  status_code?: number | null;
  success: boolean;
  error?: NetworkError | null;
  response?: ResponseSnapshot | null;
}

/** 历史列表项（带请求摘要） */
export interface ExecutionListItem {
  execution: RequestExecution;
  method: HttpMethod;
  url: string;
  name: string;
}

export interface Project {
  id: string;
  name: string;
  description?: string | null;
  root_path?: string | null;
  created_at: number;
  updated_at: number;
}

/** 集合（V2：挂载在 Project 下，无 request_ids 列表） */
export interface Collection {
  id: string;
  project_id: string;
  name: string;
  description?: string | null;
  sort_order: number;
  created_at: number;
  updated_at: number;
}

export interface Folder {
  id: string;
  collection_id: string;
  parent_id?: string | null;
  name: string;
  sort_order: number;
}

export type VariableKind = 'plain' | 'secret';

export interface EnvironmentVariable {
  name: string;
  value: string;
  kind: VariableKind;
  enabled: boolean;
}

export interface Environment {
  id: string;
  project_id?: string | null;
  name: string;
  base_url: string;
  vars: EnvironmentVariable[];
  active: boolean;
}

export interface Favorite {
  id: string;
  request: Request;
  created_at: number;
}

export interface Settings {
  timeout_ms: number;
  proxy_url: string;
  theme: 'light' | 'dark' | 'system';
  auto_save_history: boolean;
  history_limit: number;
  verify_ssl: boolean;
  follow_redirects: boolean;
  /** 文本响应预览上限（字节），默认 5MB */
  max_preview_size: number;
}

export interface ExportPayload {
  version: number;
  projects: Project[];
  collections: Collection[];
  folders: Folder[];
  requests: Request[];
  environments: Environment[];
  favorites: Favorite[];
}

export interface ImportPayload {
  projects: Project[];
  collections: Collection[];
  folders: Folder[];
  requests: Request[];
  environments: Environment[];
  favorites: Favorite[];
}

export const HTTP_METHODS: HttpMethod[] = [
  'GET',
  'POST',
  'PUT',
  'PATCH',
  'DELETE',
  'HEAD',
  'OPTIONS',
];

export function makeEmptyRequest(): Request {
  return {
    id: '',
    collection_id: '',
    folder_id: null,
    name: '',
    method: 'GET',
    url: '',
    headers: [],
    query: [],
    body: null,
    auth: null,
    sort_order: 0,
    created_at: Date.now(),
    updated_at: Date.now(),
  };
}
