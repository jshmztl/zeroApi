// Shiki 代码高亮封装(按需加载以减小包体积)
import type { Highlighter, BundledLanguage, BundledTheme } from "shiki";

let highlighterPromise: Promise<Highlighter> | null = null;

const LANGS: BundledLanguage[] = [
  "json", "xml", "html", "javascript", "typescript", "css", "yaml", "markdown", "shell",
];

export function getHighlighter(): Promise<Highlighter> {
  if (!highlighterPromise) {
    highlighterPromise = import("shiki").then(async (mod) => {
      const hl = await mod.createHighlighter({
        themes: ["github-light", "github-dark"] as BundledTheme[],
        langs: LANGS,
      });
      return hl;
    });
  }
  return highlighterPromise;
}

export async function highlightCode(code: string, lang: string, isDark = false): Promise<string> {
  if (!code) return "";
  const hl = await getHighlighter();
  const safeLang = (LANGS as string[]).includes(lang) ? (lang as BundledLanguage) : "text";
  try {
    return hl.codeToHtml(code, {
      lang: safeLang as BundledLanguage,
      theme: isDark ? "github-dark" : "github-light",
    });
  } catch {
    return `<pre class="font-mono text-sm whitespace-pre-wrap">${escapeHtml(code)}</pre>`;
  }
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
