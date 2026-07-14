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
        "bg-white border border-gray-100 rounded-xl shadow-sm",
        className
      )}
      {...rest}
    >
      {children}
    </div>
  );
}
