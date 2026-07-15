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
    <div className={cn("flex items-center border-b border-gray-200 dark:border-gray-800", className)}>
      {items.map((it) => (
        <button
          key={it.value}
          onClick={() => onChange(it.value)}
          className={cn(
            "relative px-3 h-9 text-sm font-medium transition-colors flex items-center gap-1.5",
            value === it.value
              ? "text-primary-700 dark:text-primary-400 after:absolute after:left-0 after:right-0 after:bottom-[-1px] after:h-[2px] after:bg-primary-500"
              : "text-gray-500 dark:text-gray-500 hover:text-gray-900 dark:hover:text-gray-300"
          )}
        >
          {it.label}
          {it.badge != null && (
            <span className={cn(
              "px-1.5 h-4 inline-flex items-center justify-center text-[10px] font-semibold rounded",
              value === it.value ? "bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400" : "bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400"
            )}>
              {it.badge}
            </span>
          )}
        </button>
      ))}
    </div>
  );
}
