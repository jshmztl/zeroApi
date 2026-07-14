import * as React from "react";
import { create } from "zustand";
import { nanoid } from "@/lib/nanoid";
import { cn } from "@/lib/utils";

interface ToastItem {
  id: string;
  type: "success" | "error" | "info" | "warning";
  message: string;
  duration: number;
}

interface ToastState {
  toasts: ToastItem[];
  push: (t: Omit<ToastItem, "id" | "duration"> & { duration?: number }) => void;
  dismiss: (id: string) => void;
}

const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],
  push: (t) => {
    const id = nanoid();
    const item: ToastItem = { id, duration: 3000, ...t };
    set({ toasts: [...get().toasts, item] });
    setTimeout(() => get().dismiss(id), item.duration);
  },
  dismiss: (id) => set({ toasts: get().toasts.filter((t) => t.id !== id) }),
}));

export const toast = {
  success: (message: string) => useToastStore.getState().push({ type: "success", message }),
  error: (message: string) => useToastStore.getState().push({ type: "error", message }),
  info: (message: string) => useToastStore.getState().push({ type: "info", message }),
  warning: (message: string) => useToastStore.getState().push({ type: "warning", message }),
};

export function ToastProvider({ children }: { children: React.ReactNode }) {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  return (
    <>
      {children}
      <div className="fixed top-4 left-1/2 -translate-x-1/2 z-50 flex flex-col gap-2 pointer-events-none">
        {toasts.map((t) => {
          const styleMap: Record<string, string> = {
            success: "bg-emerald-50 border-emerald-200 text-emerald-800",
            error: "bg-red-50 border-red-200 text-red-800",
            info: "bg-blue-50 border-blue-200 text-blue-800",
            warning: "bg-amber-50 border-amber-200 text-amber-800",
          };
          const icon: Record<string, string> = {
            success: "✓",
            error: "✕",
            info: "ℹ",
            warning: "!",
          };
          return (
            <div
              key={t.id}
              onClick={() => dismiss(t.id)}
              className={cn(
                "pointer-events-auto px-4 py-2.5 rounded-lg border shadow-sm",
                "text-sm font-medium flex items-center gap-2 cursor-pointer animate-fade-in min-w-[200px] max-w-md",
                styleMap[t.type]
              )}
            >
              <span className="w-4 h-4 inline-flex items-center justify-center text-xs">
                {icon[t.type]}
              </span>
              {t.message}
            </div>
          );
        })}
      </div>
    </>
  );
}
