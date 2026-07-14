import * as React from "react";
import { Star, History, Globe, FolderOpen, Upload, Plus, Trash2 } from "lucide-react";
import { Link } from "react-router-dom";
import { useDataStore } from "@/store/dataStore";
import { cn, formatDate } from "@/lib/utils";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { EnvironmentDialog } from "@/components/Sidebar/EnvironmentDialog";
import type { Environment } from "@/types";

type Section = "favorites" | "history" | "collections" | "environments";

export function Sidebar() {
  const {
    favorites, removeFavorite, loadFavorites,
    history, clearHistory, loadHistory,
    collections, loadCollections, removeCollection,
    environments, removeEnvironment, loadEnvironments,
    activeEnvId, setActiveEnv,
  } = useDataStore();

  const [section, setSection] = React.useState<Section>("favorites");
  const [envDialog, setEnvDialog] = React.useState(false);
  const [editingEnv, setEditingEnv] = React.useState<Environment | null>(null);

  React.useEffect(() => {
    loadFavorites();
    loadHistory();
    loadCollections();
    loadEnvironments();
  }, [loadFavorites, loadHistory, loadCollections, loadEnvironments]);

  const nav = [
    { key: "favorites" as Section, label: "收藏", icon: Star, count: favorites.length },
    { key: "history" as Section, label: "历史", icon: History, count: history.length },
    { key: "collections" as Section, label: "集合", icon: FolderOpen, count: collections.length },
    { key: "environments" as Section, label: "环境", icon: Globe, count: environments.length },
  ];

  return (
    <aside className="w-64 border-r border-gray-200 bg-gray-50 flex flex-col overflow-hidden">
      <div className="px-3 py-2 flex items-center gap-1 border-b border-gray-200 bg-gray-50">
        {nav.map((n) => {
          const Icon = n.icon;
          const active = section === n.key;
          return (
            <button
              key={n.key}
              onClick={() => setSection(n.key)}
              className={cn(
                "flex-1 h-8 rounded-md flex items-center justify-center gap-1 text-xs font-medium transition-colors",
                active
                  ? "bg-white text-primary-700 shadow-sm"
                  : "text-gray-500 hover:text-gray-700 hover:bg-white/60"
              )}
              title={n.label}
            >
              <Icon className="h-3.5 w-3.5" />
              <span>{n.label}</span>
              {n.count > 0 && (
                <span className="text-[10px] text-gray-400">{n.count}</span>
              )}
            </button>
          );
        })}
      </div>

      <div className="flex-1 overflow-auto p-2">
        {section === "favorites" && (
          <FavoritesList
            items={favorites}
            onRemove={async (id: string) => {
              await removeFavorite(id);
              toast.success("已删除收藏");
            }}
          />
        )}
        {section === "history" && (
          <HistoryList
            items={history}
            onClear={async () => {
              if (confirm("清空所有历史记录？")) {
                await clearHistory();
                toast.success("已清空历史");
              }
            }}
          />
        )}
        {section === "collections" && (
          <CollectionsList
            items={collections}
            onRemove={async (id: string) => {
              await removeCollection(id);
              toast.success("已删除集合");
            }}
          />
        )}
        {section === "environments" && (
          <EnvironmentsList
            items={environments}
            activeId={activeEnvId}
            onSetActive={setActiveEnv}
            onEdit={(env: Environment) => { setEditingEnv(env); setEnvDialog(true); }}
            onRemove={async (id: string) => {
              await removeEnvironment(id);
              toast.success("已删除环境");
            }}
            onNew={() => { setEditingEnv(null); setEnvDialog(true); }}
          />
        )}
      </div>

      <div className="px-3 py-2 border-t border-gray-200 flex items-center gap-1">
        <Link
          to="/import"
          className="flex-1 h-7 px-2 inline-flex items-center justify-center gap-1 text-xs text-gray-600 hover:bg-white rounded"
        >
          <Upload className="h-3 w-3" />
          导入 cURL / JSON
        </Link>
      </div>

      {envDialog && (
        <EnvironmentDialog
          env={editingEnv}
          onClose={() => setEnvDialog(false)}
        />
      )}
    </aside>
  );
}

function FavoritesList({ items, onRemove }: { items: any[]; onRemove: (id: string) => void }) {
  if (items.length === 0) {
    return <EmptyState tip="还没有收藏。点击 ⭐ 收藏一个请求。" />;
  }
  return (
    <div className="space-y-1">
      {items.map((f) => (
        <RequestListItem
          key={f.id}
          item={f.request}
          right={
            <button
              onClick={(e) => { e.stopPropagation(); onRemove(f.id); }}
              className="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 p-0.5"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          }
        />
      ))}
    </div>
  );
}

function HistoryList({ items, onClear }: { items: any[]; onClear: () => void }) {
  if (items.length === 0) {
    return <EmptyState tip="暂无历史记录" />;
  }
  return (
    <div className="space-y-1">
      <div className="flex justify-end mb-1">
        <button
          onClick={onClear}
          className="text-[10px] text-gray-400 hover:text-red-500"
        >
          清空
        </button>
      </div>
      {items.map((h) => (
        <RequestListItem
          key={h.id}
          item={h.request}
          subtitle={`${h.response.status} · ${h.response.time_ms}ms`}
          right={
            <span className="text-[10px] text-gray-400">{formatDate(h.created_at)}</span>
          }
        />
      ))}
    </div>
  );
}

function CollectionsList({ items, onRemove }: { items: any[]; onRemove: (id: string) => void }) {
  if (items.length === 0) {
    return <EmptyState tip="暂无集合" />;
  }
  return (
    <div className="space-y-1">
      {items.map((c) => (
        <div
          key={c.id}
          className="group px-2 py-1.5 rounded hover:bg-white cursor-pointer"
        >
          <div className="flex items-center justify-between">
            <div className="min-w-0 flex-1">
              <div className="text-xs font-medium text-gray-700 truncate">{c.name}</div>
              <div className="text-[10px] text-gray-400">
                {c.request_ids.length} 个请求
              </div>
            </div>
            <button
              onClick={() => onRemove(c.id)}
              className="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500 p-0.5"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </div>
        </div>
      ))}
    </div>
  );
}

function EnvironmentsList({ items, activeId, onSetActive, onEdit, onRemove, onNew }: {
  items: Environment[];
  activeId: string | null;
  onSetActive: (id: string | null) => void;
  onEdit: (env: Environment) => void;
  onRemove: (id: string) => void;
  onNew: () => void;
}) {
  return (
    <div className="space-y-1">
      <button
        onClick={onNew}
        className="w-full h-7 px-2 text-xs text-primary-700 hover:bg-primary-50 rounded inline-flex items-center gap-1 justify-center"
      >
        <Plus className="h-3 w-3" /> 新建环境
      </button>
      {items.length === 0 ? (
        <EmptyState tip="还没有环境变量" />
      ) : (
        items.map((e) => (
          <div
            key={e.id}
            className="group px-2 py-1.5 rounded hover:bg-white cursor-pointer"
            onClick={() => onSetActive(activeId === e.id ? null : e.id)}
          >
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-1.5 min-w-0 flex-1">
                <div className={`w-1.5 h-1.5 rounded-full ${activeId === e.id ? "bg-primary-500" : "bg-gray-300"}`} />
                <div className="min-w-0 flex-1">
                  <div className="text-xs font-medium text-gray-700 truncate">{e.name}</div>
                  <div className="text-[10px] text-gray-400">{e.vars.length} 变量</div>
                </div>
              </div>
              <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100">
                <button
                  onClick={(ev) => { ev.stopPropagation(); onEdit(e); }}
                  className="text-gray-400 hover:text-primary-600 text-[10px] px-1"
                >
                  编辑
                </button>
                <button
                  onClick={(ev) => { ev.stopPropagation(); onRemove(e.id); }}
                  className="text-gray-400 hover:text-red-500 p-0.5"
                >
                  <Trash2 className="h-3 w-3" />
                </button>
              </div>
            </div>
          </div>
        ))
      )}
    </div>
  );
}

function RequestListItem({
  item, subtitle, right,
}: {
  item: any;
  subtitle?: string;
  right?: React.ReactNode;
}) {
  return (
    <div
      onClick={() => {
        window.dispatchEvent(new CustomEvent("zeroapi:load-request", { detail: item }));
      }}
      className="group px-2 py-1.5 rounded hover:bg-white cursor-pointer"
    >
      <div className="flex items-center justify-between gap-1">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            <span className={cn(
              "px-1 h-4 inline-flex items-center text-[9px] font-bold rounded",
              "bg-gray-100 text-gray-700"
            )}>
              {item.method}
            </span>
            <span className="text-xs text-gray-700 truncate">
              {item.name || item.url || "(空)"}
            </span>
          </div>
          {subtitle && (
            <div className="text-[10px] text-gray-400 mt-0.5 truncate">{subtitle}</div>
          )}
        </div>
        {right}
      </div>
    </div>
  );
}

function EmptyState({ tip }: { tip: string }) {
  return (
    <div className="px-3 py-8 text-center text-xs text-gray-400">
      {tip}
    </div>
  );
}
