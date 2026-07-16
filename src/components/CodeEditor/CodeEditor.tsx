import * as React from "react";
import Editor, { loader } from "@monaco-editor/react";
import * as monaco from "monaco-editor";
import { useSettingsStore } from "@/store/settingsStore";

// 使用 Vite 已打包的 monaco-editor 实例，避免运行时时从 URL 加载
// （打包安装后路径解析失败会导致编辑器一直卡在 Loading 状态）
loader.config({ monaco });

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
