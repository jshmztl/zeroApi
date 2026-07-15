import * as React from "react";
import { X, FolderOpen } from "lucide-react";
import { Input } from "@/components/ui/Input";
import { Button } from "@/components/ui/Button";

interface Props {
  onClose: () => void;
  onSave: (name: string, description: string) => void;
}

export function CollectionDialog({ onClose, onSave }: Props) {
  const [name, setName] = React.useState("");
  const [description, setDescription] = React.useState("");

  const handleSave = () => {
    const trimmed = name.trim();
    if (!trimmed) return;
    onSave(trimmed, description.trim());
    onClose();
  };

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/40" onClick={onClose} />
      <div className="fixed inset-0 z-50 flex items-center justify-center pointer-events-none">
        <div className="pointer-events-auto w-full max-w-sm bg-white dark:bg-gray-900 rounded-xl shadow-2xl border border-gray-200 dark:border-gray-800 animate-fade-in">
          <div className="flex items-center justify-between px-5 py-4 border-b border-gray-100 dark:border-gray-800">
            <div className="flex items-center gap-2">
              <FolderOpen className="h-4 w-4 text-primary-500" />
              <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
                新建集合
              </h3>
            </div>
            <button
              onClick={onClose}
              className="p-1 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-400"
            >
              <X className="h-4 w-4" />
            </button>
          </div>

          <div className="px-5 py-4 space-y-4">
            <div>
              <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                集合名称 <span className="text-red-500">*</span>
              </label>
              <Input
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="例如：用户模块 API"
                autoFocus
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleSave();
                }}
              />
            </div>
            <div>
              <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1.5">
                描述（可选）
              </label>
              <Input
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="简要描述这个集合的用途"
              />
            </div>
          </div>

          <div className="px-5 py-3 border-t border-gray-100 dark:border-gray-800 flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={onClose}>
              取消
            </Button>
            <Button
              variant="primary"
              size="sm"
              onClick={handleSave}
              disabled={!name.trim()}
            >
              创建
            </Button>
          </div>
        </div>
      </div>
    </>
  );
}
