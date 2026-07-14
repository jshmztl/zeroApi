import * as React from "react";
import type { Body } from "@/types";
import { KeyValueEditor } from "@/components/KeyValueEditor/KeyValueEditor";
import { CodeEditor } from "@/components/CodeEditor/CodeEditor";
import { cn } from "@/lib/utils";

type Mode = "none" | "form_data" | "url_encoded" | "raw";

const MODE_OPTIONS = [
  { value: "none", label: "none" },
  { value: "form_data", label: "form-data" },
  { value: "url_encoded", label: "x-www-form-urlencoded" },
  { value: "raw", label: "raw" },
];

const RAW_TYPES = [
  { value: "application/json", label: "JSON (application/json)" },
  { value: "application/xml", label: "XML (application/xml)" },
  { value: "text/plain", label: "Text (text/plain)" },
  { value: "text/html", label: "HTML (text/html)" },
  { value: "application/javascript", label: "JavaScript" },
];

export function BodyEditor({ value, onChange }: { value: Body; onChange: (b: Body) => void }) {
  const mode: Mode = value.type === "none" ? "none"
    : value.type === "form_data" ? "form_data"
    : value.type === "url_encoded" ? "url_encoded"
    : "raw";

  const handleMode = (m: Mode) => {
    if (m === mode) return;
    if (m === "none") onChange({ type: "none" });
    else if (m === "form_data") onChange({ type: "form_data", items: [] });
    else if (m === "url_encoded") onChange({ type: "url_encoded", items: [] });
    else onChange({ type: "raw", content_type: "application/json", content: "" });
  };

  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-1">
        {MODE_OPTIONS.map((o) => (
          <button
            key={o.value}
            onClick={() => handleMode(o.value as Mode)}
            className={cn(
              "px-2.5 h-7 text-xs rounded-md transition-colors",
              mode === o.value
                ? "bg-primary-50 text-primary-700 border border-primary-200"
                : "text-gray-600 hover:bg-gray-100 border border-transparent"
            )}
          >
            {o.label}
          </button>
        ))}
      </div>

      {value.type === "none" && (
        <div className="text-center py-6 text-xs text-gray-400">
          请求体为空
        </div>
      )}

      {value.type === "form_data" && (
        <KeyValueEditor
          value={value.items}
          onChange={(items) => onChange({ type: "form_data", items })}
          keyPlaceholder="字段名"
          valuePlaceholder="值"
        />
      )}

      {value.type === "url_encoded" && (
        <KeyValueEditor
          value={value.items}
          onChange={(items) => onChange({ type: "url_encoded", items })}
          keyPlaceholder="字段名"
          valuePlaceholder="值"
          bulkPaste
        />
      )}

      {value.type === "raw" && (
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <label className="text-[10px] text-gray-500">Content-Type:</label>
            <select
              value={value.content_type}
              onChange={(e) => onChange({ ...value, content_type: e.target.value })}
              className="h-7 px-2 text-xs border border-gray-200 rounded focus:outline-none focus:ring-2 focus:ring-primary-500/30"
            >
              {RAW_TYPES.map((t) => (
                <option key={t.value} value={t.value}>{t.label}</option>
              ))}
            </select>
            <button
              onClick={() => {
                try {
                  const pretty = JSON.stringify(JSON.parse(value.content), null, 2);
                  onChange({ ...value, content: pretty });
                } catch {}
              }}
              className="text-[10px] text-primary-600 hover:text-primary-700"
            >
              美化 JSON
            </button>
          </div>
          <CodeEditor
            value={value.content}
            onChange={(v) => onChange({ ...value, content: v })}
            language={value.content_type.includes("json") ? "json" : value.content_type.includes("xml") ? "xml" : value.content_type.includes("html") ? "html" : "text"}
            height="220px"
          />
        </div>
      )}
    </div>
  );
}
