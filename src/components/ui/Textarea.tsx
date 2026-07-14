import * as React from "react";
import { cn } from "@/lib/utils";

export interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {}

export const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, ...props }, ref) => (
    <textarea
      ref={ref}
      className={cn(
        "w-full px-3 py-2 text-sm bg-white border border-gray-200 rounded-md",
        "placeholder:text-gray-400 transition-colors resize-none",
        "focus:outline-none focus:ring-2 focus:ring-primary-500/30 focus:border-primary-500",
        "font-mono",
        className
      )}
      {...props}
    />
  )
);
Textarea.displayName = "Textarea";
