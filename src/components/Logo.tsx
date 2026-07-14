import * as React from "react";

export function Logo({ size = 32 }: { size?: number }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="logoGradient" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor="#9DD9BB" />
          <stop offset="100%" stopColor="#4F9C7B" />
        </linearGradient>
      </defs>
      <rect x="2" y="2" width="60" height="60" rx="14" fill="url(#logoGradient)" />
      {/* 镂空 Z 字 */}
      <path
        d="M 18 20 L 46 20 L 46 27 L 28 41 L 46 41 L 46 48 L 18 48 L 18 41 L 36 27 L 18 27 Z"
        fill="white"
        fillRule="evenodd"
      />
      {/* API 端点圆点 */}
      <circle cx="50" cy="32" r="3.5" fill="white" />
      <line x1="46" y1="32" x2="56" y2="32" stroke="white" strokeWidth="2" strokeLinecap="round" />
    </svg>
  );
}
