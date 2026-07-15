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
        "h-9 w-full px-3 text-sm bg-white dark:bg-gray-800 border rounded-md",
        "placeholder:text-gray-400 dark:placeholder:text-gray-500 transition-colors",
        "focus:outline-none focus:ring-2 focus:ring-primary-500/30",
        "text-gray-900 dark:text-gray-100",
        invalid
          ? "border-red-300 dark:border-red-800 focus:border-red-400"
          : "border-gray-200 dark:border-gray-700 focus:border-primary-500",
        className
      )}
      {...props}
    />
  )
);
Input.displayName = "Input";
