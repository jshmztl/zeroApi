import * as React from "react";
import { cn } from "@/lib/utils";

type Variant = "default" | "primary" | "ghost" | "outline" | "danger" | "subtle";
type Size = "sm" | "md" | "lg" | "icon";

const variantClass: Record<Variant, string> = {
  default: "bg-gray-900 text-white hover:bg-gray-800",
  primary: "bg-primary-500 text-white hover:bg-primary-600 shadow-sm",
  ghost: "text-gray-700 hover:bg-gray-100",
  outline: "border border-gray-200 text-gray-700 hover:bg-gray-50",
  danger: "bg-white border border-red-200 text-red-600 hover:bg-red-50",
  subtle: "bg-primary-50 text-primary-700 hover:bg-primary-100",
};

const sizeClass: Record<Size, string> = {
  sm: "h-7 px-2.5 text-xs gap-1",
  md: "h-9 px-3.5 text-sm gap-1.5",
  lg: "h-10 px-5 text-sm gap-2",
  icon: "h-9 w-9 p-0",
};

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
  loading?: boolean;
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = "default", size = "md", loading, className, children, disabled, ...props }, ref) => {
    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={cn(
          "inline-flex items-center justify-center font-medium rounded-md transition-all",
          "focus:outline-none focus:ring-2 focus:ring-primary-500/30",
          "active:scale-[0.98] disabled:opacity-50 disabled:pointer-events-none",
          variantClass[variant],
          sizeClass[size],
          className
        )}
        {...props}
      >
        {loading && (
          <svg className="animate-spin h-3.5 w-3.5" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" strokeOpacity="0.25" strokeWidth="3" />
            <path d="M22 12a10 10 0 0 1-10 10" stroke="currentColor" strokeWidth="3" strokeLinecap="round" />
          </svg>
        )}
        {children}
      </button>
    );
  }
);
Button.displayName = "Button";
