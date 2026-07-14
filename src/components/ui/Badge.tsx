import * as React from "react";
import { cn } from "@/lib/utils";

export function Badge({
  children,
  variant = "default",
  className,
}: {
  children: React.ReactNode;
  variant?: "default" | "primary" | "success" | "warning" | "danger" | "info" | "method";
  className?: string;
}) {
  const variantClass: Record<string, string> = {
    default: "bg-gray-100 text-gray-700",
    primary: "bg-primary-50 text-primary-700",
    success: "bg-emerald-50 text-emerald-700",
    warning: "bg-amber-50 text-amber-700",
    danger: "bg-red-50 text-red-700",
    info: "bg-blue-50 text-blue-700",
    method: "bg-gray-100 text-gray-700",
  };
  return (
    <span
      className={cn(
        "inline-flex items-center px-1.5 h-5 text-[10px] font-semibold uppercase tracking-wide rounded",
        variantClass[variant],
        className
      )}
    >
      {children}
    </span>
  );
}
