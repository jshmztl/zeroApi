import * as React from "react";
import { useNavigate } from "react-router-dom";
import { ArrowLeft, Save, Trash2, RefreshCw, Download, Upload, Github, Sun, Moon, Monitor, FolderTree, Cookie, X } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { useSettingsStore } from "@/store/settingsStore";
import { useDataStore } from "@/store/dataStore";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { copyToClipboard, downloadText } from "@/lib/formatter";
import { Logo } from "@/components/Logo";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { open } from "@tauri-apps/plugin-dialog";
import type { Project } from "@/types";

export function SettingsPage() {
  const nav = useNavigate();
  const { settings, update } = useSettingsStore();
  const { loadAll } = useDataStore();
  const [version, setVersion] = React.useState("");
  const [projects, setProjects] = React.useState<Project[]>([]);
  const [exportProjectId, setExportProjectId] = React.useState("");
  const [cookieUrl, setCookieUrl] = React.useState("");
  const [cookieText, setCookieText] = React.useState("");
  const [cookieLoading, setCookieLoading] = React.useState(false);

  React.useEffect(() => {
    tauri.appVersion().then(setVersion).catch(() => {});
    tauri.listProjects().then((ps) => {
      setProjects(ps);
      if (ps.length > 0) setExportProjectId(ps[0].id);
    }).catch(() => {});
  }, []);

  // 离开设置页 / 关闭窗口时强制 flush 待保存的设置
  React.useEffect(() => {
    const handler = () => {
      useSettingsStore.getState().flush();
    };
    window.addEventListener("beforeunload", handler);
    return () => {
      handler();
      window.removeEventListener("beforeunload", handler);
    };
  }, []);

  const clearAll = async () => {
    const input = window.prompt(
      '⚠️ 此操作将永久删除全部数据（历史/收藏/集合/环境/设置）且无法恢复！\n\n请输入 "确认清空" 四个字后点击确定：'
    );
    if (input !== '确认清空') {
      if (input !== null) toast.error('输入不正确，操作已取消');
      return;
    }
    await tauri.clearAllData();
    await loadAll();
    toast.success('已清空所有数据');
  };

  const exportAll = async () => {
    const json = await tauri.exportJson();
    const filename = `zeroapi-backup-${new Date().toISOString().slice(0, 10)}.json`;
    downloadText(json, filename);
    toast.success("已导出");
  };

  const importFromFile = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "application/json";
    input.onchange = async (e: any) => {
      const file = e.target.files?.[0];
      if (!file) return;
      const text = await file.text();
      try {
        await tauri.importJson(text);
        await loadAll();
        toast.success("导入成功");
      } catch (err) {
        toast.error("导入失败: " + String(err));
      }
    };
    input.click();
  };

  const checkUpdate = async () => {
    try {
      const update = await check();
      if (update) {
        toast.info(`发现新版本 ${update.version},正在准备...`);
        await update.downloadAndInstall();
        await relaunch();
      } else {
        toast.success("已是最新版本");
      }
    } catch (e) {
      toast.error("检查更新失败: " + String(e));
    }
  };

  /* ---- 项目文件（Git-friendly）---- */
  const exportProjectDir = async () => {
    if (!exportProjectId) {
      toast.error("请先创建项目");
      return;
    }
    const dir = await open({ directory: true, title: "选择导出目录" });
    if (!dir) return;
    try {
      await tauri.exportProject(exportProjectId, dir);
      toast.success(`已导出到 ${dir}`);
    } catch (e) {
      toast.error("导出失败: " + String(e));
    }
  };

  const importProjectDir = async () => {
    const dir = await open({ directory: true, title: "选择项目目录（含 zeroapi.yaml）" });
    if (!dir) return;
    try {
      const pid = await tauri.importProject(dir);
      toast.success("项目导入成功");
      const ps = await tauri.listProjects();
      setProjects(ps);
      setExportProjectId(pid);
      await loadAll();
    } catch (e) {
      toast.error("导入失败: " + String(e));
    }
  };

  /* ---- Cookie 会话管理 ---- */
  const currentProjectId = exportProjectId;
  const queryCookies = async () => {
    if (!currentProjectId) {
      toast.error("请先创建项目");
      return;
    }
    if (!cookieUrl.trim()) {
      toast.error("请输入 URL");
      return;
    }
    setCookieLoading(true);
    try {
      const c = await tauri.getSessionCookies(currentProjectId, cookieUrl.trim());
      setCookieText(c || "（该域名下暂无 Cookie）");
    } catch (e) {
      toast.error("查询失败: " + String(e));
    } finally {
      setCookieLoading(false);
    }
  };
  const clearCookies = async () => {
    if (!currentProjectId) return;
    await tauri.clearSessionCookies(currentProjectId);
    setCookieText("");
    toast.success("已清空该项目会话 Cookie");
  };

  return (
    <div className="max-w-2xl mx-auto px-6 py-8 space-y-8">
      {/* 返回按钮 */}
      <button
        onClick={() => nav("/")}
        className="inline-flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
      >
        <ArrowLeft className="h-4 w-4" />
        返回主页
      </button>

      <div className="flex items-center gap-3 pb-4 border-b border-gray-200 dark:border-gray-800">
        <Logo size={36} />
        <div>
          <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100">设置</h1>
          <p className="text-xs text-gray-500 dark:text-gray-500">ZeroApi · v{version} · 本地构建</p>
        </div>
      </div>

      {/* 外观 */}
      <Section title="外观" desc="切换应用主题">
        <Row label="主题">
          <div className="flex gap-1">
            {[
              { v: "light", icon: Sun, label: "浅色" },
              { v: "dark", icon: Moon, label: "深色" },
              { v: "system", icon: Monitor, label: "跟随系统" },
            ].map((o) => {
              const Icon = o.icon;
              return (
                <button
                  key={o.v}
                  onClick={() => update({ theme: o.v as any })}
                  className={`h-8 px-2.5 text-xs rounded-md inline-flex items-center gap-1.5 transition-colors ${
                    settings.theme === o.v
                      ? "bg-primary-50 dark:bg-primary-900/20 text-primary-700 dark:text-primary-400 border border-primary-200 dark:border-primary-800"
                      : "text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 border border-transparent"
                  }`}
                >
                  <Icon className="h-3.5 w-3.5" />
                  {o.label}
                </button>
              );
            })}
          </div>
        </Row>
      </Section>

      {/* 网络 */}
      <Section title="网络" desc="超时 / 代理 / SSL">
        <Row label="请求超时(毫秒)">
          <Input
            type="number"
            value={settings.timeout_ms}
            onChange={(e) => update({ timeout_ms: Number(e.target.value) || 30000 })}
            className="w-32"
          />
        </Row>
        <Row label="代理 URL" desc="例如 http://127.0.0.1:7890 · 留空表示不使用">
          <Input
            value={settings.proxy_url}
            onChange={(e) => update({ proxy_url: e.target.value })}
            placeholder="http://127.0.0.1:7890"
            className="w-72 font-mono"
          />
        </Row>
        <Row label="SSL 证书验证">
          <Toggle
            checked={settings.verify_ssl}
            onChange={(v) => update({ verify_ssl: v })}
          />
        </Row>
        <Row label="自动跟随重定向">
          <Toggle
            checked={settings.follow_redirects}
            onChange={(v) => update({ follow_redirects: v })}
          />
        </Row>
      </Section>

      {/* 历史 */}
      <Section title="历史记录" desc="自动保存请求历史">
        <Row label="自动保存历史">
          <Toggle
            checked={settings.auto_save_history}
            onChange={(v) => update({ auto_save_history: v })}
          />
        </Row>
        <Row label="最多保留条数">
          <Input
            type="number"
            value={settings.history_limit}
            onChange={(e) => update({ history_limit: Number(e.target.value) || 100 })}
            className="w-24"
            disabled={!settings.auto_save_history}
          />
        </Row>
      </Section>

      {/* 数据 */}
      <Section title="数据管理" desc="导入 / 导出 / 清空">
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={exportAll}>
            <Download className="h-3.5 w-3.5" />
            导出全部为 JSON
          </Button>
          <Button variant="outline" onClick={importFromFile}>
            <Upload className="h-3.5 w-3.5" />
            从 JSON 导入
          </Button>
          <Button variant="danger" onClick={clearAll}>
            <Trash2 className="h-3.5 w-3.5" />
            清空所有数据
          </Button>
        </div>
      </Section>

      {/* 项目文件（Git-friendly） */}
      <Section title="项目文件" desc="导出为可进入 Git 的目录结构（zeroapi.yaml + collections/*.yaml + environments/*.yaml）">
        <Row label="项目" desc="选择要导出 / Cookie 会话所属的项目">
          <select
            value={exportProjectId}
            onChange={(e) => setExportProjectId(e.target.value)}
            className="h-8 px-2 text-xs border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-primary-500/30"
          >
            {projects.length === 0 && <option value="">（暂无项目，请先在集合中创建）</option>}
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.name}
              </option>
            ))}
          </select>
        </Row>
        <div className="flex flex-wrap gap-2 pt-1">
          <Button variant="outline" onClick={exportProjectDir}>
            <FolderTree className="h-3.5 w-3.5" />
            导出项目到目录
          </Button>
          <Button variant="outline" onClick={importProjectDir}>
            <Upload className="h-3.5 w-3.5" />
            从目录导入项目
          </Button>
        </div>
        <p className="text-[11px] text-gray-400 dark:text-gray-500">
          Secret 变量不会出现在项目文件中（只保留名称与类型），重新导入后需重新填写。
        </p>
      </Section>

      {/* Cookie 会话 */}
      <Section title="Cookie 会话" desc="按项目隔离的 Cookie（文档 §31）">
        <div className="space-y-2">
          <div className="flex items-center gap-2">
            <Input
              value={cookieUrl}
              onChange={(e) => setCookieUrl(e.target.value)}
              placeholder="https://api.example.com"
              className="flex-1 font-mono text-xs"
              onKeyDown={(e) => {
                if (e.key === "Enter") queryCookies();
              }}
            />
            <Button variant="outline" size="sm" onClick={queryCookies} disabled={cookieLoading}>
              <Cookie className="h-3.5 w-3.5" />
              {cookieLoading ? "查询中..." : "查看 Cookie"}
            </Button>
            <Button variant="danger" size="sm" onClick={clearCookies}>
              <X className="h-3.5 w-3.5" />
              清空
            </Button>
          </div>
          {cookieText && (
            <div className="flex items-start gap-2">
              <pre className="flex-1 text-[11px] font-mono text-gray-600 dark:text-gray-300 bg-gray-50 dark:bg-gray-800 border border-gray-100 dark:border-gray-700 rounded p-2 whitespace-pre-wrap break-all">
                {cookieText}
              </pre>
              <Button
                variant="ghost"
                size="sm"
                onClick={async () => {
                  const ok = await copyToClipboard(cookieText);
                  toast[ok ? "success" : "error"](ok ? "已复制" : "复制失败");
                }}
              >
                复制
              </Button>
            </div>
          )}
        </div>
      </Section>

      {/* 更新 */}
      <Section title="应用更新" desc="通过 GitHub Releases 自动更新">
        <div className="flex items-center gap-2">
          <Button variant="primary" onClick={checkUpdate}>
            <RefreshCw className="h-3.5 w-3.5" />
            检查更新
          </Button>
          <span className="text-xs text-gray-500 dark:text-gray-500">当前 v{version}</span>
        </div>
      </Section>

      {/* 关于 */}
      <Section title="关于" desc="">
        <div className="text-xs text-gray-500 dark:text-gray-500 space-y-1">
          <div>ZeroApi 是一款开源、零登录、零云端、零 CORS 的 API 调试工具。</div>
          <div className="flex items-center gap-1">
            <Github className="h-3 w-3" />
            <a
              href="https://github.com/jshmztl/zeroApi"
              target="_blank"
              rel="noreferrer"
              className="text-primary-600 dark:text-primary-400 hover:underline"
            >
              github.com/jshmztl/zeroApi
            </a>
          </div>
          <div className="text-gray-400 dark:text-gray-600">© 2026 ZeroApi · Made with 💚</div>
        </div>
      </Section>
    </div>
  );
}

function Section({ title, desc, children }: { title: string; desc?: string; children: React.ReactNode }) {
  return (
    <section className="space-y-3">
      <div>
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">{title}</h2>
        {desc && <p className="text-xs text-gray-500 dark:text-gray-500 mt-0.5">{desc}</p>}
      </div>
      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-xl p-4 space-y-3">{children}</div>
    </section>
  );
}

function Row({ label, desc, children }: { label: string; desc?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-1.5">
      <div className="min-w-0">
        <div className="text-sm text-gray-700 dark:text-gray-300">{label}</div>
        {desc && <div className="text-[11px] text-gray-400 dark:text-gray-500 mt-0.5">{desc}</div>}
      </div>
      <div className="flex-shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <button
      onClick={() => onChange(!checked)}
      className={`relative w-9 h-5 rounded-full transition-colors overflow-hidden ${checked ? "bg-primary-500" : "bg-gray-300 dark:bg-gray-600"}`}
    >
      <span
        className={`absolute top-0.5 left-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform ${
          checked ? "translate-x-4" : "translate-x-0"
        }`}
      />
    </button>
  );
}
