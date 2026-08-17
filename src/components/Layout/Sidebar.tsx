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
  RotateCw,
  Activity,
  Globe,
} from 'lucide-react';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { useDataStore } from '@/store/dataStore';
import { useRequestStore } from '@/store/requestStore';
import { cn, formatDate } from '@/lib/utils';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { EnvironmentDialog } from '@/components/Sidebar/EnvironmentDialog';
import { CollectionDialog } from '@/components/Sidebar/CollectionDialog';
import { DiffModal } from '@/components/ResponsePanel/DiffModal';
import { nanoid } from '@/lib/nanoid';
import type { Collection, Request } from '@/types';
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
    requests,
    loadRequests,
    activeEnvId,
    setActiveEnv,
  } = useDataStore();
  const navigate = useNavigate();
  const location = useLocation();

  const [section, setSection] = React.useState<Section>('collections');
  const [envDialog, setEnvDialog] = React.useState(false);
  const [editingEnv, setEditingEnv] = React.useState<any | null>(null);
  const [collectionDialog, setCollectionDialog] = React.useState(false);
  const [envDropdown, setEnvDropdown] = React.useState(false);
  const [expandedCollections, setExpandedCollections] = React.useState<Set<string>>(new Set());
  // Response Diff 状态
  const [diffSelecting, setDiffSelecting] = React.useState(false);
  const [diffSelected, setDiffSelected] = React.useState<string[]>([]);
  const [diffTarget, setDiffTarget] = React.useState<{ a: any; b: any } | null>(null);

  React.useEffect(() => {
    loadFavorites();
    loadHistory();
    loadCollections();
    loadEnvironments();
    loadRequests();
  }, [loadFavorites, loadHistory, loadCollections, loadEnvironments, loadRequests]);

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
        project_id: 'default',
        name,
        description,
        sort_order: 0,
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
    <aside className="w-64 border-r border-border bg-card/60 dark:bg-gray-950/60 backdrop-blur-xl flex flex-col overflow-hidden">
      {/* 导航标签 + 新建 */}
      <div className="px-2.5 py-2.5 flex items-center gap-1 border-b border-border">
        {navItems.map((n) => {
          const Icon = n.icon;
          const active = section === n.key;
          return (
            <button
              key={n.key}
              onClick={() => setSection(n.key)}
              className={cn(
                'flex-1 h-8 rounded-lg flex items-center justify-center gap-1 text-xs font-medium transition-all duration-150',
                active
                  ? 'bg-white dark:bg-gray-800 text-primary-600 dark:text-primary-400 shadow-soft'
                  : 'text-gray-500 dark:text-gray-500 hover:text-gray-700 dark:hover:text-gray-300 hover:bg-gray-100/70 dark:hover:bg-gray-800/50',
              )}
              title={n.label}
            >
              <Icon className="h-3.5 w-3.5 flex-shrink-0" />
              <span className="whitespace-nowrap">{n.label}</span>
              {n.count > 0 && (
                <span className={cn(
                  'text-[10px] px-1 rounded-full',
                  active ? 'bg-primary-50 dark:bg-primary-900/30 text-primary-600' : 'text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-800',
                )}>
                  {n.count}
                </span>
              )}
            </button>
          );
        })}
        <button
          onClick={handleNewRequest}
          className="h-8 w-8 inline-flex items-center justify-center rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-500 dark:text-gray-400 hover:text-primary-600 transition-all flex-shrink-0"
          title="新建空白请求 (Ctrl+N)"
        >
          <Plus className="h-4 w-4" />
        </button>
      </div>

      {/* 内容区 */}
      <div className="flex-1 overflow-auto p-2">
        {section === 'collections' && (
          <TreeCollectionsList
            items={collections}
            requests={requests}
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
            selecting={diffSelecting}
            selectedIds={diffSelected}
            onToggleSelect={(id) => {
              setDiffSelected((prev) =>
                prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id],
              );
            }}
            onStartDiff={() => {
              setDiffSelecting(true);
              setDiffSelected([]);
            }}
            onCancelDiff={() => {
              setDiffSelecting(false);
              setDiffSelected([]);
            }}
            onRunDiff={() => {
              if (diffSelected.length !== 2) return;
              const [idA, idB] = diffSelected;
              const a = history.find((h) => h.execution.id === idA);
              const b = history.find((h) => h.execution.id === idB);
              if (a && b) {
                setDiffTarget({
                  a: { exec: a, title: `${a.name} · ${a.execution.status_code ?? '-'}` },
                  b: { exec: b, title: `${b.name} · ${b.execution.status_code ?? '-'}` },
                });
              }
              setDiffSelecting(false);
              setDiffSelected([]);
            }}
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
      <div className="px-2 py-2 border-t border-border space-y-1.5">
        <div className="relative">
          <button
            onClick={() => setEnvDropdown(!envDropdown)}
            className={cn(
              "w-full h-8 px-2.5 text-xs rounded-lg flex items-center justify-between transition-colors",
              envDropdown
                ? "bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 shadow-soft"
                : "text-gray-600 dark:text-gray-400 hover:bg-white dark:hover:bg-gray-800/70",
            )}
          >
            <span className="truncate flex items-center gap-1.5">
              <Globe className="h-3 w-3 text-primary-500 flex-shrink-0" />
              {activeEnvId
                ? environments.find((e) => e.id === activeEnvId)?.name || '选择环境'
                : '选择环境'}
            </span>
            <ChevronDown className={cn("h-3 w-3 text-gray-400 transition-transform", envDropdown && "rotate-180")} />
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
          className="w-full h-8 px-2.5 inline-flex items-center justify-center gap-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-white dark:hover:bg-gray-800 rounded-lg transition-colors"
        >
          <Upload className="h-3 w-3" /> 导入 cURL / JSON
        </Link>
        <Link
          to="/diagnostics"
          className="w-full h-8 px-2.5 inline-flex items-center justify-center gap-1 text-xs text-gray-600 dark:text-gray-400 hover:bg-white dark:hover:bg-gray-800 rounded-lg transition-colors"
        >
          <Activity className="h-3 w-3" /> 网络诊断
        </Link>
      </div>

      {envDialog && <EnvironmentDialog env={editingEnv} onClose={() => setEnvDialog(false)} />}
      {collectionDialog && (
        <CollectionDialog
          onClose={() => setCollectionDialog(false)}
          onSave={handleCreateCollection}
        />
      )}
      {diffTarget && (
        <DiffModal
          leftText={diffTarget.a.exec.execution.response?.body.type === 'text' ? diffTarget.a.exec.execution.response.body.text : '(二进制)'}
          rightText={diffTarget.b.exec.execution.response?.body.type === 'text' ? diffTarget.b.exec.execution.response.body.text : '(二进制)'}
          leftTitle={diffTarget.a.title}
          rightTitle={diffTarget.b.title}
          onClose={() => setDiffTarget(null)}
        />
      )}
    </aside>
  );
}

/* ========== 树形集合列表（Collection → Request） ========== */
function TreeCollectionsList({
  items,
  requests,
  expanded,
  onToggle,
  baseUrl,
  onNew,
  onRemove,
}: {
  items: Collection[];
  requests: Request[];
  expanded: Set<string>;
  onToggle: (id: string) => void;
  baseUrl: string;
  onNew: () => void;
  onRemove: (id: string) => void;
}) {
  return (
    <div className="space-y-0.5">
      <button
        onClick={onNew}
        className="w-full h-8 px-2 text-xs text-primary-600 dark:text-primary-400 hover:bg-primary-50 dark:hover:bg-primary-900/20 rounded-lg inline-flex items-center gap-1 justify-center mb-1 border border-dashed border-primary-200 dark:border-primary-800/60 hover:border-primary-400 transition-colors"
      >
        <Plus className="h-3.5 w-3.5" /> 新建集合
      </button>
      {items.length === 0 ? (
        <EmptyState tip="暂无集合" />
      ) : (
        items.map((c) => {
          const children = requests.filter((r) => r.collection_id === c.id);
          const isOpen = expanded.has(c.id);
          return (
            <div key={c.id}>
              {/* 一级：集合名 */}
              <div className="group flex items-center rounded-lg hover:bg-gray-100/70 dark:hover:bg-gray-800/60 transition-colors">
                <button
                  onClick={() => onToggle(c.id)}
                  className="p-1 text-gray-400 hover:text-gray-600 flex-shrink-0"
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
                  <span className="text-[10px] px-1.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-400 dark:text-gray-500 flex-shrink-0">
                    {children.length}
                  </span>
                </Link>
                <button
                  onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onRemove(c.id);
                  }}
                  className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-red-500 p-1 flex-shrink-0 mr-1 transition-opacity"
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

/* ========== 历史列表（基于 RequestExecution） ========== */
function HistoryList({
  items,
  baseUrl,
  selecting,
  selectedIds,
  onToggleSelect,
  onStartDiff,
  onCancelDiff,
  onRunDiff,
  onClear,
  onRemove,
}: {
  items: any[];
  baseUrl: string;
  selecting: boolean;
  selectedIds: string[];
  onToggleSelect: (id: string) => void;
  onStartDiff: () => void;
  onCancelDiff: () => void;
  onRunDiff: () => void;
  onClear: () => void;
  onRemove: (id: string) => void;
}) {
  const navigate = useNavigate();
  const location = useLocation();
  if (items.length === 0) return <EmptyState tip="暂无历史记录" />;
  return (
    <div className="space-y-1">
      <div className="flex justify-between items-center mb-1">
        {selecting ? (
          <div className="flex items-center gap-1">
            <span className="text-[10px] text-gray-400">
              已选 {selectedIds.length}/2（选择两条进行对比）
            </span>
            <button
              onClick={onRunDiff}
              disabled={selectedIds.length !== 2}
              className="text-[10px] px-1.5 py-0.5 rounded bg-primary-50 text-primary-700 hover:bg-primary-100 disabled:opacity-40 disabled:cursor-not-allowed"
            >
              对比
            </button>
            <button
              onClick={onCancelDiff}
              className="text-[10px] text-gray-400 hover:text-gray-600"
            >
              取消
            </button>
          </div>
        ) : (
          <div className="flex justify-end flex-1">
            <button
              onClick={onStartDiff}
              className="text-[10px] text-gray-400 dark:text-gray-500 hover:text-primary-600"
              title="选择两条历史响应进行对比"
            >
              对比
            </button>
            <button
              onClick={onClear}
              className="text-[10px] text-gray-400 dark:text-gray-500 hover:text-red-500 ml-2"
            >
              清空
            </button>
          </div>
        )}
      </div>
      {items.map((h) => {
        const exec = h.execution;
        const status = exec.status_code;
        const isSelected = selectedIds.includes(exec.id);
        return (
          <div
            key={exec.id}
            onClick={
              selecting
                ? () => onToggleSelect(exec.id)
                : async () => {
                    // 加载该次执行对应的请求（Replay 的基础）
                    try {
                      const req = await tauri.getRequest(exec.request_id);
                      if (req) {
                        if (location.pathname !== '/') navigate('/');
                        window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: req }));
                      }
                    } catch (e) {
                      console.error(e);
                    }
                  }
            }
            className={`group px-2 py-1.5 rounded cursor-pointer ${
              isSelected
                ? 'bg-primary-50 dark:bg-primary-900/20 ring-1 ring-primary-300 dark:ring-primary-700'
                : 'hover:bg-white dark:hover:bg-gray-800'
            }`}
          >
            <div className="flex items-center justify-between gap-1">
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                  {selecting && (
                    <input
                      type="checkbox"
                      checked={isSelected}
                      onChange={() => onToggleSelect(exec.id)}
                      onClick={(e) => e.stopPropagation()}
                      className="rounded text-primary-500 flex-shrink-0"
                    />
                  )}
                  <span className="px-1 h-4 inline-flex items-center text-[9px] font-bold rounded bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 flex-shrink-0">
                    {h.method}
                  </span>
                  {status != null && (
                    <span
                      className={cn(
                        'px-1 h-4 inline-flex items-center text-[9px] rounded flex-shrink-0',
                        status >= 200 && status < 300
                          ? 'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/40 dark:text-emerald-400'
                          : status >= 400
                            ? 'bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-400'
                            : 'bg-gray-100 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
                      )}
                    >
                      {status}
                    </span>
                  )}
                  <span className="text-xs text-gray-700 dark:text-gray-300 truncate">
                    {h.name || '(未命名)'}
                  </span>
                </div>
                <div className="flex items-center gap-2 mt-0.5">
                  {h.url && (
                    <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono truncate">
                      {fullUrlDisplay(h.url, baseUrl)}
                    </span>
                  )}
                  <span className="text-[10px] text-gray-400 dark:text-gray-500 flex-shrink-0">
                    {exec.duration_ms}ms
                  </span>
                </div>
              </div>
              {!selecting && (
                <div className="flex items-center gap-0.5 flex-shrink-0">
                  <span className="text-[10px] text-gray-400 dark:text-gray-500">
                    {formatDate(exec.started_at)}
                  </span>
                  {/* Replay（文档 §25）：恢复请求并重新执行，产生新 RequestExecution */}
                  <button
                    onClick={async (e) => {
                      e.stopPropagation();
                      try {
                        const req = await tauri.getRequest(exec.request_id);
                        if (!req) {
                          toast.error('原请求不存在，无法重放');
                          return;
                        }
                        if (location.pathname !== '/') navigate('/');
                        window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: req }));
                        // 等待 load 生效后自动发送
                        setTimeout(() => {
                          useRequestStore.getState().send();
                        }, 50);
                      } catch (err) {
                        toast.error('重放失败: ' + String(err));
                      }
                    }}
                    className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-primary-600 p-0.5"
                    title="重放此请求"
                  >
                    <RotateCw className="h-3 w-3" />
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      onRemove(exec.id);
                    }}
                    className="opacity-0 group-hover:opacity-100 text-gray-400 dark:text-gray-500 hover:text-red-500 p-0.5"
                  >
                    <Trash2 className="h-3 w-3" />
                  </button>
                </div>
              )}
            </div>
          </div>
        );
      })}
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
