// 格式化工具
export function tryPrettyJSON(input: string): { ok: boolean; text: string } {
  if (!input.trim()) return { ok: false, text: input };
  try {
    const obj = JSON.parse(input);
    return { ok: true, text: JSON.stringify(obj, null, 2) };
  } catch {
    return { ok: false, text: input };
  }
}

export function tryDetectContentType(headers: Record<string, string>): string {
  const ct = headers["content-type"] || headers["Content-Type"] || "";
  return ct.split(";")[0].trim();
}

export function languageFromContentType(ct: string): string {
  if (!ct) return "text";
  const lower = ct.toLowerCase();
  if (lower.includes("json")) return "json";
  if (lower.includes("xml")) return "xml";
  if (lower.includes("html")) return "html";
  if (lower.includes("javascript")) return "javascript";
  if (lower.includes("css")) return "css";
  if (lower.includes("yaml")) return "yaml";
  return "text";
}

export function buildFullUrl(url: string, params: { key: string; value: string; enabled: boolean }[]): string {
  if (!url) return "";
  const enabled = params.filter((p) => p.enabled && p.key);
  if (enabled.length === 0) return url;
  const sep = url.includes("?") ? "&" : "?";
  const qs = enabled
    .map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
    .join("&");
  return url + sep + qs;
}

export function substituteVars(input: string, vars: Record<string, string>): string {
  if (!input) return input;
  return input.replace(/\{\{([^}]+)\}\}/g, (m, key) => {
    const trimmed = String(key).trim();
    return Object.prototype.hasOwnProperty.call(vars, trimmed) ? vars[trimmed] : m;
  });
}

export function copyToClipboard(text: string): Promise<boolean> {
  if (navigator.clipboard) {
    return navigator.clipboard.writeText(text).then(() => true).catch(() => false);
  }
  // fallback
  try {
    const ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.left = "-9999px";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
    return Promise.resolve(true);
  } catch {
    return Promise.resolve(false);
  }
}

export function downloadText(text: string, filename: string) {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
