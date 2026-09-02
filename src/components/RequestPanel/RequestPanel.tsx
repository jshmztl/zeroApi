import * as React from 'react';
import { Send, Star, X, TerminalSquare } from 'lucide-react';
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
import { HTTP_METHODS } from '@/types';
import type { Collection } from '@/types';
import { nanoid } from '@/lib/nanoid';
import { buildFullUrl, copyToClipboard } from '@/lib/formatter';
import { cn } from '@/lib/utils';

const METHOD_OPTIONS = HTTP_METHODS.map((m) => ({
  value: m,
  label: m,
  color:
    m === 'GET'
      ? '#43D6A0'
      : m === 'POST'
        ? '#5C9BEF'
        : m === 'PUT'
          ? '#F0A53C'
          : m === 'PATCH'
            ? '#A88CF0'
            : m === 'DELETE'
              ? '#F0605F'
              : '#96A0AC',
}));

type TabKey = 'params' | 'headers' | 'body' | 'auth';

export function RequestPanel() {
  const {
    request,
    setMethod,
    setUrl,
    setQuery,
    setHeaders,
    setBody,
    setAuth,
    loading,
    response,
    send,
    cancel,
  } = useRequestStore();
  const {
    loadHistory,
    loadFavorites,
    loadCollections,
    loadRequests,
    collections,
    environments,
    activeEnvId,
  } = useDataStore();
  const [tab, setTab] = React.useState<TabKey>('body');
  const [saveDropdown, setSaveDropdown] = React.useState(false);
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

  const enabledParamsCount = request.query.filter((p) => p.enabled && p.name).length;
  const enabledHeadersCount = request.headers.filter((h) => h.name).length;

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

  const copyCurl = async () => {
    if (!request.url.trim()) {
      toast.error('请求为空,无法生成 cURL');
      return;
    }
    try {
      const c = await tauri.exportCurl(request);
      const ok = await copyToClipboard(c);
      toast[ok ? 'success' : 'error'](ok ? '已复制 cURL 命令' : '复制失败');
    } catch (e: any) {
      toast.error('生成 cURL 失败: ' + String(e));
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
      await tauri.saveRequest({
        ...request,
        id: reqId,
        collection_id: collectionId,
      });
      await loadRequests();
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
        project_id: 'default',
        name,
        description,
        sort_order: 0,
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

  const fullUrl = buildFullUrl(request.url, request.query);

  return (
    <div className="z-panel flex flex-col overflow-hidden bg-card/70 dark:bg-gray-900/70">
      {/* 传输中：示波器描线扫过面板顶部 */}
      {loading && (
        <div className="relative h-[2px] overflow-hidden bg-primary-500/0">
          <div className="absolute top-0 bottom-0 w-1/2 bg-gradient-to-r from-transparent via-primary-400/80 to-transparent animate-trace" />
        </div>
      )}
      {/* 顶部：名称 + 方法 + URL + 操作按钮 */}
      <div className="px-4 py-3 border-b border-border flex items-center gap-2">
        {/* 请求名（必填） */}
        <Input
          value={request.name}
          onChange={(e) => useRequestStore.getState().setName(e.target.value)}
          placeholder="请求名（必填）"
          className="w-40 text-sm border-primary-200 dark:border-primary-900/50 focus:border-primary-500"
        />

        {/* 方法 */}
        <Select value={request.method} onChange={setMethod} options={METHOD_OPTIONS} />

        {/* URL 输入（带 base_url 前缀） */}
        <div className="flex-1 flex flex-col">
          <div className="flex items-center">
            {baseUrl && (
              <span className="h-9 px-2.5 text-xs font-mono bg-gray-100 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 border-r-0 rounded-l-lg flex items-center text-gray-500 dark:text-gray-400 whitespace-nowrap select-all cursor-default">
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
            <div className="text-[10px] font-mono text-gray-400 dark:text-gray-500 truncate mt-1 pl-0.5 select-all">
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

        {/* 复制 cURL */}
        <Button variant="outline" onClick={copyCurl} title="复制为 cURL 命令">
          <TerminalSquare className="h-3.5 w-3.5" />
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
            value={request.query}
            onChange={setQuery}
            keyPlaceholder="参数名"
            valuePlaceholder="参数值"
            bulkPaste
          />
        )}
        {tab === 'headers' && (
          <KeyValueEditor
            value={request.headers.map((h) => ({ name: h.name, value: h.value, enabled: true }))}
            onChange={(items) =>
              setHeaders(
                items
                  .filter((i) => i.name.trim())
                  .map((i) => ({ name: i.name, value: i.value })),
              )
            }
            keyPlaceholder="Header"
            valuePlaceholder="Value"
            presets={[
              { name: 'Content-Type', value: 'application/json' },
              { name: 'Accept', value: 'application/json' },
              { name: 'User-Agent', value: 'ZeroApi/2.0' },
            ]}
          />
        )}
        {tab === 'body' && (
          <BodyEditor value={request.body ?? { type: 'none' }} onChange={setBody} />
        )}
        {tab === 'auth' && (
          <AuthEditor value={request.auth ?? { type: 'none' }} onChange={setAuth} />
        )}
      </div>

      {/* 状态栏 */}
      {response && (
        <div className="px-4 py-2 border-t border-border bg-accent/40 dark:bg-accent/20 text-[11px] text-gray-500 dark:text-gray-400 flex items-center gap-3 font-mono">
          <span
            className={`font-mono font-semibold ${response.status >= 200 && response.status < 300 ? 'text-success' : response.status >= 400 ? 'text-danger' : 'text-gray-600 dark:text-gray-300'}`}
          >
            {response.status} {response.status_text}
          </span>
          <span className="text-muted-foreground">·</span>
          <span>{response.timing?.total_ms ?? 0} ms</span>
          <span className="text-muted-foreground">·</span>
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
