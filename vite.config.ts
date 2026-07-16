import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";
import monacoEditorPluginFn from "vite-plugin-monaco-editor";
const monacoEditorPlugin =
  (monacoEditorPluginFn as any).default || monacoEditorPluginFn;

// Tauri 2 默认固定 1420 端口,避免被代理
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [
    react(),
    // Monaco Editor 内置打包：将 workers 打包进构建产物，不依赖 CDN
    monacoEditorPlugin({
      languageWorkers: ["json", "typescript", "editorWorkerService"],
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // 防止 Vite 隐藏 rust 错误
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? { protocol: "ws", host, port: 1421 }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    target: ["es2021", "chrome105", "safari14"],
    minify: "esbuild",
    sourcemap: false,
    chunkSizeWarningLimit: 1500,
  },
});
