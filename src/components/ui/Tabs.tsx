import * as React from "react";
import { cn } from "@/lib/utils";

export function Tabs({
  value,
  onChange,
  items,
  className,
}: {
  value: string;
  onChange: (v: string) => void;
  items: { value: string; label: string; badge?: number | string }[];
  className?: string;
}) {
  return (
    <div className={cn("flex items-center border-b border-gray-200", className)}>
      {items.map((it) => (
        <button
          key={it.value}
          onClick={() => onChange(it.value)}
          className={cn(
            "relative px-3 h-9 text-sm font-medium transition-colors flex items-center gap-1.5",
            value === it.value
              ? "text-primary-700 after:absolute after:left-0 after:right-0 after:bottom-[-1px] after:h-[2px] after:bg-primary-500"
              : "text-gray-500 hover:text-gray-900"
          )}
        >
          {it.label}
          {it.badge != null && (
            <span className={cn(
              "px-1.5 h-4 inline-flex items-center justify-center text-[10px] font-semibold rounded",
              value === it.value ? "bg-primary-100 text-primary-700" : "bg-gray-100 text-gray-600"
            )}>
              {it.badge}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}
