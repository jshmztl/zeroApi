import * as React from 'react';
import { X, GitCompareArrows } from 'lucide-react';
import * as jsondiffpatch from 'jsondiffpatch';
import * as htmlFormatter from 'jsondiffpatch/formatters/html';
import 'jsondiffpatch/formatters/styles/html.css';
import { Button } from '@/components/ui/Button';

interface Props {
  leftText: string;
  rightText: string;
  leftTitle: string;
  rightTitle: string;
  onClose: () => void;
}

/**
 * JSON 响应对比（文档 §26 Response Diff）
 * 第一版只做 JSON Diff；非 JSON 内容退化为逐行文本对比。
 */
export function DiffModal({ leftText, rightText, leftTitle, rightTitle, onClose }: Props) {
  const [html, setHtml] = React.useState('');
  const [mode, setMode] = React.useState<'json' | 'text'>('json');
  const [error, setError] = React.useState('');

  React.useEffect(() => {
    try {
      const a = JSON.parse(leftText || 'null');
      const b = JSON.parse(rightText || 'null');
      const delta = jsondiffpatch.diff(a, b);
      if (!delta) {
        setHtml('<div class="diff-empty"><div class="jd-icon">✓</div>两个响应完全相同</div>');
      } else {
        setHtml(htmlFormatter.format(delta, a) ?? '');
      }
      setMode('json');
      setError('');
    } catch {
      // 非 JSON：逐行文本对比
      setMode('text');
      setError('');
      setHtml(textDiffHtml(leftText, rightText));
    }
  }, [leftText, rightText]);

  const copyDiff = async () => {
    try {
      await navigator.clipboard.writeText(
        `${leftTitle}\n${leftText}\n\n---- DIFF ----\n${rightTitle}\n${rightText}`,
      );
      alert('已复制');
    } catch {
      /* ignore */
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 animate-fade-in">
      <div className="bg-white dark:bg-gray-900 rounded-xl shadow-2xl w-[860px] max-w-[95vw] max-h-[85vh] flex flex-col">
        {/* 头部 */}
        <div className="px-5 py-3 border-b border-gray-100 dark:border-gray-800 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <GitCompareArrows className="h-4 w-4 text-primary-500" />
            <h2 className="font-semibold text-base">响应对比</h2>
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-800 text-gray-500">
              {mode === 'json' ? 'JSON Diff' : '文本对比'}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <Button variant="ghost" size="sm" onClick={copyDiff}>
              复制
            </Button>
            <button onClick={onClose} className="text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 p-1">
              <X className="h-4 w-4" />
            </button>
          </div>
        </div>

        {/* 标题对 */}
        <div className="px-5 py-2 flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400 border-b border-gray-50 dark:border-gray-800">
          <span className="flex-1 truncate bg-red-50 dark:bg-red-950/30 text-red-600 dark:text-red-400 rounded px-2 py-1 font-medium">
            A · {leftTitle}
          </span>
          <span className="flex-1 truncate bg-emerald-50 dark:bg-emerald-950/30 text-emerald-600 dark:text-emerald-400 rounded px-2 py-1 font-medium">
            B · {rightTitle}
          </span>
        </div>

        {/* Diff 内容 */}
        <div className="flex-1 min-h-0 overflow-auto px-5 py-4 diff-host">
          {error ? (
            <div className="text-red-500 text-xs">{error}</div>
          ) : (
            <div dangerouslySetInnerHTML={{ __html: html }} />
          )}
        </div>
      </div>
    </div>
  );
}

/** 逐行文本对比（简单的行级标注） */
function textDiffHtml(a: string, b: string): string {
  const linesA = (a || '').split('\n');
  const linesB = (b || '').split('\n');
  const rows: string[] = [];
  const max = Math.max(linesA.length, linesB.length);
  for (let i = 0; i < max; i++) {
    const la = linesA[i] ?? '';
    const lb = linesB[i] ?? '';
    const cls = la === lb ? '' : 'row-diff';
    rows.push(
      `<div class="text-row ${cls}"><span class="ln">${i + 1}</span><span class="la">${escapeHtml(la) || '&nbsp;'}</span><span class="lb">${escapeHtml(lb) || '&nbsp;'}</span></div>`,
    );
  }
  return `<div class="text-diff">${rows.join('')}</div>`;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');
}
