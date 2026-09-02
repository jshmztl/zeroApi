/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    container: {
      center: true,
      padding: "2rem",
      screens: { "2xl": "1400px" },
    },
    extend: {
      colors: {
        border: "hsl(var(--border))",
        input: "hsl(var(--input))",
        ring: "hsl(var(--ring))",
        background: "hsl(var(--background))",
        foreground: "hsl(var(--foreground))",
        primary: {
          50: "#EDFBFF",
          100: "#D2F1FC",
          200: "#A9E4F8",
          300: "#71CFEF",
          400: "#3FB7E0",
          500: "#1FA3D2",
          600: "#1286B8",
          700: "#116280",
          800: "#144F66",
          900: "#153F52",
          DEFAULT: "#1FA3D2",
          foreground: "#0A2733",
        },
        secondary: {
          DEFAULT: "hsl(var(--secondary))",
          foreground: "hsl(var(--secondary-foreground))",
        },
        destructive: {
          DEFAULT: "#F0435A",
          foreground: "#FFFFFF",
        },
        // 中性色下沉为"墨水/石墨"色阶，作为仪器控制台的画布基准
        gray: {
          50: "#F5F6F4",
          100: "#E9EBE7",
          200: "#D6DAD2",
          300: "#BBC2B6",
          400: "#969E90",
          500: "#757E70",
          600: "#585F55",
          700: "#3F443D",
          800: "#272B27",
          900: "#171A18",
          950: "#0E1110",
        },
        muted: {
          DEFAULT: "hsl(var(--muted))",
          foreground: "hsl(var(--muted-foreground))",
        },
        accent: {
          DEFAULT: "hsl(var(--accent))",
          foreground: "hsl(var(--accent-foreground))",
        },
        popover: {
          DEFAULT: "hsl(var(--popover))",
          foreground: "hsl(var(--popover-foreground))",
        },
        card: {
          DEFAULT: "hsl(var(--card))",
          foreground: "hsl(var(--card-foreground))",
        },
        // HTTP 方法颜色（语义色，独立于主强调色）
        method: {
          get: "#43D6A0",
          post: "#5C9BEF",
          put: "#F0A53C",
          patch: "#A88CF0",
          delete: "#F0605F",
          head: "#96A0AC",
          options: "#96A0AC",
        },
        // 语义/状态色
        success: "#43D6A0",
        warning: "#F0A53C",
        danger: "#F0605F",
        info: "#5C9BEF",
      },
      fontFamily: {
        // 仪器控制台：Saira(表壳/标题) + Chivo(正文) + JetBrains Mono(数据/代码)
        display: ["Saira Variable", "PingFang SC", "Microsoft YaHei", "sans-serif"],
        sans: [
          "Chivo Variable",
          "PingFang SC",
          "Hiragino Sans GB",
          "Microsoft YaHei",
          "-apple-system",
          "BlinkMacSystemFont",
          "Segoe UI",
          "Roboto",
          "sans-serif",
        ],
        mono: [
          "JetBrains Mono Variable",
          "JetBrains Mono",
          "Fira Code",
          "Consolas",
          "Monaco",
          "monospace",
        ],
      },
      borderRadius: {
        lg: "10px",
        md: "7px",
        sm: "4px",
        xl: "14px",
      },
      keyframes: {
        "accordion-down": {
          from: { height: "0" },
          to: { height: "var(--radix-accordion-content-height)" },
        },
        "accordion-up": {
          from: { height: "var(--radix-accordion-content-height)" },
          to: { height: "0" },
        },
        "fade-in": {
          from: { opacity: "0", transform: "translateY(4px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        "spin-slow": {
          to: { transform: "rotate(360deg)" },
        },
        // 磷光信号脉冲：活动状态灯/传输中
        "signal-pulse": {
          "0%, 100%": { opacity: "1", "box-shadow": "0 0 0 0 hsl(var(--ring) / 0.4)" },
          "50%": { opacity: "0.6", "box-shadow": "0 0 0 4px hsl(var(--ring) / 0)" },
        },
        // 网络轨迹描线，模拟示波器扫描
        trace: {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(220%)" },
        },
      },
      animation: {
        "accordion-down": "accordion-down 0.2s ease-out",
        "accordion-up": "accordion-up 0.2s ease-out",
        "fade-in": "fade-in 200ms ease-out",
        "spin-slow": "spin-slow 1s linear infinite",
        "signal-pulse": "signal-pulse 1.6s ease-in-out infinite",
        trace: "trace 1.8s linear infinite",
      },
    },
  },
  plugins: [],
};