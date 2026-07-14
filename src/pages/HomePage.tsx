import * as React from "react";
import { RequestPanel } from "@/components/RequestPanel/RequestPanel";
import { ResponsePanel } from "@/components/ResponsePanel/ResponsePanel";
import { useRequestStore } from "@/store/requestStore";
import { useDataStore } from "@/store/dataStore";

export function HomePage() {
  const { loadRequest, resetRequest } = useRequestStore();
  const { loadHistory, loadFavorites } = useDataStore();

  React.useEffect(() => {
    // 监听侧边栏点击"加载请求"
    const handler = (e: any) => {
      const req = e.detail;
      if (req) {
        loadRequest(req);
        loadHistory();
      }
    };
    window.addEventListener("zeroapi:load-request", handler);
    return () => window.removeEventListener("zeroapi:load-request", handler);
  }, [loadRequest, loadHistory]);

  // 键盘快捷键
  React.useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter") {
        e.preventDefault();
        document.getElementById("zeroapi-send-btn")?.click();
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        document.getElementById("zeroapi-fav-btn")?.click();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <div className="h-full flex flex-col">
      <RequestPanel />
      <div className="flex-1 min-h-0 border-t border-gray-200">
        <ResponsePanel />
      </div>
    </div>
  );
}
