import * as React from "react";
import { Link, useNavigate } from "react-router-dom";
import { Settings, Globe, ChevronDown, RefreshCw, Check } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { useDataStore } from "@/store/dataStore";
import { useSettingsStore } from "@/store/settingsStore";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { Logo } from "@/components/Logo";
import { cn } from "@/lib/utils";

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
    <header className="h-13 px-4 flex items-center gap-4 border-b border-border bg-card/80 backdrop-blur-xl supports-[backdrop-filter]:bg-card/70 sticky top-0 z-30">
      <Link to="/" className="flex items-center gap-2.5 mr-2 group">
        <Logo size={26} />
        <span className="font-semibold text-gray-900 dark:text-gray-100 tracking-tight text-[15px] group-hover:text-primary-600 dark:group-hover:text-primary-400 transition-colors">
          ZeroApi
        </span>
        <span className="text-[9px] px-1.5 py-0.5 rounded-full bg-primary-50 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 font-medium hidden sm:inline-flex">
          v{version || "2"}
        </span>
      </Link>

      <div className="relative">
        <button
          onClick={() => setOpen(!open)}
          className={cn(
            "flex items-center gap-1.5 h-8 px-3 rounded-lg text-sm transition-all",
            open
              ? "bg-primary-50 dark:bg-primary-900/25 text-primary-700 dark:text-primary-300 ring-1 ring-primary-200 dark:ring-primary-800"
              : "hover:bg-gray-100 dark:hover:bg-gray-800",
          )}
        >
          <Globe className="h-3.5 w-3.5 text-primary-500" />
          <span className="font-medium text-gray-700 dark:text-gray-300">
            {activeEnv ? activeEnv.name : "无环境"}
          </span>
          <ChevronDown className={cn("h-3.5 w-3.5 text-gray-400 transition-transform", open && "rotate-180")} />
        </button>
        {open && (
          <>
            <div className="fixed inset-0 z-30" onClick={() => setOpen(false)} />
            <div className="absolute z-40 top-full left-0 mt-1.5 w-64 bg-card border border-border rounded-xl shadow-lift p-1.5 animate-fade-in">
              <div className="px-2 py-1.5 text-[10px] text-muted-foreground font-semibold uppercase tracking-wider">
                切换环境
              </div>
              <button
                onClick={() => handleSetActive(null)}
                className={cn(
                  "w-full text-left px-2.5 py-2 rounded-lg text-sm transition-colors flex items-center justify-between",
                  !activeEnvId
                    ? "text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-900/25"
                    : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800",
                )}
              >
                无环境
                {!activeEnvId && <Check className="h-3.5 w-3.5" />}
              </button>
              {environments.map((e) => (
                <button
                  key={e.id}
                  onClick={() => handleSetActive(e.id)}
                  className={cn(
                    "w-full text-left px-2.5 py-2 rounded-lg text-sm transition-colors flex items-center justify-between",
                    activeEnvId === e.id
                      ? "text-primary-700 dark:text-primary-400 bg-primary-50 dark:bg-primary-900/25"
                      : "text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800",
                  )}
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <span className="truncate">{e.name}</span>
                    <span className="text-[10px] text-muted-foreground">{e.vars.length} 变量</span>
                  </div>
                  {activeEnvId === e.id && <Check className="h-3.5 w-3.5 flex-shrink-0" />}
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
        className="w-8 h-8 inline-flex items-center justify-center rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 text-gray-500 dark:text-gray-400 hover:text-primary-600 dark:hover:text-primary-400 transition-all"
        title="设置"
      >
        <Settings className="h-4 w-4" />
      </button>
    </header>
  );
}
