import * as React from 'react';
import { Copy, Download, AlertCircle, Clock, Package, Globe } from 'lucide-react';
import { useRequestStore } from '@/store/requestStore';
import { Tabs } from '@/components/ui/Tabs';
import { Button } from '@/components/ui/Button';
import { CodeEditor } from '@/components/CodeEditor/CodeEditor';
import {
  tryPretty,
  languageFromContentType,
  tryDetectContentType,
  copyToClipboard,
  downloadText,
} from '@/lib/formatter';
import type { PrettyLang } from '@/lib/formatter';
import { formatBytes, formatDuration, statusColor } from '@/lib/utils';
import { toast } from '@/components/ui/Toast';

type RTab = 'body' | 'headers' | 'cookies';

export function ResponsePanel() {
  const { response, error, loading, request } = useRequestStore();
  const [tab, setTab] = React.useState<RTab>('body');
  const [raw, setRaw] = React.useState(false);

  // ⚠️ 所有 useMemo 必须在条件返回之前调用（React Hooks 规则）
  const colorInfo = response ? statusColor(response.status) : { bg: '', text: '' };
  const contentType = response ? tryDetectContentType(response.headers) : '';
  const lang = response ? languageFromContentType(contentType) : 'text';

  const displayBody = React.useMemo(() => {
    if (!response) return '';
    if (raw) return response.body;
    const r = tryPretty(response.body, lang as PrettyLang);
    return r.ok ? r.text : response.body;
  }, [response, raw, lang]);

  const cookieEntries = React.useMemo(() => {
    if (!response) return [];
    const setCookie = response.headers['set-cookie'] || response.headers['Set-Cookie'];
    if (!setCookie) return [];
    return setCookie
      .split(/,(?=[^;]+=)/)
      .map((c) => c.trim())
      .filter(Boolean);
  }, [response]);

  const headerEntries = React.useMemo(() => {
    if (!response) return [];
    return Object.entries(response.headers).filter(([k]) => k.toLowerCase() !== 'set-cookie');
  }, [response]);

  if (error) {
    return (
      <div className="h-full flex items-center justify-center bg-red-50/30 dark:bg-red-950/20">
        <div className="text-center">
          <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-red-100 dark:bg-red-900/30 flex items-center justify-center">
            <AlertCircle className="h-6 w-6 text-red-500" />
          </div>
          <div className="text-sm font-medium text-red-700 dark:text-red-400">请求失败</div>
          <div className="text-xs text-red-500 dark:text-red-400/80 mt-1 max-w-md px-4 whitespace-pre-wrap">
            {error}
          </div>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="h-full flex items-center justify-center dark:bg-gray-900">
        <div className="text-center text-gray-400 dark:text-gray-500">
          <div className="w-8 h-8 mx-auto mb-2 border-2 border-primary-200 dark:border-primary-800 border-t-primary-500 rounded-full animate-spin" />
          <div className="text-xs">请求中...</div>
        </div>
      </div>
    );
  }

  if (!response) {
    return (
      <div className="h-full flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center text-gray-400 dark:text-gray-500 max-w-sm">
          <div className="text-5xl mb-3">🚀</div>
          <div className="text-sm font-medium text-gray-600 dark:text-gray-400">还没有响应</div>
          <div className="text-xs mt-1">
            输入 URL,点击「发送」或按{' '}
            <kbd className="px-1.5 py-0.5 bg-white dark:bg-gray-800 border dark:border-gray-700 rounded text-[10px]">
              Ctrl+Enter
            </kbd>
          </div>
          <div className="mt-3 text-[10px] text-gray-400 dark:text-gray-600">
            请求方法:{request.method} · 目标:{request.url || '(空)'}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col bg-white dark:bg-gray-900">
      {/* 状态栏 */}
      <div className="px-4 py-2 border-b border-gray-200 dark:border-gray-800 flex items-center gap-4 text-xs">
        <span
          className={`px-2 py-0.5 rounded font-mono font-semibold ${colorInfo.bg} ${colorInfo.text}`}
        >
          {response.status} {response.status_text}
        </span>
        <span className="inline-flex items-center gap-1 text-gray-600 dark:text-gray-400">
          <Clock className="h-3 w-3" />
          {formatDuration(response.time_ms)}
        </span>
        <span className="inline-flex items-center gap-1 text-gray-600 dark:text-gray-400">
          <Package className="h-3 w-3" />
          {formatBytes(response.size_bytes)}
        </span>
        <span className="inline-flex items-center gap-1 text-gray-500 dark:text-gray-500 text-[10px]">
          <Globe className="h-3 w-3" />
          {contentType || 'unknown'}
        </span>
        <div className="flex-1" />
        <Button
          variant="ghost"
          size="sm"
          onClick={async () => {
            const ok = await copyToClipboard(displayBody);
            toast[ok ? 'success' : 'error'](ok ? '已复制响应内容' : '复制失败');
          }}
        >
          <Copy className="h-3 w-3" /> 复制
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => downloadText(response.body, `response-${Date.now()}.txt`)}
        >
          <Download className="h-3 w-3" /> 下载
        </Button>
      </div>

      <Tabs
        value={tab}
        onChange={(v) => setTab(v as RTab)}
        items={[
          { value: 'body', label: 'Body' },
          { value: 'headers', label: 'Headers', badge: headerEntries.length },
          { value: 'cookies', label: 'Cookies', badge: cookieEntries.length },
        ]}
        className="px-4"
      />

      {/* 使用 overflow-hidden 确保高度链不断裂 */}
      <div className="flex-1 min-h-0 overflow-hidden">
        {tab === 'body' && (
          <div className="h-full flex flex-col">
            <div className="px-4 py-1.5 flex items-center gap-2 text-[10px]">
              {['json', 'xml', 'html', 'sql', 'css'].includes(lang) && (
                <>
                  <button
                    onClick={() => setRaw(!raw)}
                    className="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded text-gray-600 dark:text-gray-400"
                  >
                    {raw ? 'Pretty' : 'Raw'}
                  </button>
                  {!raw && (
                    <span className="text-gray-400 dark:text-gray-500">
                      {lang.toUpperCase()} 已美化
                    </span>
                  )}
                </>
              )}
            </div>
            <div className="flex-1 min-h-0 px-4 pb-2">
              {!response.body || response.body.trim().length === 0 ? (
                <div className="h-full flex items-center justify-center text-xs text-gray-400 dark:text-gray-500">
                  响应体为空
                </div>
              ) : (
                <CodeEditor
                  value={displayBody}
                  language={lang === 'json' && !raw ? 'json' : lang}
                  height="100%"
                  readOnly
                />
              )}
            </div>
          </div>
        )}

        {tab === 'headers' && (
          <div className="h-full overflow-auto px-4 py-2">
            <table className="w-full text-xs">
              <thead>
                <tr className="text-gray-500 dark:text-gray-400 text-left">
                  <th className="font-medium py-1 w-1/3">Name</th>
                  <th className="font-medium py-1">Value</th>
                </tr>
              </thead>
              <tbody>
                {headerEntries.map(([k, v]) => (
                  <tr key={k} className="border-t border-gray-100 dark:border-gray-800">
                    <td className="py-1.5 pr-3 font-mono text-gray-700 dark:text-gray-300 align-top">
                      {k}
                    </td>
                    <td className="py-1.5 font-mono text-gray-600 dark:text-gray-400 break-all">
                      {v}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {tab === 'cookies' && (
          <div className="h-full overflow-auto px-4 py-2">
            {cookieEntries.length === 0 ? (
              <div className="text-center py-6 text-xs text-gray-400 dark:text-gray-500">
                响应中没有 Set-Cookie
              </div>
            ) : (
              <div className="space-y-1.5">
                {cookieEntries.map((c, i) => {
                  const [pair, ...attrs] = c.split(';').map((s) => s.trim());
                  const [k, v] = pair.split('=');
                  return (
                    <div
                      key={i}
                      className="border border-gray-100 dark:border-gray-800 rounded p-2 bg-gray-50 dark:bg-gray-800"
                    >
                      <div className="font-mono text-xs">
                        <span className="text-primary-700 dark:text-primary-400 font-semibold">
                          {k}
                        </span>{' '}
                        = {v}
                      </div>
                      <div className="mt-1 text-[10px] text-gray-500 dark:text-gray-400 font-mono">
                        {attrs.join('; ')}
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
