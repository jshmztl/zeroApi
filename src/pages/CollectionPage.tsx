import * as React from "react";
import { useParams, useNavigate } from "react-router-dom";
import { ArrowLeft } from "lucide-react";
import { useDataStore } from "@/store/dataStore";

export function CollectionPage() {
  const { id } = useParams();
  const nav = useNavigate();
  const { collections, favorites, loadCollections } = useDataStore();

  React.useEffect(() => {
    loadCollections();
  }, [loadCollections]);

  const col = collections.find((c) => c.id === id);

  if (!col) {
    return (
      <div className="p-8 text-center text-gray-500">
        <div>集合不存在</div>
        <button onClick={() => nav("/")} className="mt-2 text-primary-600">返回</button>
      </div>
    );
  }

  const items = favorites.filter((f) => col.request_ids.includes(f.request.id || f.id));

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <button onClick={() => nav("/")} className="text-sm text-gray-500 hover:text-gray-900 flex items-center gap-1 mb-3">
        <ArrowLeft className="h-3 w-3" /> 返回
      </button>
      <h1 className="text-xl font-semibold">{col.name}</h1>
      <p className="text-xs text-gray-500 mt-1">{col.description || "无描述"} · {items.length} 个请求</p>

      <div className="mt-4 space-y-1.5">
        {items.length === 0 && (
          <div className="text-center py-12 text-gray-400 text-sm">这个集合里还没有请求</div>
        )}
        {items.map((f) => (
          <div
            key={f.id}
            onClick={() => {
              window.dispatchEvent(new CustomEvent("zeroapi:load-request", { detail: f.request }));
              nav("/");
            }}
            className="px-3 py-2 border border-gray-100 rounded-lg hover:border-primary-300 cursor-pointer bg-white"
          >
            <div className="flex items-center gap-2">
              <span className="text-[10px] font-bold text-primary-700">{f.request.method}</span>
              <span className="text-sm">{f.request.name || f.request.url}</span>
            </div>
            <div className="text-[10px] text-gray-400 font-mono mt-0.5 truncate">{f.request.url}</div>
          </div>
        ))}
      </div>
    </div>
  );
}
