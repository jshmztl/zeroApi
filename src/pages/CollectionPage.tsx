import * as React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Trash2, Edit2, Check, X } from 'lucide-react';
import { useDataStore } from '@/store/dataStore';
import { toast } from '@/components/ui/Toast';
import { REQUEST_STATUS_META } from '@/types';
import type { Request } from '@/types';

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
  } = useDataStore();
  const [editing, setEditing] = React.useState(false);
  const [editName, setEditName] = React.useState('');
  const [editDesc, setEditDesc] = React.useState('');

  React.useEffect(() => {
    loadCollections();
    loadSavedRequests(id);
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
          return (
            <div
              key={req.id}
              onClick={() => {
                window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: req }));
                nav('/');
              }}
              className="px-3 py-2 border border-gray-100 dark:border-gray-700 rounded-lg hover:border-primary-300 dark:hover:border-primary-600 cursor-pointer bg-white dark:bg-gray-800 flex items-center gap-2"
            >
              <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400 w-10 flex-shrink-0">
                {req.method}
              </span>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5">
                  <span className="text-sm text-gray-800 dark:text-gray-200">
                    {req.name || req.url || '(未命名)'}
                  </span>
                  {req.name && (
                    <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono truncate">
                      {req.url}
                    </span>
                  )}
                </div>
              </div>
              {statusMeta && (
                <span
                  className={`px-1.5 py-0.5 text-[9px] rounded whitespace-nowrap flex-shrink-0 ${statusMeta.color}`}
                >
                  {statusMeta.label}
                </span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
