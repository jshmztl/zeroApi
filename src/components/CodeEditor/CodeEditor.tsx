import * as React from "react";
import Editor, { loader } from "@monaco-editor/react";
import { useSettingsStore } from "@/store/settingsStore";

// 预配置 CDN 源，支持多 CDN 切换
// jsdelivr 国内可用性较高，如果失败可尝试 unpkg
loader.config({
  paths: {
    vs: "https://cdn.jsdelivr.net/npm/monaco-editor@0.55.1/min/vs",
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
  const [monacoReady, setMonacoReady] = React.useState<boolean | null>(null);
  const isDark = settings.theme === "dark" ||
    (settings.theme === "system" && window.matchMedia("(prefers-color-scheme: dark)").matches);

  // 加载超时检测：5秒后若 Monaco 仍未挂载，自动降级为 textarea
  React.useEffect(() => {
    const timer = setTimeout(() => {
      if (monacoReady === null) {
        setMonacoReady(false);
      }
    }, 5000);
    return () => clearTimeout(timer);
  }, [monacoReady]);

  // 降级方案：普通 textarea（Monaco 加载失败时的回退）
  if (monacoReady === false) {
    return (
      <div
        className="border border-gray-200 dark:border-gray-700 rounded-md overflow-hidden bg-white dark:bg-gray-900"
        style={{ height }}
      >
        <textarea
          className="w-full h-full bg-transparent p-3 font-mono text-xs resize-none focus:outline-none text-gray-800 dark:text-gray-200 placeholder-gray-400"
          value={value}
          onChange={(e) => onChange?.(e.target.value)}
          readOnly={readOnly}
          placeholder="输入请求体..."
        />
      </div>
    );
  }

  const loadingElement = (
    <div
      className="flex items-center justify-center text-xs text-gray-400 bg-gray-50 dark:bg-gray-800/50"
      style={{ height }}
    >
      加载编辑器...
    </div>
  );

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
        loading={loadingElement}
        onMount={() => setMonacoReady(true)}
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
