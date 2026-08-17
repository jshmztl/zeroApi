import * as React from "react";
import { useNavigate } from "react-router-dom";
import {
  ArrowLeft,
  Activity,
  Loader2,
  Search,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  Clock,
  ShieldCheck,
  Globe,
  Server,
  Zap,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";

interface DiagResult {
  target: string;
  scheme: string;
  host: string;
  port: number;
  dns: { ok: boolean; addresses: string[]; ms: number; error?: string };
  tcp: { ok: boolean; port: number; ms: number; error?: string };
  tls?: { ok: boolean; ms: number; peer_cert_subject?: string; error?: string };
  http?: { ok: boolean; status?: number; ms: number; error?: string };
  total_ms: number;
}

export function DiagnosticsPage() {
  const nav = useNavigate();
  const [url, setUrl] = React.useState("https://api.github.com");
  const [loading, setLoading] = React.useState(false);
  const [result, setResult] = React.useState<DiagResult | null>(null);

  const run = async () => {
    if (!url.trim()) {
      toast.error("请输入目标地址");
      return;
    }
    setLoading(true);
    setResult(null);
    try {
      const r = await tauri.diagnoseNetwork(url.trim());
      setResult(r as any);
    } catch (e: any) {
      toast.error("诊断失败: " + String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="max-w-2xl mx-auto px-6 py-6 space-y-5">
      <button
        onClick={() => nav("/")}
        className="inline-flex items-center gap-1 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors"
      >
        <ArrowLeft className="h-4 w-4" />
        返回主页
      </button>

      <div className="flex items-center gap-2">
        <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-emerald-400 to-teal-500 flex items-center justify-center text-white shadow-md shadow-emerald-500/20">
          <Activity className="h-4.5 w-4.5" />
        </div>
        <div>
          <h1 className="text-lg font-semibold text-gray-900 dark:text-gray-100">网络诊断</h1>
          <p className="text-xs text-gray-500 dark:text-gray-500">
            DNS → TCP → TLS → HTTP 全链路探测（内网环境友好）
          </p>
        </div>
      </div>

      {/* 输入区 */}
      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
        <div className="flex items-center gap-2">
          <Input
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder="https://api.example.com"
            className="flex-1 font-mono text-sm"
            onKeyDown={(e) => {
              if (e.key === "Enter") run();
            }}
          />
          <Button variant="primary" onClick={run} disabled={loading}>
            {loading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Search className="h-3.5 w-3.5" />}
            {loading ? "诊断中..." : "开始诊断"}
          </Button>
        </div>
        <div className="mt-2 flex flex-wrap gap-1.5">
          {["https://api.github.com", "https://www.baidu.com", "http://localhost:8080", "https://10.0.0.1"].map((s) => (
            <button
              key={s}
              onClick={() => setUrl(s)}
              className="text-[10px] px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-500 dark:text-gray-400 hover:bg-primary-50 hover:text-primary-700 transition-colors"
            >
              {s}
            </button>
          ))}
        </div>
      </div>

      {/* 结果区 */}
      {loading && (
        <div className="flex flex-col items-center py-12 text-gray-400">
          <Loader2 className="h-6 w-6 animate-spin mb-2" />
          <span className="text-xs">正在探测 {url}...</span>
        </div>
      )}

      {result && !loading && (
        <div className="space-y-3 animate-fade-in">
          {/* 目标摘要 */}
          <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
            <div className="flex items-center justify-between">
              <div className="min-w-0">
                <div className="text-sm font-medium text-gray-800 dark:text-gray-200 font-mono truncate">
                  {result.target}
                </div>
                <div className="text-[10px] text-gray-400 mt-0.5">
                  {result.host}:{result.port} · {result.scheme}
                </div>
              </div>
              <div className="text-right">
                <div className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {result.total_ms}ms
                </div>
                <div className="text-[10px] text-gray-400">总耗时</div>
              </div>
            </div>
          </div>

          {/* 分段结果 */}
          <DiagRow
            icon={<Globe className="h-4 w-4" />}
            label="DNS 解析"
            ok={result.dns.ok}
            detail={result.dns.addresses.join(", ") || result.dns.error}
            ms={result.dns.ms}
            hint={result.dns.ok ? "解析成功" : "解析失败"}
          />
          <DiagRow
            icon={<Server className="h-4 w-4" />}
            label={`TCP 连接 (:${result.tcp.port})`}
            ok={result.tcp.ok}
            detail={result.tcp.error}
            ms={result.tcp.ms}
            hint={result.tcp.ok ? "连接成功" : "连接失败"}
          />
          {result.tls && (
            <DiagRow
              icon={<ShieldCheck className="h-4 w-4" />}
              label="TLS 握手"
              ok={result.tls.ok}
              detail={result.tls.peer_cert_subject || result.tls.error}
              ms={result.tls.ms}
              hint={result.tls.ok ? "握手成功" : "证书/握手错误"}
              warn={!result.tls.ok}
            />
          )}
          {result.http && (
            <DiagRow
              icon={<Zap className="h-4 w-4" />}
              label="HTTP 探测"
              ok={result.http.ok}
              detail={result.http.status ? `HTTP ${result.http.status}` : result.http.error}
              ms={result.http.ms}
              hint={result.http.status ? `状态码 ${result.http.status}` : "请求失败"}
              warn={result.http.status != null && result.http.status >= 400}
            />
          )}

          {/* 失败建议 */}
          {!result.dns.ok && (
            <Advice
              title="DNS 解析失败"
              items={[
                "检查域名拼写是否正确",
                "尝试 ping 域名确认网络",
                "检查系统 DNS 配置 / 代理设置",
              ]}
            />
          )}
          {result.dns.ok && !result.tcp.ok && (
            <Advice
              title="TCP 连接被拒绝"
              items={[
                "检查服务是否已启动",
                "检查端口是否监听",
                "检查防火墙规则",
                "内网环境确认是否在 VPN 内",
              ]}
            />
          )}
          {result.tcp.ok && result.tls && !result.tls.ok && (
            <Advice
              title="TLS 证书错误"
              items={[
                "确认证书是否过期",
                "内网自签名证书需在设置中关闭 SSL 验证",
                "检查 SNI / 服务器名称是否匹配",
              ]}
            />
          )}
        </div>
      )}
    </div>
  );
}

function DiagRow({
  icon,
  label,
  ok,
  detail,
  ms,
  hint,
  warn,
}: {
  icon: React.ReactNode;
  label: string;
  ok: boolean;
  detail?: string;
  ms: number;
  hint: string;
  warn?: boolean;
}) {
  const status = !ok ? "fail" : warn ? "warn" : "ok";
  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm flex items-center gap-3">
      <div
        className={cn(
          "w-9 h-9 rounded-xl flex items-center justify-center flex-shrink-0",
          status === "ok" && "bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400",
          status === "warn" && "bg-amber-50 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400",
          status === "fail" && "bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400",
        )}
      >
        {icon}
      </div>
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium text-gray-800 dark:text-gray-200">{label}</span>
          {status === "ok" && <CheckCircle2 className="h-3.5 w-3.5 text-emerald-500" />}
          {status === "warn" && <AlertTriangle className="h-3.5 w-3.5 text-amber-500" />}
          {status === "fail" && <XCircle className="h-3.5 w-3.5 text-red-500" />}
        </div>
        <div className="text-[11px] text-gray-500 dark:text-gray-400 font-mono truncate mt-0.5">
          {detail || hint}
        </div>
      </div>
      <div className="flex items-center gap-1 text-sm font-semibold text-gray-700 dark:text-gray-300 flex-shrink-0">
        <Clock className="h-3 w-3 text-gray-400" />
        {ms}ms
      </div>
    </div>
  );
}

function Advice({ title, items }: { title: string; items: string[] }) {
  return (
    <div className="bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-800 rounded-2xl p-4">
      <div className="text-sm font-medium text-amber-800 dark:text-amber-300 mb-2">💡 {title}，建议：</div>
      <ol className="space-y-1">
        {items.map((it, i) => (
          <li key={i} className="text-xs text-amber-700 dark:text-amber-400/90 flex gap-2">
            <span className="text-amber-500 font-semibold">{i + 1}.</span>
            {it}
          </li>
        ))}
      </ol>
    </div>
  );
}
