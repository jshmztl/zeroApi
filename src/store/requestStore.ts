import { create } from 'zustand';
import type { Request, ResponseSnapshot, KeyValue, Body, Auth } from '@/types';
import { makeEmptyRequest } from '@/types';

interface RequestState {
  request: Request;
  response: ResponseSnapshot | null;
  loading: boolean;
  error: string | null;
  setMethod: (m: string) => void;
  setUrl: (u: string) => void;
  setName: (n: string) => void;
  setBody: (b: Body) => void;
  setAuth: (a: Auth) => void;
  setParams: (p: KeyValue[]) => void;
  setHeaders: (h: KeyValue[]) => void;
  loadRequest: (r: Request) => void;
  resetRequest: () => void;
  setResponse: (r: ResponseSnapshot | null) => void;
  setLoading: (l: boolean) => void;
  setError: (e: string | null) => void;
  clearError: () => void;
}

export const useRequestStore = create<RequestState>((set) => ({
  request: makeEmptyRequest(),
  response: null,
  loading: false,
  error: null,

  setMethod: (m) => set((s) => ({ request: { ...s.request, method: m } })),
  setUrl: (u) => set((s) => ({ request: { ...s.request, url: u } })),
  setName: (n) => set((s) => ({ request: { ...s.request, name: n } })),
  setBody: (b) => set((s) => ({ request: { ...s.request, body: b } })),
  setAuth: (a) => set((s) => ({ request: { ...s.request, auth: a } })),
  setParams: (p) => set((s) => ({ request: { ...s.request, params: p } })),
  setHeaders: (h) => set((s) => ({ request: { ...s.request, headers: h } })),
  loadRequest: (r) => set({ request: r, response: null, error: null }),
  resetRequest: () => set({ request: makeEmptyRequest(), response: null, error: null }),
  setResponse: (r) => set({ response: r, loading: false }),
  setLoading: (l) => set({ loading: l }),
  setError: (e) => set({ error: e, loading: false }),
  clearError: () => set({ error: null }),
}));
