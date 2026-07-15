import * as React from 'react';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { X, Plus, Check } from 'lucide-react';
import { useDataStore } from '@/store/dataStore';
import { nanoid } from '@/lib/nanoid';
import type { Environment, KeyValue } from '@/types';

export function EnvironmentDialog({
  env,
  onClose,
}: {
  env: Environment | null;
  onClose: () => void;
}) {
  const { loadEnvironments } = useDataStore();
  const [name, setName] = React.useState(env?.name || '');
  const [baseUrl, setBaseUrl] = React.useState(env?.base_url || '');
  const [vars, setVars] = React.useState<KeyValue[]>(env?.vars || []);

  React.useEffect(() => {
    setName(env?.name || '');
    setBaseUrl(env?.base_url || '');
    setVars(env?.vars || []);
  }, [env]);

  const addVar = () => {
    setVars([...vars, { key: '', value: '', enabled: true }]);
  };
  const updateVar = (i: number, patch: Partial<KeyValue>) => {
    setVars(vars.map((v, idx) => (idx === i ? { ...v, ...patch } : v)));
  };
  const removeVar = (i: number) => {
    setVars(vars.filter((_, idx) => idx !== i));
  };

  const save = async () => {
    if (!name.trim()) {
      toast.error('请输入环境名');
      return;
    }
    const newEnv: Environment = {
      id: env?.id || nanoid(),
      name: name.trim(),
      base_url: baseUrl.trim(),
      vars: vars.filter((v) => v.key.trim()),
      active: env?.active || false,
    };
    try {
      await tauri.saveEnvironment(newEnv);
      await loadEnvironments();
      toast.success('已保存');
      onClose();
    } catch (e) {
      toast.error('保存失败: ' + String(e));
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 animate-fade-in">
      <div className="bg-white rounded-xl shadow-xl w-[520px] max-h-[80vh] flex flex-col">
        <div className="px-5 py-3 border-b border-gray-100 flex items-center justify-between">
          <h2 className="font-semibold text-base">{env ? '编辑环境' : '新建环境'}</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-700">
            <X className="h-4 w-4" />
          </button>
        </div>
        <div className="px-5 py-4 space-y-4 overflow-auto">
          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">环境名</label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="例如：开发环境 / 生产环境"
              autoFocus
            />
          </div>

          <div>
            <label className="block text-xs font-medium text-gray-600 mb-1">
              接口前缀 (base URL)
            </label>
            <Input
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              placeholder="https://api.dev.example.com"
              className="font-mono text-xs"
            />
            <p className="mt-1 text-[11px] text-gray-400">
              以 <code className="font-mono text-primary-600">/</code> 开头的 URL 会自动拼接此外前缀
            </p>
          </div>

          <div>
            <div className="flex items-center justify-between mb-1.5">
              <label className="text-xs font-medium text-gray-600">变量 (key = value)</label>
              <button
                onClick={addVar}
                className="text-xs text-primary-600 hover:text-primary-700 inline-flex items-center gap-1"
              >
                <Plus className="h-3 w-3" /> 添加
              </button>
            </div>
            <div className="space-y-1.5 border border-gray-100 rounded-md p-2 bg-gray-50 max-h-64 overflow-auto">
              {vars.length === 0 && (
                <div className="text-xs text-gray-400 text-center py-4">
                  还没有变量 · 点击「添加」开始
                </div>
              )}
              {vars.map((v, i) => (
                <div key={i} className="flex items-center gap-1.5">
                  <input
                    type="checkbox"
                    checked={v.enabled}
                    onChange={(e) => updateVar(i, { enabled: e.target.checked })}
                    className="rounded"
                  />
                  <Input
                    value={v.key}
                    onChange={(e) => updateVar(i, { key: e.target.value })}
                    placeholder="KEY"
                    className="flex-1 font-mono text-xs"
                  />
                  <Input
                    value={v.value}
                    onChange={(e) => updateVar(i, { value: e.target.value })}
                    placeholder="value"
                    className="flex-1 font-mono text-xs"
                  />
                  <button
                    onClick={() => removeVar(i)}
                    className="text-gray-400 hover:text-red-500 p-1"
                  >
                    <X className="h-3 w-3" />
                  </button>
                </div>
              ))}
            </div>
            <p className="mt-1.5 text-[11px] text-gray-400">
              在 URL、Headers、Body 中用{' '}
              <code className="font-mono text-primary-600">{'{{key}}'}</code> 引用
            </p>
          </div>
        </div>
        <div className="px-5 py-3 border-t border-gray-100 flex items-center justify-end gap-2">
          <Button variant="ghost" onClick={onClose}>
            取消
          </Button>
          <Button variant="primary" onClick={save}>
            <Check className="h-3.5 w-3.5" />
            保存
          </Button>
        </div>
      </div>
    </div>
  );
}
