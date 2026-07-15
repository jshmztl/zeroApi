import * as React from 'react';
import {
  Star,
  History,
  FolderOpen,
  Upload,
  Plus,
  Trash2,
  ChevronRight,
  ChevronDown,
} from 'lucide-react';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useDataStore } from '@/store/dataStore';
import { useRequestStore } from '@/store/requestStore';
import { cn, formatDate } from '@/lib/utils';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { EnvironmentDialog } from '@/components/Sidebar/EnvironmentDialog';
import { CollectionDialog } from '@/components/Sidebar/CollectionDialog';
import { nanoid } from '@/lib/nanoid';
import type { Environment, Collection, Request, RequestStatus } from '@/types';
import { REQUEST_STATUS_META } from '@/types';
import { fullUrlDisplay } from '@/components/RequestPanel/RequestPanel';

type Section = 'collections' | 'history' | 'favorites';

export function Sidebar() {
  const {
    favorites,
    removeFavorite,
    loadFavorites,
    history,
    clearHistory,
    loadHistory,
    removeHistory,
    collections,
    loadCollections,
    removeCollection,
    environments,
    removeEnvironment,
    loadEnvironments,
    savedRequests,
    loadSavedRequests,
    activeEnvId,
    setActiveEnv,
  } = useDataStore();
  const navigate = useNavigate();
  const location = useLocation();

  const [section, setSection] = React.useState<Section>('collections');
  const [envDialog, setEnvDialog] = React.useState(false);
  const [editingEnv, setEditingEnv] = React.useState<Environment | null>(null);
  const [collectionDialog, setCollectionDialog] = React.useState(false);
  const [envDropdown, setEnvDropdown] = React.useState(false);
  const [expandedCollections, setExpandedCollections] = React.useState<Set<string>>(new Set());

  React.useEffect(() => {
    loadFavorites();
    loadHistory();
    loadCollections();
    loadEnvironments();
    loadSavedRequests();
  }, [loadFavorites, loadHistory, loadCollections, loadEnvironments, loadSavedRequests]);

  const navItems = [
    { key: 'collections' as Section, label: '集合', icon: FolderOpen, count: collections.length },
    { key: 'history' as Section, label: '历史', icon: History, count: history.length },
    { key: 'favorites' as Section, label: '收藏', icon: Star, count: favorites.length },
  ];

  const handleNewRequest = () => {
    useRequestStore.getState().resetRequest();
    navigate('/');
    toast.success('已新建空白请求');
  };

  const handleCreateCollection = async (name: string, description: string) => {
    try {
      const c: Collection = {
        id: nanoid(),
        name,
        description,
        request_ids: [],
        created_at: Date.now(),
        updated_at: Date.now(),
      };
      await tauri.saveCollection(c);
      await loadCollections();
      toast.success('集合已创建');
    } catch (e: any) {
      toast.error('创建集合失败: ' + String(e));
    }
  };

  const activeEnv = environments.find((e) => e.id === activeEnvId);
  const baseUrl = activeEnv?.base_url || '';

  const toggleExpand = (id: string) => {
    setExpandedCollections((prev) => {
      const next = new Set(prev);
      next.has(id) ? next.delete(id) : next.add(id);
      return next;
    });
  };

  return (
    <aside className="w-64 border-r border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-950 flex flex-col overflow-hidden">
      {/* 导航标签 + 新建 */}
      <div className="px-2 py-2 flex items-center gap-1 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-950">
        {navItems.map((n) => {
          const Icon = n.icon;
          const active = section === n.key;
          return (
            <button
              key={n.key}
              onClick={() => setSection(n.key)}
              className={cn(
                'flex-1 h-7 rounded-md flex items-center justify-center gap-1 text-xs font-medium transition-colors',
                active
                  ? 'bg-white dark:bg-gray-800 text-primary-700 dark:text-primary-400 shadow-sm'
                  : 'text-gray-500 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-white/60 dark:hover:bg-gray-800/60',
              )}
              title={n.label}
            >
              <Icon className="h-3 w-3 flex-shrink-0" />
              <span className="whitespace-nowrap">{n.label}</span>
              {n.count > 0 && (
                <span className="text-[10px] text-gray-400 dark:text-gray-500">{n.count}</span>
              )}
            </button>
          );
        })}
        <button
          onClick={handleNewRequest}
          className="h-7 w-7 inline-flex items-center justify-center rounded-md hover:bg-white dark:hover:bg-gray-800 text-gray-500 dark:text-gray-400 hover:text-primary-600 flex-shrink-0"
          title="新建空白请求 (Ctrl+N)"
        >
          <Plus className="h-3.5 w-3.5" />
        </button>
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-auto p-2">
        {section === 'collections' && (
          <TreeCollectionsList
            items={collections}
            savedRequests={savedRequests}
            expanded={expandedCollections}
            onToggle={toggleExpand}
            baseUrl={baseUrl}
            onNew={() => setCollectionDialog(true)}
            onRemove={async (id: string) => {
              await removeCollection(id);
              toast.success('已删除集合');
            }}
          />
        )}
        {section === 'history' && (
          <HistoryList
            items={history}
            baseUrl={baseUrl}
            onClear={async () => {
              if (confirm('清空所有历史记录？')) {
                await clearHistory();
                toast.success('已清空历史');
              }
            }}
            onRemove={async (id: string) => {
              await removeHistory(id);
              toast.success('已删除');
            }}
          />
        )}
        {section === 'favorites' && (
          <FavoritesList
            items={favorites}
            baseUrl={baseUrl}
            onRemove={async (id: string) => {
              await removeFavorite(id);
              toast.success('已删除收藏');
            }}
          />
        )}
      </div>

      {/* 底部：环境切换 + 导入入口 */}
      <div className="px-2 py-2 border-t border-gray-200 dark:border-gray-800 space-y-1.5">
        <div className="relative">
          <button
            onClick={() => setEnvDropdown(!envDropdown)}
            className="w-full h-7 px-2 text-xs rounded flex items-center justify-between text-gray-600 dark:text-gray-400 hover:bg-white dark:hover:bg-gray-800"
          >
            <span className="truncate">
              {activeEnvId
                ? environments.find((e) => e.id === activeEnvId)?.name || '选择环境'
                : '选择环境'}
            </span>
            <span className="text-[10px] text-gray-400 ml-1">▼</span>
          </button>
          {envDropdown && (
            <div className="absolute bottom-full left-0 right-0 mb-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-20 max-h-40 overflow-auto">
              <button
                onClick={() => {
                  setActiveEnv(null);
                  setEnvDropdown(false);
                }}
                className={cn(
                  'w-full text-left px-2 py-1.5 text-xs',
                  !activeEnvId && 'text-primary-600 bg-primary-50 dark:bg-primary-900/20',
                )}
              >
                无环境
              </button>
              {environments.map((e) => (
                <button
                  key={e.id}
                  onClick={() => {
                    setActiveEnv(e.id);
                    setEnvDropdown(false);
                  }}
                  className={cn(
                    'w-full text-left px-2 py-1.5 text-xs flex items-center justify-between',
                    activeEnvId === e.id && 'text-primary-600 bg-primary-50 dark:bg-primary-900/20',
                  )}
                >
                  <span>{e.name}</span>
                  <span className="flex items-center gap-0.5">
                    <button
                      onClick={(ev) => {
                        ev.stopPropagation();
                        setEditingEnv(e);
                        setEnvDialog(true);
                        setEnvDropdown(false);
                      }}
                      className="text-[10px] text-gray-400 hover:text-primary-600"
                    >
                      编辑
                    </button>
                    <button
                      onClick={(ev) => {
                        ev.stopPropagation();
                        removeEnvironment(e.id);
                        setEnvDropdown(false);
                      }}
                      className="text-[10px] text-gray-400 hover:text-red-500"
                    >
                      删除
                    </button>
                  </span>
                </button>
              ))}
              <div className="border-t border-gray-100 dark:border-gray-700">
                <button
                  onClick={() => {
                    setEditingEnv(null);
                    setEnvDialog(true);
                    setEnvDropdown(false);
                  }}
                  className="w-full text-left px-2 py-1.5 text-xs text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/20"
                >
                  + 新建环境
                </button>
              </div>
            </div>
          )}
        </div>
        <Link
          to="/import"
          className="w-full h-7 px-2 inline-flex items-center justify-center gap-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-white dark:hover:bg-gray-800 rounded"
        >
          <Upload className="h-3 w-3" /> 导入 cURL / JSON
        </Link>
      </div>

      {envDialog && <EnvironmentDialog env={editingEnv} onClose={() => setEnvDialog(false)} />}
      {collectionDialog && (
        <CollectionDialog
          onClose={() => setCollectionDialog(false)}
          onSave={handleCreateCollection}
        />
      )}
    </aside>
  );
}

/* ========== 树形集合列表 ========== */
function TreeCollectionsList({
  items,
  savedRequests,
  expanded,
  onToggle,
  baseUrl,
  onNew,
  onRemove,
}: {
  items: Collection[];
  savedRequests: Request[];
  expanded: Set<string>;
  onToggle: (id: string) => void;
  baseUrl: string;
  onNew: () => void;
  onRemove: (id: string) => void;
}) {
  const navigate = useNavigate();
  return (
    <div className="space-y-0.5">
      <button
        onClick={onNew}
        className="w-full h-7 px-2 text-xs text-primary-700 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 rounded inline-flex items-center gap-1 justify-center mb-1"
      >
        <Plus className="h-3 w-3" /> 新建集合
      </button>
      {items.length === 0 ? (
        <EmptyState tip="暂无集合" />
      ) : (
        items.map((c) => {
          const children = savedRequests.filter((r) => c.request_ids.includes(r.id));
          const isOpen = expanded.has(c.id);
          return (
            <div key={c.id}>
              {/* 一级：集合名 */}
              <div className="group flex items-center rounded hover:bg-white dark:hover:bg-gray-800 cursor-pointer">
                <button
                  onClick={() => onToggle(c.id)}
                  className="p-0.5 text-gray-400 hover:text-gray-600 flex-shrink-0"
                >
                  {isOpen ? (
                    <ChevronDown className="h-3 w-3" />
                  ) : (
                    <ChevronRight className="h-3 w-3" />
                  )}
                </button>
                <Link
                  to={`/collection/${c.id}`}
                  className="flex-1 min-w-0 flex items-center justify-between px-1 py-1.5"
                >
                  <div className="min-w-0 flex-1">
                    <div className="text-xs font-medium text-gray-700 dark:text-gray-300 truncate">
                      {c.name}
                    </div>
                  </div>
                  <span className="text-[10px] text-gray-400 dark:text-gray-500 flex-shrink-0">
                    {children.length}
                  </span>
                </Link>
                <button
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onRemove(c.id);
                  }}
                  className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-red-500 p-0.5 flex-shrink-0 mr-1"
                >
                  <Trash2 className="h-3 w-3" />
                </button>
              </div>
              {/* 二级：集合内请求 */}
              {isOpen && children.length > 0 && (
                <div className="ml-4 border-l border-gray-200 dark:border-gray-700 pl-2 space-y-0.5 mt-0.5">
                  {children.map((req) => (
                    <RequestMiniItem key={req.id} item={req} baseUrl={baseUrl} />
                  ))}
                </div>
              )}
            </div>
          );
        })
      )}
    </div>
  );
}

/* 集合内的二级请求项 */
function RequestMiniItem({ item, baseUrl }: { item: Request; baseUrl: string }) {
  const navigate = useNavigate();
  const location = useLocation();
  const statusMeta = REQUEST_STATUS_META[item.status as RequestStatus];
  const fullUrl = fullUrlDisplay(item.url, baseUrl);
  return (
    <div
      onClick={() => {
        if (location.pathname !== '/') navigate('/');
        window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: item }));
      }}
      className="group px-1.5 py-1 rounded hover:bg-white dark:hover:bg-gray-800 cursor-pointer flex items-center gap-1.5"
    >
      <span className="text-[9px] font-bold text-gray-400 dark:text-gray-500 flex-shrink-0 w-8 text-right">
        {item.method}
      </span>
      <div className="flex-1 min-w-0">
        <span className="text-xs text-gray-600 dark:text-gray-400 truncate block">{item.name}</span>
        {fullUrl && (
          <span className="text-[9px] text-gray-400 dark:text-gray-500 font-mono truncate block">
            {fullUrl}
          </span>
        )}
      </div>
      {statusMeta && (
        <span
          className={cn(
            'px-1 h-3.5 inline-flex items-center text-[8px] rounded whitespace-nowrap flex-shrink-0',
            statusMeta.color,
          )}
        >
          {statusMeta.label}
        </span>
      )}
    </div>
  );
}

/* ========== 收藏列表 ========== */
function FavoritesList({
  items,
  baseUrl,
  onRemove,
}: {
  items: any[];
  baseUrl: string;
  onRemove: (id: string) => void;
}) {
  if (items.length === 0) return <EmptyState tip="还没有收藏。点击 ⭐ 收藏一个请求。" />;
  return (
    <div className="space-y-1">
      {items.map((f) => (
        <RequestListItem
          key={f.id}
          item={f.request}
          baseUrl={baseUrl}
          right={
            <button
              onClick={(e) => {
                e.stopPropagation();
                onRemove(f.id);
              }}
              className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-red-500 p-0.5 flex-shrink-0"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          }
        />
      ))}
    </div>
  );
}

/* ========== 历史列表 ========== */
function HistoryList({
  items,
  baseUrl,
  onClear,
  onRemove,
}: {
  items: any[];
  baseUrl: string;
  onClear: () => void;
  onRemove: (id: string) => void;
}) {
  if (items.length === 0) return <EmptyState tip="暂无历史记录" />;
  return (
    <div className="space-y-1">
      <div className="flex justify-end mb-1">
        <button
          onClick={onClear}
          className="text-[10px] text-gray-400 dark:text-gray-500 hover:text-red-500"
        >
          清空
        </button>
      </div>
      {items.map((h) => (
        <RequestListItem
          key={h.id}
          item={h.request}
          baseUrl={baseUrl}
          subtitle={`${h.response.status} · ${h.response.time_ms}ms`}
          right={
            <div className="flex items-center gap-0.5">
              <span className="text-[10px] text-gray-400 dark:text-gray-500 flex-shrink-0">
                {formatDate(h.created_at)}
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  onRemove(h.id);
                }}
                className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-red-500 p-0.5 flex-shrink-0"
              >
                <Trash2 className="h-3 w-3" />
              </button>
            </div>
          }
        />
      ))}
    </div>
  );
}

/* ========== 通用请求列表项 ========== */
function RequestListItem({
  item,
  baseUrl,
  subtitle,
  right,
}: {
  item: any;
  baseUrl: string;
  subtitle?: string;
  right?: React.ReactNode;
}) {
  const navigate = useNavigate();
  const location = useLocation();
  const statusMeta = REQUEST_STATUS_META[item.status as RequestStatus];
  const displayUrl = fullUrlDisplay(item.url, baseUrl);
  return (
    <div
      onClick={() => {
        if (location.pathname !== '/') navigate('/');
        window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: item }));
      }}
      className="group px-2 py-1.5 rounded hover:bg-white dark:hover:bg-gray-800 cursor-pointer"
    >
      <div className="flex items-center justify-between gap-1">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5">
            <span className="px-1 h-4 inline-flex items-center text-[9px] font-bold rounded bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 flex-shrink-0">
              {item.method}
            </span>
            {statusMeta && (
              <span
                className={cn(
                  'px-1 h-4 inline-flex items-center text-[9px] rounded whitespace-nowrap flex-shrink-0',
                  statusMeta.color,
                )}
              >
                {statusMeta.label}
              </span>
            )}
            <span className="text-xs text-gray-700 dark:text-gray-300 truncate">
              {item.name || '(空)'}
            </span>
          </div>
          <div className="flex items-center gap-2 mt-0.5">
            {displayUrl && (
              <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono truncate">
                {displayUrl}
              </span>
            )}
            {subtitle && (
              <span className="text-[10px] text-gray-400 dark:text-gray-500 flex-shrink-0">
                {subtitle}
              </span>
            )}
          </div>
        </div>
        {right}
      </div>
    </div>
  );
}

function EmptyState({ tip }: { tip: string }) {
  return (
    <div className="px-3 py-8 text-center text-xs text-gray-400 dark:text-gray-500">{tip}</div>
  );
}
