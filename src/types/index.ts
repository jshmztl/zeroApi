// 全局数据模型(与 Rust 端 models.rs 对齐)

export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE' | 'HEAD' | 'OPTIONS';

export interface KeyValue {
  key: string;
  value: string;
  enabled: boolean;
  description?: string;
}

export type Body =
  | { type: 'none' }
  | { type: 'form_data'; items: KeyValue[] }
  | { type: 'url_encoded'; items: KeyValue[] }
  | { type: 'raw'; content_type: string; content: string };

export type Auth =
  | { type: 'none' }
  | { type: 'bearer'; token: string }
  | { type: 'basic'; username: string; password: string }
  | {
      type: 'api_key';
      key: string;
      value: string;
      location: 'header' | 'query';
    };

export type RequestStatus = 'in_development' | 'production' | 'deprecated';

export const REQUEST_STATUS_META: Record<RequestStatus, { label: string; color: string }> = {
  in_development: {
    label: '开发中',
    color: 'bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-400',
  },
  deprecated: {
    label: '已废弃',
    color: 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-400',
  },
  production: {
    label: '已上线',
    color: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400',
  },
};

export interface Request {
  id: string;
  name: string;
  method: string;
  url: string;
  params: KeyValue[];
  headers: KeyValue[];
  body: Body;
  auth: Auth;
  collection_id?: string | null;
  status: RequestStatus;
  last_response?: ResponseSnapshot | null;
}

export interface ResponseSnapshot {
  status: number;
  status_text: string;
  headers: Record<string, string>;
  body: string;
  time_ms: number;
  size_bytes: number;
  content_type: string;
}

export interface HistoryItem {
  id: string;
  request: Request;
  response: ResponseSnapshot;
  created_at: number;
}

export interface Favorite {
  id: string;
  request: Request;
  created_at: number;
}

export interface Collection {
  id: string;
  name: string;
  description: string;
  request_ids: string[];
  created_at: number;
  updated_at: number;
}

export interface Environment {
  id: string;
  name: string;
  base_url: string;
  vars: KeyValue[];
  active: boolean;
}

export interface Settings {
  timeout_ms: number;
  proxy_url: string;
  theme: 'light' | 'dark' | 'system';
  auto_save_history: boolean;
  history_limit: number;
  verify_ssl: boolean;
  follow_redirects: boolean;
}

export interface ExportPayload {
  version: number;
  requests: Request[];
  collections: Collection[];
  environments: Environment[];
  favorites: Favorite[];
}

export interface ImportPayload {
  requests: Request[];
  collections: Collection[];
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
    name: '',
    method: 'GET',
    url: '',
    params: [],
    headers: [],
    body: { type: 'none' },
    auth: { type: 'none' },
    collection_id: null,
    status: 'in_development',
  };
}
