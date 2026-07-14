import * as React from "react";
import Editor from "@monaco-editor/react";
import { useSettingsStore } from "@/store/settingsStore";

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
    <div className="border border-gray-200 rounded-md overflow-hidden bg-white">
      <Editor
        value={value}
        onChange={(v) => onChange?.(v ?? "")}
        language={language}
        height={height}
        theme={isDark ? "vs-dark" : "vs"}
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
