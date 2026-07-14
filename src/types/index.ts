// 全局数据模型(与 Rust 端 models.rs 对齐)

export type HttpMethod =
  | "GET"
  | "POST"
  | "PUT"
  | "PATCH"
  | "DELETE"
  | "HEAD"
  | "OPTIONS";

export interface KeyValue {
  key: string;
  value: string;
  enabled: boolean;
}

export type Body =
  | { type: "none" }
  | { type: "form_data"; items: KeyValue[] }
  | { type: "url_encoded"; items: KeyValue[] }
  | { type: "raw"; content_type: string; content: string };

export type Auth =
  | { type: "none" }
  | { type: "bearer"; token: string }
  | { type: "basic"; username: string; password: string }
  | {
      type: "api_key";
      key: string;
      value: string;
      location: "header" | "query";
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
  vars: KeyValue[];
  active: boolean;
}

export interface Settings {
  timeout_ms: number;
  proxy_url: string;
  theme: "light" | "dark" | "system";
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
  "GET",
  "POST",
  "PUT",
  "PATCH",
  "DELETE",
  "HEAD",
  "OPTIONS",
];

export function makeEmptyRequest(): Request {
  return {
    id: "",
    name: "",
    method: "GET",
    url: "",
    params: [],
    headers: [],
    body: { type: "none" },
    auth: { type: "none" },
    collection_id: null,
  };
}
