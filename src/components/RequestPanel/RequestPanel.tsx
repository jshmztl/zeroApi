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
import { HTTP_METHODS, type KeyValue } from '@/types';
import { nanoid } from '@/lib/nanoid';
import { buildFullUrl } from '@/lib/formatter';

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
    loading,
    response,
    error,
    send,
    cancel,
  } = useRequestStore();
  const { loadHistory, loadFavorites } = useDataStore();
  const [tab, setTab] = React.useState<TabKey>('params');

  const enabledParamsCount = request.params.filter((p) => p.enabled && p.key).length;
  const enabledHeadersCount = request.headers.filter((h) => h.enabled && h.key).length;

  const handleSend = async () => {
    if (!request.url.trim()) {
      toast.error('请输入 URL');
      return;
    }
    try {
      await send();
      // 发送完成后刷新历史
      if (!getErrorIsCancel()) {
        await loadHistory();
      }
    } catch (e: any) {
      const msg = typeof e === 'string' ? e : e?.message || String(e);
      if (!msg.includes('请求已取消')) {
        toast.error(msg.length > 200 ? msg.slice(0, 200) + '...' : msg);
      }
    }
  };

  const getErrorIsCancel = () => {
    return false; // 取消错误已在 send 中静默处理
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

  const fullUrl = buildFullUrl(request.url, request.params);

  return (
    <div className="flex flex-col bg-white dark:bg-gray-900">
      {/* 顶部：名称 + 方法 + URL + 发送/取消 */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-800 flex items-center gap-2">
        <Input
          value={request.name}
          onChange={(e) => useRequestStore.getState().setName(e.target.value)}
          placeholder="请求名（可空）"
          className="w-36 text-sm"
        />
        <Select value={request.method} onChange={setMethod} options={METHOD_OPTIONS} />
        <Input
          value={request.url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder="https://api.example.com/users"
          className="flex-1 font-mono text-sm"
          onKeyDown={(e) => {
            if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
              loading ? handleCancel() : handleSend();
            }
          }}
        />
        <Button
          id="zeroapi-send-btn"
          variant={loading ? 'danger' : 'primary'}
          onClick={loading ? handleCancel : handleSend}
          loading={loading}
        >
          {loading ? <X className="h-3.5 w-3.5" /> : <Send className="h-3.5 w-3.5" />}
          {loading ? '取消' : '发送'}
        </Button>
        <Button id="zeroapi-fav-btn" variant="outline" onClick={favorite} title="Ctrl+S 收藏">
          <Star className="h-3.5 w-3.5" />
        </Button>
      </div>

      {/* Tab 切换 */}
      <Tabs
        value={tab}
        onChange={(v) => setTab(v as TabKey)}
        items={[
          { value: 'params', label: 'Params', badge: enabledParamsCount || undefined },
          { value: 'headers', label: 'Headers', badge: enabledHeadersCount || undefined },
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

      {/* 状态栏：最后响应 */}
      {response && (
        <div className="px-4 py-1.5 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-800/50 text-[11px] text-gray-500 dark:text-gray-400 flex items-center gap-3">
          <span
            className={`font-mono font-semibold ${response.status >= 200 && response.status < 300 ? 'text-emerald-600 dark:text-emerald-400' : response.status >= 400 ? 'text-red-600 dark:text-red-400' : 'text-gray-600 dark:text-gray-400'}`}
          >
            {response.status} {response.status_text}
          </span>
          <span>·</span>
          <span>⚡ {response.time_ms} ms</span>
          <span>·</span>
          <span>📦 {(response.size_bytes / 1024).toFixed(2)} KB</span>
        </div>
      )}
    </div>
  );
}
