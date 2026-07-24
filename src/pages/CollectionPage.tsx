import * as React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Trash2, Edit2, Check, X, ExternalLink } from 'lucide-react';
import { useDataStore } from '@/store/dataStore';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { REQUEST_STATUS_META } from '@/types';
import { fullUrlDisplay } from '@/components/RequestPanel/RequestPanel';

export function CollectionPage() {
  const { id } = useParams();
  const nav = useNavigate();
  const {
    collections,
    loadCollections,
    removeCollection,
    updateCollection,
    savedRequests,
    loadSavedRequests,
    detachFromCollection,
    attachToCollection,
    environments,
    activeEnvId,
  } = useDataStore();
  const [editing, setEditing] = React.useState(false);
  const [editName, setEditName] = React.useState('');
  const [editDesc, setEditDesc] = React.useState('');
  const [moveTarget, setMoveTarget] = React.useState<string | null>(null);
  const [moveReqId, setMoveReqId] = React.useState<string | null>(null);

  const activeEnv = environments.find((e) => e.id === activeEnvId);
  const baseUrl = activeEnv?.base_url || '';

  React.useEffect(() => {
    loadCollections();
    loadSavedRequests();
  }, [loadCollections, loadSavedRequests, id]);

  const col = collections.find((c) => c.id === id);

  if (!col) {
    return (
      <div className="p-8 text-center text-gray-500">
        <div>集合不存在</div>
        <button onClick={() => nav('/')} className="mt-2 text-primary-600">
          返回
        </button>
      </div>
    );
  }

  const items = savedRequests.filter((r) => col.request_ids.includes(r.id));
  const otherCollections = collections.filter((c) => c.id !== col.id);

  /* 重命名 */
  const handleRename = () => {
    setEditName(col.name);
    setEditDesc(col.description || '');
    setEditing(true);
  };
  const handleSaveRename = async () => {
    if (!editName.trim()) {
      toast.error('集合名不能为空');
      return;
    }
    try {
      await updateCollection(col.id, { name: editName.trim(), description: editDesc.trim() });
      toast.success('集合已更新');
      setEditing(false);
    } catch (e: any) {
      toast.error('更新失败: ' + String(e));
    }
  };

  /* 删除集合 */
  const handleDelete = async () => {
    if (!confirm(`确定删除集合「${col.name}」？此操作不可恢复。`)) return;
    try {
      await removeCollection(col.id);
      toast.success('集合已删除');
      nav('/');
    } catch (e: any) {
      toast.error('删除失败: ' + String(e));
    }
  };

  /* 从集合移除 */
  const handleRemoveRequest = async (reqId: string) => {
    try {
      await detachFromCollection(col.id, reqId);
      await loadSavedRequests();
      toast.success('已从集合移除');
    } catch (e: any) {
      toast.error('移除失败: ' + String(e));
    }
  };

  /* 迁移到其他集合 */
  const handleMoveRequest = async (reqId: string, targetId: string) => {
    try {
      await detachFromCollection(col.id, reqId);
      await attachToCollection(targetId, reqId);
      await loadSavedRequests();
      toast.success('已迁移');
      setMoveReqId(null);
      setMoveTarget(null);
    } catch (e: any) {
      toast.error('迁移失败: ' + String(e));
    }
  };

  return (
    <div className="p-6 max-w-3xl mx-auto">
      <button
        onClick={() => nav('/')}
        className="text-sm text-gray-500 hover:text-gray-900 dark:hover:text-gray-200 flex items-center gap-1 mb-3"
      >
        <ArrowLeft className="h-3 w-3" /> 返回
      </button>

      {/* 集合标题区 */}
      <div className="flex items-center justify-between mb-6">
        {editing ? (
          <div className="flex items-center gap-2 flex-1">
            <input
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              className="text-xl font-semibold border-b border-gray-300 dark:border-gray-600 bg-transparent px-1 py-0.5 focus:outline-none focus:border-primary-500 w-48"
              autoFocus
            />
            <input
              value={editDesc}
              onChange={(e) => setEditDesc(e.target.value)}
              className="text-sm border-b border-gray-200 dark:border-gray-600 bg-transparent px-1 py-0.5 focus:outline-none focus:border-primary-500 flex-1 max-w-xs"
              placeholder="描述"
            />
            <button
              onClick={handleSaveRename}
              className="text-emerald-600 hover:text-emerald-700 p-1"
            >
              <Check className="h-4 w-4" />
            </button>
            <button
              onClick={() => setEditing(false)}
              className="text-gray-400 hover:text-gray-600 p-1"
            >
              <X className="h-4 w-4" />
            </button>
          </div>
        ) : (
          <>
            <div>
              <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100">{col.name}</h1>
              <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                {col.description || '无描述'} · {items.length} 个请求
              </p>
            </div>
            <div className="flex items-center gap-1">
              <button
                onClick={handleRename}
                className="text-gray-400 hover:text-primary-600 p-1.5 rounded"
                title="编辑集合"
              >
                <Edit2 className="h-3.5 w-3.5" />
              </button>
              <button
                onClick={handleDelete}
                className="text-gray-400 hover:text-red-500 p-1.5 rounded"
                title="删除集合"
              >
                <Trash2 className="h-3.5 w-3.5" />
              </button>
            </div>
          </>
        )}
      </div>

      {/* 请求列表 */}
      <div className="space-y-1.5">
        {items.length === 0 && (
          <div className="text-center py-12 text-gray-400 text-sm">这个集合里还没有请求</div>
        )}
        {items.map((req) => {
          const statusMeta = (
            REQUEST_STATUS_META as Record<string, { label: string; color: string }>
          )[req.status];
          const displayUrl = fullUrlDisplay(req.url, baseUrl);
          return (
            <div
              key={req.id}
              className="px-3 py-2 border border-gray-100 dark:border-gray-700 rounded-lg hover:border-primary-300 dark:hover:border-primary-600 bg-white dark:bg-gray-800 flex items-center gap-2 group"
            >
              <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 w-10 flex-shrink-0">
                {req.method}
              </span>
              <div
                onClick={() => {
                  window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: req }));
                  nav('/');
                }}
                className="flex-1 min-w-0 cursor-pointer"
              >
                <span className="text-sm text-gray-800 dark:text-gray-200 block">
                  {req.name || '(未命名)'}
                </span>
                {displayUrl && (
                  <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono truncate block">
                    {displayUrl}
                  </span>
                )}
              </div>
              {statusMeta && (
                <span
                  className={`px-1.5 py-0.5 text-[9px] rounded whitespace-nowrap flex-shrink-0 ${statusMeta.color}`}
                >
                  {statusMeta.label}
                </span>
              )}
              {/* 操作按钮 */}
              <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 flex-shrink-0">
                {/* 迁移 */}
                <div className="relative">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setMoveReqId(moveReqId === req.id ? null : req.id);
                    }}
                    className="p-1 text-gray-400 hover:text-primary-600 rounded"
                    title="迁移到其他集合"
                  >
                    <ExternalLink className="h-3 w-3" />
                  </button>
                  {moveReqId === req.id && (
                    <div className="absolute right-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 min-w-[140px] max-h-36 overflow-auto">
                      <div className="px-2 py-1 text-[10px] text-gray-400 border-b border-gray-100 dark:border-gray-700">
                        迁移到：
                      </div>
                      {otherCollections.length === 0 ? (
                        <div className="px-2 py-1.5 text-xs text-gray-400">暂无其他集合</div>
                      ) : (
                        otherCollections.map((c) => (
                          <button
                            key={c.id}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleMoveRequest(req.id, c.id);
                            }}
                            className="w-full text-left px-2 py-1 text-xs text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                          >
                            {c.name}
                          </button>
                        ))
                      )}
                    </div>
                  )}
                </div>
                {/* 移除 */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    if (confirm('从集合中移除该接口？')) handleRemoveRequest(req.id);
                  }}
                  className="p-1 text-gray-400 hover:text-red-500 rounded"
                  title="从集合移除"
                >
                  <Trash2 className="h-3 w-3" />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
