import { create } from 'zustand';
import type { Request, ResponseSnapshot, KeyValue, Body, Auth, RequestStatus } from '@/types';
import { makeEmptyRequest } from '@/types';
import { tauri } from '@/lib/tauri';
import { nanoid } from '@/lib/nanoid';

interface RequestState {
  request: Request;
  response: ResponseSnapshot | null;
  loading: boolean;
  error: string | null;
  clientId: string | null;
  setMethod: (m: string) => void;
  setUrl: (u: string) => void;
  setName: (n: string) => void;
  setBody: (b: Body) => void;
  setAuth: (a: Auth) => void;
  setParams: (p: KeyValue[]) => void;
  setHeaders: (h: KeyValue[]) => void;
  setStatus: (s: RequestStatus) => void;
  loadRequest: (r: Request) => void;
  resetRequest: () => void;
  setResponse: (r: ResponseSnapshot | null) => void;
  setLoading: (l: boolean) => void;
  setError: (e: string | null) => void;
  clearError: () => void;
  send: () => Promise<void>;
  cancel: () => Promise<void>;
}

export const useRequestStore = create<RequestState>((set, get) => ({
  request: makeEmptyRequest(),
  response: null,
  loading: false,
  error: null,
  clientId: null,

  setMethod: (m) => set((s) => ({ request: { ...s.request, method: m } })),
  setUrl: (u) => set((s) => ({ request: { ...s.request, url: u } })),
  setName: (n) => set((s) => ({ request: { ...s.request, name: n } })),
  setBody: (b) => set((s) => ({ request: { ...s.request, body: b } })),
  setAuth: (a) => set((s) => ({ request: { ...s.request, auth: a } })),
  setParams: (p) => set((s) => ({ request: { ...s.request, params: p } })),
  setHeaders: (h) => set((s) => ({ request: { ...s.request, headers: h } })),
  setStatus: (st) => set((s) => ({ request: { ...s.request, status: st } })),
  loadRequest: (r) => set({ request: r, response: null, error: null }),
  resetRequest: () =>
    set({ request: makeEmptyRequest(), response: null, error: null, clientId: null }),
  setResponse: (r) => set({ response: r, loading: false }),
  setLoading: (l) => set({ loading: l }),
  setError: (e) => set({ error: e, loading: false }),
  clearError: () => set({ error: null }),

  send: async () => {
    const { request } = get();
    if (!request.url.trim()) return;
    const clientId = nanoid();
    set({ clientId, loading: true, error: null });
    try {
      const resp = await tauri.sendRequest(request, clientId);
      set({ response: resp, loading: false, clientId: null });
    } catch (e: any) {
      const msg = typeof e === 'string' ? e : e?.message || String(e);
      // 取消操作的错误不显示给用户
      if (msg.includes('请求已取消')) {
        set({ loading: false, clientId: null });
        return;
      }
      set({ error: msg, loading: false, clientId: null });
    }
  },

  cancel: async () => {
    const id = get().clientId;
    if (id) {
      await tauri.cancelRequest(id);
    }
  },
}));
