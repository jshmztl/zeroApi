import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  return `${(ms / 1000).toFixed(2)} s`;
}

export function formatDate(timestamp: number): string {
  const d = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - d.getTime();
  const day = 24 * 60 * 60 * 1000;
  if (diff < day && d.getDate() === now.getDate()) {
    return d.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" });
  }
  if (diff < 2 * day) return "昨天";
  if (diff < 7 * day) {
    return d.toLocaleDateString("zh-CN", { weekday: "short" });
  }
  return d.toLocaleDateString("zh-CN", { month: "2-digit", day: "2-digit" });
}

export function statusColor(status: number): {
  bg: string;
  text: string;
  label: string;
} {
  if (status >= 200 && status < 300) return { bg: "bg-emerald-50", text: "text-emerald-700", label: "success" };
  if (status >= 300 && status < 400) return { bg: "bg-blue-50", text: "text-blue-700", label: "redirect" };
  if (status >= 400 && status < 500) return { bg: "bg-amber-50", text: "text-amber-700", label: "client error" };
  if (status >= 500) return { bg: "bg-red-50", text: "text-red-700", label: "server error" };
  return { bg: "bg-gray-50", text: "text-gray-700", label: "unknown" };
}

export function methodColor(method: string): string {
  const m = method.toUpperCase();
  switch (m) {
    case "GET": return "text-method-get bg-emerald-50";
    case "POST": return "text-method-post bg-blue-50";
    case "PUT": return "text-method-put bg-amber-50";
    case "PATCH": return "text-method-patch bg-violet-50";
    case "DELETE": return "text-method-delete bg-red-50";
    case "HEAD": return "text-method-head bg-gray-100";
    case "OPTIONS": return "text-method-options bg-gray-100";
    default: return "text-gray-700 bg-gray-100";
  }
}
