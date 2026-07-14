import * as React from "react";
import type { Auth } from "@/types";
import { Input } from "@/components/ui/Input";
import { cn } from "@/lib/utils";

const TYPES = [
  { value: "none", label: "No Auth" },
  { value: "bearer", label: "Bearer Token" },
  { value: "basic", label: "Basic Auth" },
  { value: "api_key", label: "API Key" },
];

export function AuthEditor({ value, onChange }: { value: Auth; onChange: (a: Auth) => void }) {
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-1">
        {TYPES.map((o) => (
          <button
            key={o.value}
            onClick={() => {
              if (o.value === "none") onChange({ type: "none" });
              else if (o.value === "bearer") onChange({ type: "bearer", token: "" });
              else if (o.value === "basic") onChange({ type: "basic", username: "", password: "" });
              else onChange({ type: "api_key", key: "", value: "", location: "header" });
            }}
            className={cn(
              "px-2.5 h-7 text-xs rounded-md transition-colors",
              value.type === o.value
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
          不使用鉴权
        </div>
      )}

      {value.type === "bearer" && (
        <div>
          <label className="block text-xs font-medium text-gray-600 mb-1">Token</label>
          <Input
            value={value.token}
            onChange={(e) => onChange({ ...value, token: e.target.value })}
            placeholder="Bearer Token (不需包含 'Bearer ')"
            className="font-mono"
          />
          <p className="mt-1 text-[10px] text-gray-400">
            请求时会自动添加 <code className="text-primary-600">Authorization: Bearer ...</code> 头
          </p>
        </div>
      )}

      {value.type === "basic" && (
        <div className="grid grid-cols-2 gap-2">
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">Username</label>
            <Input
              value={value.username}
              onChange={(e) => onChange({ ...value, username: e.target.value })}
              placeholder="用户名"
            />
          </div>
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">Password</label>
            <Input
              type="password"
              value={value.password}
              onChange={(e) => onChange({ ...value, password: e.target.value })}
              placeholder="密码"
            />
          </div>
        </div>
      )}

      {value.type === "api_key" && (
        <div className="space-y-2">
          <div className="grid grid-cols-2 gap-2">
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">Key</label>
              <Input
                value={value.key}
                onChange={(e) => onChange({ ...value, key: e.target.value })}
                placeholder="X-API-Key"
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-600 mb-1">Value</label>
              <Input
                value={value.value}
                onChange={(e) => onChange({ ...value, value: e.target.value })}
                placeholder="value"
              />
            </div>
          </div>
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">Add to</label>
            <div className="flex gap-1">
              {["header", "query"].map((loc) => (
                <button
                  key={loc}
                  onClick={() => onChange({ ...value, location: loc as "header" | "query" })}
                  className={cn(
                    "px-2.5 h-7 text-xs rounded-md transition-colors",
                    value.location === loc
                      ? "bg-primary-50 text-primary-700 border border-primary-200"
                      : "text-gray-600 hover:bg-gray-100 border border-transparent"
                  )}
                >
                  {loc === "header" ? "Header" : "Query Params"}
                </button>
              ))}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
