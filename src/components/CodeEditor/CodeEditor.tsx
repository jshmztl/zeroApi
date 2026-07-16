import * as React from "react";
import Editor, { loader } from "@monaco-editor/react";
import { useSettingsStore } from "@/store/settingsStore";

// 配置 Monaco Editor 使用本地打包 workers（不依赖 CDN）
// vite-plugin-monaco-editor 在构建时将 workers 输出到 /monacoeditorwork/
// 开发模式下用 CDN 方便开发调试，生产模式直接加载本地资源
loader.config({
  paths: {
    vs: import.meta.env.PROD
      ? "/monacoeditorwork"
      : "https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs",
  },
});

export function CodeEditor({
  value, onChange, language = "text", height = "200px", readOnly = false,
}: {
  value: string;
  onChange?: (v: string) => void;
  language?: string;
  height?: string | number;
  readOnly?: boolean;
}) {
  const { settings } = useSettingsStore();
  const isDark = settings.theme === "dark" ||
    (settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  return (
    <div className={
      "border border-gray-200 dark:border-gray-700 rounded-md overflow-hidden bg-white dark:bg-gray-900"
      + (height === "100%" ? " h-full" : "")
    }>
      <Editor
        value={value}
        onChange={(v) => onChange?.(v ?? "")}
        language={language}
        height={height}
        theme={isDark ? "vs-dark" : "vs"}
        loading={
          <div className="flex items-center justify-center h-full text-xs text-gray-400">
            加载编辑器...
          </div>
        }
        options={{
          fontSize: 12,
          fontFamily: "JetBrains Mono, Fira Code, Consolas, monospace",
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          lineNumbers: "on",
          wordWrap: "on",
          renderLineHighlight: "none",
          scrollbar: {
            vertical: "auto",
            horizontal: "auto",
            verticalScrollbarSize: 6,
            horizontalScrollbarSize: 6,
          },
          padding: { top: 8, bottom: 8 },
          readOnly,
          tabSize: 2,
          automaticLayout: true,
        }}
      />
    </div>
  );
}
