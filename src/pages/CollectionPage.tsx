import * as React from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ArrowLeft, Trash2, Edit2, Check, X } from 'lucide-react';
import { useDataStore } from '@/store/dataStore';
import { toast } from '@/components/ui/Toast';
import { REQUEST_STATUS_META } from '@/types';

export function CollectionPage() {
  const { id } = useParams();
  const nav = useNavigate();
  const {
    collections,
    favorites,
    loadCollections,
    loadFavorites,
    removeCollection,
    updateCollection,
  } = useDataStore();
  const [editing, setEditing] = React.useState(false);
  const [editName, setEditName] = React.useState('');
  const [editDesc, setEditDesc] = React.useState('');

  React.useEffect(() => {
    loadCollections();
    loadFavorites();
  }, [loadCollections, loadFavorites]);

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

  const items = favorites.filter((f) => col.request_ids.includes(f.request.id || f.id));

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
        className="text-sm text-gray-500 hover:text-gray-900 flex items-center gap-1 mb-3"
      >
        <ArrowLeft className="h-3 w-3" /> 返回
      </button>

      {/* 集合标题区 */}
      <div className="flex items-center justify-between">
        {editing ? (
          <div className="flex items-center gap-2 flex-1">
            <input
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              placeholder="集合名"
              className="text-xl font-semibold border-b border-gray-300 dark:border-gray-600 bg-transparent px-1 py-0.5 focus:outline-none focus:border-primary-500 w-48"
              autoFocus
            />
            <input
              value={editDesc}
              onChange={(e) => setEditDesc(e.target.value)}
              placeholder="描述"
              className="text-sm border-b border-gray-200 dark:border-gray-600 bg-transparent px-1 py-0.5 focus:outline-none focus:border-primary-500 flex-1 max-w-xs"
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
              <h1 className="text-xl font-semibold">{col.name}</h1>
              <p className="text-xs text-gray-500 mt-1">
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
      <div className="mt-4 space-y-1.5">
        {items.length === 0 && (
          <div className="text-center py-12 text-gray-400 text-sm">这个集合里还没有请求</div>
        )}
        {items.map((f) => {
          const request = f.request;
          const statusMeta = (
            REQUEST_STATUS_META as Record<string, { label: string; color: string }>
          )[request.status];
          return (
            <div
              key={f.id}
              onClick={() => {
                window.dispatchEvent(new CustomEvent('zeroapi:load-request', { detail: request }));
                nav('/');
              }}
              className="px-3 py-2 border border-gray-100 rounded-lg hover:border-primary-300 cursor-pointer bg-white dark:bg-gray-800 dark:border-gray-700 dark:hover:border-primary-600"
            >
              <div className="flex items-center gap-2">
                <span className="text-[10px] font-bold text-primary-700 dark:text-primary-400">
                  {request.method}
                </span>
                {statusMeta && (
                  <span
                    className={`px-1 py-0 h-4 inline-flex items-center text-[9px] rounded ${statusMeta.color}`}
                  >
                    {statusMeta.label}
                  </span>
                )}
                <span className="text-sm text-gray-800 dark:text-gray-200">
                  {request.name || request.url}
                </span>
              </div>
              <div className="text-[10px] text-gray-400 font-mono mt-0.5 truncate">
                {request.url}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
