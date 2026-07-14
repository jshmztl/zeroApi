import * as React from "react";
import { cn } from "@/lib/utils";

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  invalid?: boolean;
}

export const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({ className, invalid, ...props }, ref) => (
    <input
      ref={ref}
      className={cn(
        "h-9 w-full px-3 text-sm bg-white border rounded-md",
        "placeholder:text-gray-400 transition-colors",
        "focus:outline-none focus:ring-2 focus:ring-primary-500/30",
        invalid
          ? "border-red-300 focus:border-red-400"
          : "border-gray-200 focus:border-primary-500",
        className
      )}
      {...props}
    />
  )
);
Input.displayName = "Input";
