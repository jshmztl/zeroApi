import * as React from "react";
import { cn } from "@/lib/utils";

export function Card({
  className,
  children,
  ...rest
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "bg-card border border-border rounded-xl shadow-soft transition-shadow hover:shadow-lift",
        className
      )}
      {...rest}
    >
      {children}
    </div>
  );
}
