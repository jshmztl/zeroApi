import * as React from 'react';
import { tauri } from '@/lib/tauri';
import { toast } from '@/components/ui/Toast';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { X, Plus, Check, Eye, EyeOff } from 'lucide-react';
import { useDataStore } from '@/store/dataStore';
import { nanoid } from '@/lib/nanoid';
import type { Environment, EnvironmentVariable } from '@/types';
import { cn } from '@/lib/utils';

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
  const [vars, setVars] = React.useState<EnvironmentVariable[]>(env?.vars || []);
  const [showSecrets, setShowSecrets] = React.useState(false);

  React.useEffect(() => {
    setName(env?.name || '');
    setBaseUrl(env?.base_url || '');
    setVars(env?.vars || []);
  }, [env]);

  const addVar = () => {
    setVars([...vars, { name: '', value: '', kind: 'plain', enabled: true }]);
  };
  const updateVar = (i: number, patch: Partial<EnvironmentVariable>) => {
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
      project_id: env?.project_id ?? null,
      name: name.trim(),
      base_url: baseUrl.trim(),
      vars: vars.filter((v) => v.name.trim()),
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
      <div className="bg-white rounded-xl shadow-xl w-[560px] max-h-[80vh] flex flex-col">
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
              <label className="text-xs font-medium text-gray-600">变量 (name = value)</label>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => setShowSecrets(!showSecrets)}
                  className="text-[10px] text-gray-400 hover:text-primary-600 inline-flex items-center gap-0.5"
                  title="显示/隐藏 Secret 值"
                >
                  {showSecrets ? <EyeOff className="h-3 w-3" /> : <Eye className="h-3 w-3" />}
                  显示 Secret
                </button>
                <button
                  onClick={addVar}
                  className="text-xs text-primary-600 hover:text-primary-700 inline-flex items-center gap-1"
                >
                  <Plus className="h-3 w-3" /> 添加
                </button>
              </div>
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
                    value={v.name}
                    onChange={(e) => updateVar(i, { name: e.target.value })}
                    placeholder="NAME"
                    className="flex-1 font-mono text-xs"
                  />
                  <Input
                    type={v.kind === 'secret' && !showSecrets ? 'password' : 'text'}
                    value={v.value}
                    onChange={(e) => updateVar(i, { value: e.target.value })}
                    placeholder="value"
                    className="flex-1 font-mono text-xs"
                  />
                  {/* Secret 类型切换 */}
                  <button
                    onClick={() =>
                      updateVar(i, { kind: v.kind === 'secret' ? 'plain' : 'secret' })
                    }
                    title={v.kind === 'secret' ? 'Secret 变量（不导出/不进 Git）' : '切换为 Secret 变量'}
                    className={cn(
                      'text-[9px] px-1.5 py-0.5 rounded font-mono border flex-shrink-0',
                      v.kind === 'secret'
                        ? 'bg-amber-50 text-amber-700 border-amber-200'
                        : 'bg-gray-100 text-gray-500 border-gray-200 hover:border-amber-300',
                    )}
                  >
                    {v.kind === 'secret' ? '🔒' : '普通'}
                  </button>
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
              <code className="font-mono text-primary-600">{'{{name}}'}</code>{' '}
              引用；<span className="text-amber-600">Secret 变量</span>不会出现在导出文件中
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
