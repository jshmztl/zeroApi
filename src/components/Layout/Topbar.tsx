import * as React from "react";
import { Link, useNavigate } from "react-router-dom";
import { Settings, Globe, ChevronDown, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useDataStore } from "@/store/dataStore";
import { useSettingsStore } from "@/store/settingsStore";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { Logo } from "@/components/Logo";

export function Topbar() {
  const nav = useNavigate();
  const { environments, activeEnvId, setActiveEnv, loadEnvironments } = useDataStore();
  const [open, setOpen] = React.useState(false);
  const [version, setVersion] = React.useState("");
  const [updateAvailable, setUpdateAvailable] = React.useState(false);

  React.useEffect(() => {
    tauri.appVersion().then(setVersion).catch(() => {});
  }, []);

  React.useEffect(() => {
    loadEnvironments();
  }, [loadEnvironments]);

  const activeEnv = environments.find((e) => e.id === activeEnvId);

  const handleSetActive = async (id: string | null) => {
    if (id) {
      const env = environments.find((e) => e.id === id);
      if (env) {
        await tauri.saveEnvironment({ ...env, active: true });
        await loadEnvironments();
      }
    } else {
      if (activeEnv) {
        await tauri.saveEnvironment({ ...activeEnv, active: false });
        await loadEnvironments();
      }
    }
    setActiveEnv(id);
    setOpen(false);
  };

  return (
    <header className="h-12 px-4 flex items-center gap-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
      <Link to="/" className="flex items-center gap-2 mr-2">
        <Logo size={26} />
        <span className="font-semibold text-gray-900 dark:text-gray-100 tracking-tight">ZeroApi</span>
      </Link>

      <div className="relative">
        <button
          onClick={() => setOpen(!open)}
          className="flex items-center gap-1.5 h-8 px-3 rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-sm"
        >
          <Globe className="h-3.5 w-3.5 text-primary-600 dark:text-primary-400" />
          <span className="font-medium text-gray-700 dark:text-gray-300">
            {activeEnv ? activeEnv.name : "无环境"}
          </span>
          <ChevronDown className="h-3.5 w-3.5 text-gray-400 dark:text-gray-500" />
        </button>
        {open && (
          <>
            <div className="fixed inset-0 z-30" onClick={() => setOpen(false)} />
            <div className="absolute z-40 top-full left-0 mt-1 w-64 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-lg shadow-lg p-1 animate-fade-in">
              <div className="px-2 py-1.5 text-xs text-gray-500 dark:text-gray-500 font-medium">切换环境</div>
              <button
                onClick={() => handleSetActive(null)}
                className={`w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-100 dark:hover:bg-gray-800 ${
                  !activeEnvId ? "text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-900/20" : "text-gray-700 dark:text-gray-300"
                }`}
              >
                无环境
              </button>
              {environments.map((e) => (
                <button
                  key={e.id}
                  onClick={() => handleSetActive(e.id)}
                  className={`w-full text-left px-2 py-1.5 rounded text-sm hover:bg-gray-100 dark:hover:bg-gray-800 ${
                    activeEnvId === e.id ? "text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-900/20" : "text-gray-700 dark:text-gray-300"
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <span>{e.name}</span>
                    <span className="text-[10px] text-gray-400 dark:text-gray-500">{e.vars.length} 变量</span>
                  </div>
                </button>
              ))}
              {environments.length === 0 && (
                <div className="px-2 py-3 text-xs text-gray-400 dark:text-gray-500 text-center">
                  还没有环境,去侧边栏创建一个
                </div>
              )}
            </div>
          </>
        )}
      </div>

      <div className="flex-1" />

      {updateAvailable && (
        <Button variant="primary" size="sm" onClick={() => nav("/settings")}>
          <RefreshCw className="h-3.5 w-3.5" />
          有新版本
        </Button>
      )}

      <button
        onClick={() => nav("/settings")}
        className="w-8 h-8 inline-flex items-center justify-center rounded-md hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-600 dark:text-gray-400"
        title="设置"
      >
        <Settings className="h-4 w-4" />
      </button>
    </header>
  );
}
