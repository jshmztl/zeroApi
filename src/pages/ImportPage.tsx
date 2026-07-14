import * as React from "react";
import { useNavigate } from "react-router-dom";
import { Wand2, Upload, FileJson } from "lucide-react";
import { Button } from "@/components/ui/Button";
import { CodeEditor } from "@/components/CodeEditor/CodeEditor";
import { tauri } from "@/lib/tauri";
import { useRequestStore } from "@/store/requestStore";
import { useDataStore } from "@/store/dataStore";
import { toast } from "@/components/ui/Toast";

const EXAMPLES = [
  {
    name: "GET · JSONPlaceholder",
    cmd: `curl https://jsonplaceholder.typicode.com/posts/1`,
  },
  {
    name: "POST · 创建文章",
    cmd: `curl -X POST https://jsonplaceholder.typicode.com/posts \\
  -H "Content-Type: application/json" \\
  -d '{"title":"foo","body":"bar","userId":1}'`,
  },
  {
    name: "GET · 携带 Bearer",
    cmd: `curl https://api.github.com/user \\
  -H "Authorization: Bearer ghp_xxxxxxxxxxxx" \\
  -H "Accept: application/vnd.github+json"`,
  },
  {
    name: "GET · Basic Auth",
    cmd: `curl -u admin:secret https://httpbin.org/basic-auth/admin/secret`,
  },
  {
    name: "POST · form-data",
    cmd: `curl -X POST https://httpbin.org/post \\
  -F "name=Alice" \\
  -F "avatar=@/path/to/file.png"`,
  },
];

export function ImportPage() {
  const nav = useNavigate();
  const { loadRequest } = useRequestStore();
  const { loadFavorites, loadCollections, loadEnvironments } = useDataStore();
  const [curlText, setCurlText] = React.useState("");
  const [jsonText, setJsonText] = React.useState("");
  const [mode, setMode] = React.useState<"curl" | "json">("curl");

  const importCurl = async () => {
    if (!curlText.trim()) {
      toast.error("请先粘贴 cURL 命令");
      return;
    }
    try {
      const req = await tauri.importCurl(curlText);
      loadRequest(req);
      toast.success("cURL 已解析 · 已加载到主工作区");
      nav("/");
    } catch (e: any) {
      toast.error("解析失败: " + String(e));
    }
  };

  const importJson = async () => {
    if (!jsonText.trim()) {
      toast.error("请先粘贴 JSON 内容");
      return;
    }
    try {
      const result = await tauri.importJson(jsonText);
      await Promise.all([loadFavorites(), loadCollections(), loadEnvironments()]);
      toast.success(
        `导入完成: ${result.requests.length} 请求 / ${result.collections.length} 集合 / ${result.environments.length} 环境`
      );
    } catch (e: any) {
      toast.error("导入失败: " + String(e));
    }
  };

  return (
    <div className="max-w-3xl mx-auto px-6 py-6 space-y-4">
      <div>
        <h1 className="text-lg font-semibold text-gray-900 flex items-center gap-2">
          <Wand2 className="h-4 w-4 text-primary-500" />
          导入
        </h1>
        <p className="text-xs text-gray-500 mt-0.5">从 cURL 命令或 JSON 备份快速迁移</p>
      </div>

      <div className="flex gap-1">
        <button
          onClick={() => setMode("curl")}
          className={`h-8 px-3 text-xs rounded-md ${
            mode === "curl"
              ? "bg-primary-50 text-primary-700 border border-primary-200"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          cURL 命令
        </button>
        <button
          onClick={() => setMode("json")}
          className={`h-8 px-3 text-xs rounded-md ${
            mode === "json"
              ? "bg-primary-50 text-primary-700 border border-primary-200"
              : "text-gray-600 hover:bg-gray-100"
          }`}
        >
          JSON 备份
        </button>
      </div>

      {mode === "curl" && (
        <div className="space-y-3">
          <div className="bg-white border border-gray-100 rounded-xl p-4 space-y-3">
            <div className="text-xs text-gray-500">
              粘贴一段 cURL 命令(从浏览器 DevTools / 终端复制),ZeroApi 会自动解析为请求并加载到主工作区。
            </div>
            <CodeEditor
              value={curlText}
              onChange={setCurlText}
              language="shell"
              height="180px"
            />
            <div className="flex justify-end">
              <Button variant="primary" onClick={importCurl}>
                <Wand2 className="h-3.5 w-3.5" />
                解析并使用
              </Button>
            </div>
          </div>

          <div className="bg-gray-50 border border-gray-100 rounded-xl p-3 space-y-2">
            <div className="text-xs font-medium text-gray-700">快速示例 · 点击使用</div>
            <div className="grid gap-1.5">
              {EXAMPLES.map((ex) => (
                <button
                  key={ex.name}
                  onClick={() => setCurlText(ex.cmd)}
                  className="text-left p-2 rounded hover:bg-white text-xs"
                >
                  <div className="font-medium text-gray-700">{ex.name}</div>
                  <div className="text-gray-500 font-mono text-[10px] mt-0.5 line-clamp-1">{ex.cmd}</div>
                </button>
              ))}
            </div>
          </div>
        </div>
      )}

      {mode === "json" && (
        <div className="bg-white border border-gray-100 rounded-xl p-4 space-y-3">
          <div className="text-xs text-gray-500">
            粘贴由 ZeroApi「导出全部」生成的 JSON 内容。导入将合并到当前数据库(同 ID 会覆盖)。
          </div>
          <CodeEditor
            value={jsonText}
            onChange={setJsonText}
            language="json"
            height="280px"
          />
          <div className="flex justify-end">
            <Button variant="primary" onClick={importJson}>
              <FileJson className="h-3.5 w-3.5" />
              导入 JSON
            </Button>
          </div>
        </div>
      )}
    </div>
  );
}
