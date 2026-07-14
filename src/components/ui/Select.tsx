import * as React from "react";
import { cn } from "@/lib/utils";

interface SelectProps {
  value: string;
  onChange: (v: string) => void;
  options: { value: string; label: string; color?: string }[];
  className?: string;
  size?: "sm" | "md";
}

export function Select({ value, onChange, options, className, size = "md" }: SelectProps) {
  return (
    <div className={cn("relative inline-block", className)}>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className={cn(
          "appearance-none cursor-pointer bg-white border border-gray-200 rounded-md",
          "font-medium focus:outline-none focus:ring-2 focus:ring-primary-500/30",
          "focus:border-primary-500 pr-7",
          size === "md" ? "h-9 pl-3 text-sm" : "h-7 pl-2.5 text-xs"
        )}
        style={{
          color: options.find((o) => o.value === value)?.color,
        }}
      >
        {options.map((o) => (
          <option key={o.value} value={o.value} style={{ color: o.color }}>
            {o.label}
          </option>
        ))}
      </select>
      <svg
        className="pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-gray-400"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="2"
      >
        <polyline points="6 9 12 15 18 9" />
      </svg>
    </div>
  );
}
