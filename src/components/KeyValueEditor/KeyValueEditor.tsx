import * as React from "react";
import { Plus, Trash2, Check } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";
import type { KeyValue } from "@/types";
import { cn } from "@/lib/utils";

export function KeyValueEditor({
  value, onChange,
  keyPlaceholder, valuePlaceholder,
  presets, bulkPaste,
}: {
  value: KeyValue[];
  onChange: (v: KeyValue[]) => void;
  keyPlaceholder?: string;
  valuePlaceholder?: string;
  presets?: { key: string; value: string }[];
  bulkPaste?: boolean;
}) {
  const add = (preset?: { key: string; value: string }) => {
    onChange([
      ...value,
      preset
        ? { key: preset.key, value: preset.value, enabled: true }
        : { key: "", value: "", enabled: true },
    ]);
  };
  const update = (i: number, patch: Partial<KeyValue>) => {
    onChange(value.map((v, idx) => (idx === i ? { ...v, ...patch } : v)));
  };
  const remove = (i: number) => {
    onChange(value.filter((_, idx) => idx !== i));
  };
  const handleBulkPaste = (text: string) => {
    if (!text.trim()) return;
    const lines = text.split(/\r?\n/);
    const newItems: KeyValue[] = [];
    for (const line of lines) {
      const t = line.trim();
      if (!t) continue;
      // 支持 key=value / key:value / key\tvalue / 纯 key
      const m = t.match(/^([^=:]+)[:=]\s*(.*)$/);
      if (m) {
        newItems.push({ key: m[1].trim(), value: m[2].trim(), enabled: true });
      } else {
        newItems.push({ key: t, value: "", enabled: true });
      }
    }
    if (newItems.length > 0) {
      onChange([...value, ...newItems]);
    }
  };

  return (
    <div className="space-y-1.5">
      {presets && presets.length > 0 && (
        <div className="flex items-center gap-1 mb-2">
          <span className="text-[10px] text-gray-400">快速添加:</span>
          {presets.map((p) => (
            <button
              key={p.key}
              onClick={() => add(p)}
              className="text-[10px] px-1.5 py-0.5 bg-primary-50 text-primary-700 hover:bg-primary-100 rounded transition-colors"
            >
              + {p.key}
            </button>
          ))}
        </div>
      )}

      {value.length === 0 && (
        <div className="text-center py-4 text-xs text-gray-400">
          还没有任何条目
        </div>
      )}

      {value.map((kv, i) => (
        <div key={i} className="flex items-center gap-1.5">
          <input
            type="checkbox"
            checked={kv.enabled}
            onChange={(e) => update(i, { enabled: e.target.checked })}
            className="rounded text-primary-500 focus:ring-primary-500"
          />
          <Input
            value={kv.key}
            onChange={(e) => update(i, { key: e.target.value })}
            placeholder={keyPlaceholder}
            className="flex-1 font-mono text-xs"
          />
          <Input
            value={kv.value}
            onChange={(e) => update(i, { value: e.target.value })}
            placeholder={valuePlaceholder}
            className="flex-1 font-mono text-xs"
          />
          <button
            onClick={() => remove(i)}
            className="text-gray-300 hover:text-red-500 p-1"
            title="删除"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      ))}

      <div className="flex items-center gap-2 pt-1">
        <Button variant="ghost" size="sm" onClick={() => add()}>
          <Plus className="h-3 w-3" />
          添加
        </Button>
        {bulkPaste && (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              const text = prompt("批量粘贴(每行一对 key=value):");
              if (text) handleBulkPaste(text);
            }}
          >
            批量粘贴
          </Button>
        )}
      </div>
    </div>
  );
}
