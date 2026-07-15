import vkbeautify from 'vkbeautify';

export type PrettyLang = 'json' | 'xml' | 'html' | 'sql' | 'css' | 'text';

// 通用的多语言格式化
export function tryPretty(text: string, lang: PrettyLang): { ok: boolean; text: string } {
  if (!text?.trim()) return { ok: false, text };
  try {
    switch (lang) {
      case 'json':
        return { ok: true, text: JSON.stringify(JSON.parse(text), null, 2) };
      case 'xml':
        return { ok: true, text: vkbeautify.xml(text, 2) };
      case 'html':
        return { ok: true, text: vkbeautify.xml(text, 2) };
      case 'sql':
        return { ok: true, text: vkbeautify.sql(text, 2) };
      case 'css':
        return { ok: true, text: vkbeautify.css(text, 2) };
      default:
        return { ok: false, text };
    }
  } catch {
    return { ok: false, text };
  }
}

// 兼容旧 API
export function tryPrettyJSON(input: string): { ok: boolean; text: string } {
  return tryPretty(input, 'json');
}

export function tryDetectContentType(headers: Record<string, string>): string {
  const ct = headers['content-type'] || headers['Content-Type'] || '';
  return ct.split(';')[0].trim();
}

export function languageFromContentType(ct: string): string {
  if (!ct) return 'text';
  const lower = ct.toLowerCase();
  if (lower.includes('json')) return 'json';
  if (lower.includes('xml')) return 'xml';
  if (lower.includes('html')) return 'html';
  if (lower.includes('javascript')) return 'javascript';
  if (lower.includes('css')) return 'css';
  if (lower.includes('yaml')) return 'yaml';
  return 'text';
}

export function buildFullUrl(
  url: string,
  params: { key: string; value: string; enabled: boolean }[],
): string {
  if (!url) return '';
  const enabled = params.filter((p) => p.enabled && p.key);
  if (enabled.length === 0) return url;
  const sep = url.includes('?') ? '&' : '?';
  const qs = enabled
    .map((p) => `${encodeURIComponent(p.key)}=${encodeURIComponent(p.value)}`)
    .join('&');
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
    return navigator.clipboard
      .writeText(text)
      .then(() => true)
      .catch(() => false);
  }
  try {
    const ta = document.createElement('textarea');
    ta.value = text;
    ta.style.position = 'fixed';
    ta.style.left = '-9999px';
    document.body.appendChild(ta);
    ta.select();
    document.execCommand('copy');
    document.body.removeChild(ta);
    return Promise.resolve(true);
  } catch {
    return Promise.resolve(false);
  }
}

export function downloadText(text: string, filename: string) {
  const blob = new Blob([text], { type: 'text/plain;charset=utf-8' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
