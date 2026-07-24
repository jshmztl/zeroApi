import * as React from 'react';
import { Send, Star, X } from 'lucide-react';
import { useRequestStore } from '@/store/requestStore';
import { useDataStore } from '@/store/dataStore';
import { Button } from '@/components/ui/Button';
import { Tabs } from '@/components/ui/Tabs';
import { Select } from '@/components/ui/Select';
import { Input } from '@/components/ui/Input';
import { KeyValueEditor } from '@/components/KeyValueEditor/KeyValueEditor';
import { BodyEditor } from '@/components/RequestPanel/BodyEditor';
import { AuthEditor } from '@/components/RequestPanel/AuthEditor';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { CollectionDialog } from '@/components/Sidebar/CollectionDialog';
import { HTTP_METHODS, REQUEST_STATUS_META } from '@/types';
import type { RequestStatus, Collection } from '@/types';
import { nanoid } from '@/lib/nanoid';
import { buildFullUrl } from '@/lib/formatter';
import { cn } from '@/lib/utils';

const METHOD_OPTIONS = HTTP_METHODS.map((m) => ({
  value: m,
  label: m,
  color:
    m === 'GET'
      ? '#10B981'
      : m === 'POST'
        ? '#3B82F6'
        : m === 'PUT'
          ? '#F59E0B'
          : m === 'PATCH'
            ? '#8B5CF6'
            : m === 'DELETE'
              ? '#EF4444'
              : '#6B7280',
}));

const STATUS_OPTIONS = (Object.keys(REQUEST_STATUS_META) as RequestStatus[]).map((s) => ({
  value: s,
  label: REQUEST_STATUS_META[s].label,
  color: REQUEST_STATUS_META[s].color,
}));

type TabKey = 'params' | 'headers' | 'body' | 'auth';

export function RequestPanel() {
  const {
    request,
    setMethod,
    setUrl,
    setParams,
    setHeaders,
    setBody,
    setAuth,
    setStatus,
    loading,
    response,
    send,
    cancel,
  } = useRequestStore();
  const {
    loadHistory,
    loadFavorites,
    loadCollections,
    loadSavedRequests,
    collections,
    environments,
    activeEnvId,
    attachToCollection,
  } = useDataStore();
  const [tab, setTab] = React.useState<TabKey>('body');
  const [saveDropdown, setSaveDropdown] = React.useState(false);
  const [statusDropdown, setStatusDropdown] = React.useState(false);
  const [newCollectionDialog, setNewCollectionDialog] = React.useState(false);

  // 加载已保存/历史请求时，默认切换到 Body tab
  React.useEffect(() => {
    const handler = () => setTab('body');
    window.addEventListener('zeroapi:load-request', handler);
    return () => window.removeEventListener('zeroapi:load-request', handler);
  }, []);

  React.useEffect(() => {
    loadCollections();
  }, [loadCollections]);

  const activeEnv = environments.find((e) => e.id === activeEnvId);
  const baseUrl = activeEnv?.base_url || '';

  const enabledParamsCount = request.params.filter((p) => p.enabled && p.key).length;
  const enabledHeadersCount = request.headers.filter((h) => h.enabled && h.key).length;

  const currentStatusMeta = REQUEST_STATUS_META[request.status];

  const handleSend = async () => {
    if (!request.url.trim()) {
      toast.error('请输入 URL');
      return;
    }
    try {
      await send();
      await loadHistory();
    } catch (e: any) {
      const msg = typeof e === 'string' ? e : e?.message || String(e);
      if (!msg.includes('请求已取消'))
        toast.error(msg.length > 200 ? msg.slice(0, 200) + '...' : msg);
    }
  };

  const handleCancel = async () => {
    await cancel();
  };

  const favorite = async () => {
    if (!request.url.trim()) {
      toast.error('请求为空,无法收藏');
      return;
    }
    try {
      await tauri.addFavorite({ ...request, id: request.id || nanoid() });
      await loadFavorites();
      toast.success('已加入收藏');
    } catch (e: any) {
      toast.error('收藏失败: ' + String(e));
    }
  };

  const handleSave = async (collectionId: string) => {
    if (!request.url.trim()) {
      toast.error('请求为空,无法保存');
      return;
    }
    if (!request.name.trim()) {
      toast.error('请先填写接口名称再保存');
      setSaveDropdown(false);
      return;
    }
    try {
      const reqId = request.id || nanoid();
      const lastResponse = response && response.status >= 200 && response.status < 400 ? response : null;
      await tauri.saveSavedRequest({ ...request, id: reqId, last_response: lastResponse }, collectionId);
      await attachToCollection(collectionId, reqId);
      await loadFavorites();
      await loadSavedRequests();
      toast.success('已保存到集合');
    } catch (e: any) {
      toast.error('保存失败: ' + String(e));
    }
    setSaveDropdown(false);
  };

  const handleNewCollectionFromSave = async (name: string, description: string) => {
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
      toast.success('集合已创建，请再次保存');
      setNewCollectionDialog(false);
    } catch (e: any) {
      toast.error('创建失败: ' + String(e));
    }
  };

  const fullUrl = buildFullUrl(request.url, request.params);

  return (
    <div className="flex flex-col bg-white dark:bg-gray-900">
      {/* 顶部：名称 + 状态 + 方法 + URL + 操作按钮 */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-800 flex items-center gap-2">
        {/* 请求名（必填） */}
        <Input
          value={request.name}
          onChange={(e) => useRequestStore.getState().setName(e.target.value)}
          placeholder="请求名（必填）"
          className="w-40 text-sm border-primary-300 focus:border-primary-500"
        />

        {/* 状态选择（带颜色） */}
        <div className="relative">
          <button
            onClick={() => setStatusDropdown(!statusDropdown)}
            className={cn(
              'h-8 px-2 text-xs rounded border focus:outline-none focus:ring-2 focus:ring-primary-500/30 flex items-center gap-1 whitespace-nowrap',
              currentStatusMeta.color,
              'border-gray-200 dark:border-gray-700',
            )}
            title="接口状态"
          >
            <span>{currentStatusMeta.label}</span>
            <span className="text-[10px] ml-0.5">▼</span>
          </button>
          {statusDropdown && (
            <div className="absolute left-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 min-w-[80px]">
              {STATUS_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  onClick={() => {
                    setStatus(opt.value);
                    setStatusDropdown(false);
                  }}
                  className={cn(
                    'w-full text-left px-2 py-1.5 text-xs',
                    opt.color,
                    'hover:opacity-80',
                  )}
                >
                  {opt.label}
                </button>
              ))}
            </div>
          )}
        </div>

        {/* 方法 */}
        <Select value={request.method} onChange={setMethod} options={METHOD_OPTIONS} />

        {/* URL 输入（带 base_url 前缀） */}
        <div className="flex-1 flex flex-col">
          <div className="flex items-center">
            {baseUrl && (
              <span className="h-8 px-2 text-xs font-mono bg-gray-100 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 border-r-0 rounded-l flex items-center text-gray-500 dark:text-gray-400 whitespace-nowrap select-all cursor-default">
                {baseUrl.replace(/\/$/, '')}
              </span>
            )}
            <Input
              value={request.url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder={baseUrl ? '/api/users' : 'https://api.example.com/users'}
              className={cn('flex-1 font-mono text-sm', baseUrl && 'rounded-l-none')}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && (e.ctrlKey || e.metaKey))
                  loading ? handleCancel() : handleSend();
              }}
            />
          </div>
          {fullUrl !== request.url && fullUrl && (
            <div className="text-[10px] font-mono text-gray-400 dark:text-gray-500 truncate mt-0.5 pl-0.5 select-all">
              {baseUrl ? baseUrl.replace(/\/$/, '') + fullUrl : fullUrl}
            </div>
          )}
        </div>

        {/* 发送/取消 */}
        <Button
          id="zeroapi-send-btn"
          variant={loading ? 'danger' : 'primary'}
          onClick={loading ? handleCancel : handleSend}
        >
          {loading ? <X className="h-3.5 w-3.5" /> : <Send className="h-3.5 w-3.5" />}
          {loading ? '取消' : '发送'}
        </Button>

        {/* 收藏 */}
        <Button id="zeroapi-fav-btn" variant="outline" onClick={favorite} title="Ctrl+S 收藏">
          <Star className="h-3.5 w-3.5" />
        </Button>

        {/* 保存 */}
        <div className="relative">
          <Button
            variant="outline"
            onClick={(e) => {
              e.stopPropagation();
              setSaveDropdown(!saveDropdown);
            }}
            title="保存到集合"
          >
            保存
          </Button>
          {saveDropdown && (
            <div className="absolute right-0 top-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-md shadow-lg z-50 min-w-[160px] max-h-48 overflow-auto">
              {collections.length === 0 ? (
                <div className="px-3 py-2 text-xs text-gray-400">暂无集合，请先创建</div>
              ) : (
                collections.map((c) => (
                  <button
                    key={c.id}
                    onClick={() => handleSave(c.id)}
                    className="w-full text-left px-3 py-1.5 text-xs text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                  >
                    {c.name}
                    <span className="text-[10px] text-gray-400 ml-1">({c.request_ids.length})</span>
                  </button>
                ))
              )}
              <div className="border-t border-gray-100 dark:border-gray-700">
                <button
                  onClick={() => {
                    setSaveDropdown(false);
                    setNewCollectionDialog(true);
                  }}
                  className="w-full text-left px-3 py-1.5 text-xs text-primary-600 hover:bg-primary-50 dark:hover:bg-primary-900/20"
                >
                  + 新建集合
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Tab 切换 */}
      <Tabs
        value={tab}
        onChange={(v) => setTab(v as TabKey)}
        items={[
          { value: 'headers', label: 'Headers', badge: enabledHeadersCount || undefined },
          { value: 'params', label: 'Query', badge: enabledParamsCount || undefined },
          { value: 'body', label: 'Body' },
          { value: 'auth', label: 'Auth' },
        ]}
        className="px-4"
      />

      {/* Tab 内容 */}
      <div className="px-4 py-3 max-h-[42vh] overflow-auto">
        {tab === 'params' && (
          <KeyValueEditor
            value={request.params}
            onChange={setParams}
            keyPlaceholder="参数名"
            valuePlaceholder="参数值"
            bulkPaste
            showDescription
          />
        )}
        {tab === 'headers' && (
          <KeyValueEditor
            value={request.headers}
            onChange={setHeaders}
            keyPlaceholder="Header"
            valuePlaceholder="Value"
            presets={[
              { key: 'Content-Type', value: 'application/json' },
              { key: 'Accept', value: 'application/json' },
              { key: 'User-Agent', value: 'ZeroApi/1.0' },
            ]}
          />
        )}
        {tab === 'body' && <BodyEditor value={request.body} onChange={setBody} />}
        {tab === 'auth' && <AuthEditor value={request.auth} onChange={setAuth} />}
      </div>

      {/* 状态栏 */}
      {response && (
        <div className="px-4 py-1.5 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-800/50 text-[11px] text-gray-500 dark:text-gray-400 flex items-center gap-3">
          <span
            className={`font-mono font-semibold ${response.status >= 200 && response.status < 300 ? 'text-emerald-600 dark:text-emerald-400' : response.status >= 400 ? 'text-red-600 dark:text-red-400' : 'text-gray-600 dark:text-gray-400'}`}
          >
            {response.status} {response.status_text}
          </span>
          <span>·</span>
          <span>{response.time_ms} ms</span>
          <span>·</span>
          <span>{(response.size_bytes / 1024).toFixed(2)} KB</span>
        </div>
      )}
      {newCollectionDialog && (
        <CollectionDialog
          onClose={() => setNewCollectionDialog(false)}
          onSave={handleNewCollectionFromSave}
        />
      )}
    </div>
  );
}

/** 拼接完整请求路径（base_url + url），供列表展示用 */
export function fullUrlDisplay(url: string, baseUrl: string): string {
  if (!url) return '';
  if (url.startsWith('/') && baseUrl) {
    return baseUrl.replace(/\/$/, '') + url;
  }
  return url;
}
