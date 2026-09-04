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
  Network,
  Waypoints,
  Route,
} from "lucide-react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import { tauri } from "@/lib/tauri";
import { toast } from "@/components/ui/Toast";
import { cn } from "@/lib/utils";

interface CertInfo {
  depth: number;
  subject: string;
  issuer: string;
  serial: string;
  not_before: string;
  not_after: string;
  is_ca: boolean;
  expired: boolean;
}

interface DiagResult {
  target: string;
  scheme: string;
  host: string;
  port: number;
  dns: { ok: boolean; addresses: string[]; ms: number; error?: string };
  tcp: { ok: boolean; port: number; ms: number; error?: string };
  tls?: { ok: boolean; ms: number; peer_cert_subject?: string; chain?: CertInfo[]; error?: string };
  http?: { ok: boolean; status?: number; ms: number; error?: string };
  total_ms: number;
}

interface ProxyDiagResult {
  proxy_host: string;
  proxy_port: number;
  proxy_tcp: { ok: boolean; port: number; ms: number; error?: string };
  target: string;
  scheme: string;
  target_host: string;
  dns: { ok: boolean; addresses: string[]; ms: number; error?: string };
  tls?: { ok: boolean; ms: number; peer_cert_subject?: string; chain?: CertInfo[]; error?: string };
  http?: { ok: boolean; status?: number; ms: number; error?: string };
  total_ms: number;
}

interface RouteHop {
  ttl: number;
  ip?: string;
  ms: number;
  ok: boolean;
  reached: boolean;
  status?: number;
}

interface RouteResult {
  target: string;
  resolved_ip?: string;
  hops: RouteHop[];
  reached: boolean;
  max_ttl: number;
  total_ms: number;
  error?: string;
}

export function DiagnosticsPage() {
  const nav = useNavigate();
  const [mode, setMode] = React.useState<"direct" | "proxy" | "route">("direct");
  const [url, setUrl] = React.useState("https://api.github.com");
  const [proxyUrl, setProxyUrl] = React.useState("http://127.0.0.1:7897");
  const [loading, setLoading] = React.useState(false);
  const [result, setResult] = React.useState<DiagResult | null>(null);
  const [proxyResult, setProxyResult] = React.useState<ProxyDiagResult | null>(null);
  const [routeResult, setRouteResult] = React.useState<RouteResult | null>(null);

  const runDirect = async () => {
    if (!url.trim()) {
      toast.error("请输入目标地址");
      return;
    }
    setLoading(true);
    setResult(null);
    setProxyResult(null);
    setRouteResult(null);
    try {
      const r = await tauri.diagnoseNetwork(url.trim());
      setResult(r as any);
    } catch (e: any) {
      toast.error("诊断失败: " + String(e));
    } finally {
      setLoading(false);
    }
  };

  const runProxy = async () => {
    if (!url.trim()) {
      toast.error("请输入目标地址");
      return;
    }
    if (!proxyUrl.trim()) {
      toast.error("请输入代理地址");
      return;
    }
    setLoading(true);
    setResult(null);
    setProxyResult(null);
    setRouteResult(null);
    try {
      const r = await tauri.diagnoseProxyNetwork(url.trim(), proxyUrl.trim());
      setProxyResult(r as any);
    } catch (e: any) {
      toast.error("代理诊断失败: " + String(e));
    } finally {
      setLoading(false);
    }
  };

  const runRoute = async () => {
    if (!url.trim()) {
      toast.error("请输入目标地址");
      return;
    }
    setLoading(true);
    setResult(null);
    setProxyResult(null);
    setRouteResult(null);
    try {
      const r = await tauri.diagnoseRoute(url.trim());
      setRouteResult(r as any);
    } catch (e: any) {
      toast.error("路由追踪失败: " + String(e));
    } finally {
      setLoading(false);
    }
  };

  const run = () => (mode === "proxy" ? runProxy() : mode === "route" ? runRoute() : runDirect());

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
        <div className="w-9 h-9 rounded-xl bg-gradient-to-br from-cyan-400 to-primary-600 flex items-center justify-center text-white shadow-md shadow-primary-500/25">
          <Activity className="h-4.5 w-4.5" />
        </div>
        <div>
          <h1 className="text-lg font-display font-semibold text-gray-900 dark:text-gray-100">网络诊断</h1>
          <p className="text-xs text-gray-500 dark:text-gray-500">
            {mode === "proxy"
              ? "通过代理探测目标连通性（代理 TCP → CONNECT 隧道 → 经代理请求）"
              : mode === "route"
                ? "逐跳路由追踪（ICMP traceroute，无需管理员权限，仅 IPv4）"
                : "DNS → TCP → TLS → HTTP 全链路探测（内网环境友好）"}
          </p>
        </div>
      </div>

      {/* 模式切换 */}
      <div className="inline-flex rounded-xl bg-gray-100 dark:bg-gray-800 p-1 text-xs font-medium">
        <button
          onClick={() => setMode("direct")}
          className={cn(
            "px-3 py-1.5 rounded-lg transition-colors",
            mode === "direct" ? "bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 shadow-sm" : "text-gray-500 dark:text-gray-400",
          )}
        >
          直连诊断
        </button>
        <button
          onClick={() => setMode("proxy")}
          className={cn(
            "px-3 py-1.5 rounded-lg transition-colors inline-flex items-center gap-1",
            mode === "proxy" ? "bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 shadow-sm" : "text-gray-500 dark:text-gray-400",
          )}
        >
          <Waypoints className="h-3.5 w-3.5" />
          代理探针
        </button>
        <button
          onClick={() => setMode("route")}
          className={cn(
            "px-3 py-1.5 rounded-lg transition-colors inline-flex items-center gap-1",
            mode === "route" ? "bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 shadow-sm" : "text-gray-500 dark:text-gray-400",
          )}
        >
          <Route className="h-3.5 w-3.5" />
          路由追踪
        </button>
      </div>

      {/* 输入区 */}
      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
        {mode === "proxy" && (
          <div className="flex items-center gap-2 mb-3">
            <Network className="h-4 w-4 text-gray-400 flex-shrink-0" />
            <Input
              value={proxyUrl}
              onChange={(e) => setProxyUrl(e.target.value)}
              placeholder="http://127.0.0.1:7897"
              className="flex-1 font-mono text-sm"
            />
          </div>
        )}
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
          <span className="text-xs">
            {mode === "proxy" ? `正在经代理 ${proxyUrl} 探测 ${url}...` : `正在探测 ${url}...`}
          </span>
        </div>
      )}

      {!loading && mode === "direct" && result && <DirectResult result={result} />}
      {!loading && mode === "proxy" && proxyResult && <ProxyResult result={proxyResult} proxyUrl={proxyUrl} />}
      {!loading && mode === "route" && routeResult && <RouteResultView result={routeResult} />}
    </div>
  );
}

function DirectResult({ result }: { result: DiagResult }) {
  return (
    <div className="space-y-3 animate-fade-in">
      <SummaryCard title={result.target} sub={`${result.host}:${result.port} · ${result.scheme}`} totalMs={result.total_ms} />
      <DiagRow icon={<Globe className="h-4 w-4" />} label="DNS 解析" ok={result.dns.ok} detail={result.dns.addresses.join(", ") || result.dns.error} ms={result.dns.ms} hint={result.dns.ok ? "解析成功" : "解析失败"} />
      <DiagRow icon={<Server className="h-4 w-4" />} label={`TCP 连接 (:${result.tcp.port})`} ok={result.tcp.ok} detail={result.tcp.error} ms={result.tcp.ms} hint={result.tcp.ok ? "连接成功" : "连接失败"} />
      {result.tls && (
        <DiagRow icon={<ShieldCheck className="h-4 w-4" />} label="TLS 握手" ok={result.tls.ok} detail={result.tls.peer_cert_subject || result.tls.error} ms={result.tls.ms} hint={result.tls.ok ? "握手成功" : "证书/握手错误"} warn={!result.tls.ok} />
      )}
      {result.tls?.ok && result.tls.chain && result.tls.chain.length > 0 && <CertChainView chain={result.tls.chain} />}
      {result.http && (
        <DiagRow icon={<Zap className="h-4 w-4" />} label="HTTP 探测" ok={result.http.ok} detail={result.http.status ? `HTTP ${result.http.status}` : result.http.error} ms={result.http.ms} hint={result.http.status ? `状态码 ${result.http.status}` : "请求失败"} warn={result.http.status != null && result.http.status >= 400} />
      )}

      {!result.dns.ok && (
        <Advice title="DNS 解析失败" items={["检查域名拼写是否正确", "尝试 ping 域名确认网络", "检查系统 DNS 配置 / 代理设置"]} />
      )}
      {result.dns.ok && !result.tcp.ok && (
        <Advice title="TCP 连接被拒绝" items={["检查服务是否已启动", "检查端口是否监听", "检查防火墙规则", "内网环境确认是否在 VPN 内"]} />
      )}
      {result.tcp.ok && result.tls && !result.tls.ok && (
        <Advice title="TLS 证书错误" items={["确认证书是否过期", "内网自签名证书需在设置中关闭 SSL 验证", "检查 SNI / 服务器名称是否匹配"]} />
      )}
    </div>
  );
}

function RouteResultView({ result }: { result: RouteResult }) {
  return (
    <div className="space-y-3 animate-fade-in">
      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
        <div className="flex items-center justify-between">
          <div className="min-w-0">
            <div className="text-sm font-medium text-gray-800 dark:text-gray-200 font-mono truncate">{result.target}</div>
            <div className="text-[10px] text-gray-400 mt-0.5 flex items-center gap-1">
              {result.resolved_ip ? (
                <>
                  <span className="font-mono">解析: {result.resolved_ip}</span>
                  <span>· {result.reached ? "已到达目标" : "未到达目标"}</span>
                </>
              ) : (
                result.error
              )}
            </div>
          </div>
          <div className="text-right flex-shrink-0">
            <div className={cn("text-lg font-semibold", result.reached ? "text-emerald-600 dark:text-emerald-400" : "text-amber-600 dark:text-amber-400")}>
              {result.reached ? "可达" : "未达"}
            </div>
            <div className="text-[10px] text-gray-400">{result.total_ms}ms</div>
          </div>
        </div>
      </div>

      <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
        <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 font-mono">目标逐跳路径</div>
        {result.hops.length === 0 ? (
          <div className="text-xs text-gray-400">{result.error || "无可展示的跳点"}</div>
        ) : (
          <ol className="space-y-1">
            {result.hops.map((h, i) => (
              <li key={i} className="flex items-center gap-2 text-xs font-mono">
                <span className="w-6 text-gray-400 flex-shrink-0">{h.ttl}</span>
                <span className="text-right w-12 flex-shrink-0 text-gray-400">{h.ms}ms</span>
                {h.ok && h.ip ? (
                  <span className={cn("text-gray-800 dark:text-gray-200 truncate", h.reached && "text-emerald-600 dark:text-emerald-400 font-medium")}>{h.ip}</span>
                ) : (
                  <span className="text-gray-400">* 无响应{routeStatusLabel(h.status)}</span>
                )}
              </li>
            ))}
          </ol>
        )}
      </div>

      {result.hops.length > 0 && !result.reached && (
        <Advice title="未在最大 TTL 内到达目标" items={["部分三层设备会静默丢弃 ICMP（跳点显示 '*' 属正常）", "确认目标允许 ICMP 回显", "可用「直连诊断」确认目标 DNS/TCP 是否本身可达"]} />
      )}
    </div>
  );
}

function routeStatusLabel(s?: number) {
  if (!s) return "";
  if (s === 11010) return "（请求超时）";
  if (s === 11013) return "（TTL 超时）";
  if ([11002, 11003, 11017, 11018].includes(s)) return "（目标不可达）";
  return `（状态 ${s}）`;
}

function CertChainView({ chain }: { chain: CertInfo[] }) {
  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
      <div className="text-xs font-medium text-gray-500 dark:text-gray-400 mb-2 font-mono">证书链（{chain.length} 级）</div>
      <ol className="space-y-2">
        {chain.map((c, i) => {
          const expired = c.expired;
          return (
            <li key={i} className="rounded-xl border border-gray-100 dark:border-gray-800 p-3">
              <div className="flex items-start justify-between gap-2">
                <div className="min-w-0">
                  <div className="text-xs font-medium text-gray-800 dark:text-gray-200 font-mono truncate">
                    {c.subject}
                    <span className="ml-1.5 text-[10px] text-gray-400">（深度 {c.depth}）</span>
                    {c.is_ca && (
                      <span className="ml-1.5 inline-flex items-center px-1.5 py-0.5 rounded text-[9px] font-medium bg-indigo-50 dark:bg-indigo-900/30 text-indigo-600 dark:text-indigo-300">CA</span>
                    )}
                  </div>
                  <div className="text-[10px] text-gray-400 font-mono mt-0.5 truncate">签发: {c.issuer}</div>
                </div>
                <span
                  className={cn(
                    "inline-flex flex-shrink-0 items-center px-1.5 py-0.5 rounded text-[9px] font-medium",
                    expired
                      ? "bg-red-50 dark:bg-red-900/30 text-red-600 dark:text-red-400"
                      : "bg-emerald-50 dark:bg-emerald-900/30 text-emerald-600 dark:text-emerald-400",
                  )}
                >
                  {expired ? "已过期" : "有效"}
                </span>
              </div>
              <div className="text-[10px] text-gray-400 font-mono mt-1.5 flex items-center gap-2">
                <span>有效期 {c.not_before} ~ {c.not_after}</span>
                <span className="text-gray-300 dark:text-gray-600">|</span>
                <span>序列号 {c.serial}</span>
              </div>
            </li>
          );
        })}
      </ol>
    </div>
  );
}

function ProxyResult({ result, proxyUrl }: { result: ProxyDiagResult; proxyUrl: string }) {
  return (
    <div className="space-y-3 animate-fade-in">
      <SummaryCard title={`经 ${result.proxy_host}:${result.proxy_port} 探针`} sub={`目标 ${result.target_host} · ${result.scheme}`} totalMs={result.total_ms} />
      <DiagRow icon={<Network className="h-4 w-4" />} label="代理 TCP 可达" ok={result.proxy_tcp.ok} detail={result.proxy_tcp.error} ms={result.proxy_tcp.ms} hint={result.proxy_tcp.ok ? "代理可连接" : "代理不可达"} />
      <DiagRow icon={<Globe className="h-4 w-4" />} label="目标 DNS（本地对照）" ok={result.dns.ok} detail={result.dns.addresses.join(", ") || result.dns.error} ms={result.dns.ms} hint={result.dns.ok ? "解析成功" : "解析失败"} />
      {result.tls && (
        <DiagRow icon={<ShieldCheck className="h-4 w-4" />} label="CONNECT 隧道 + TLS" ok={result.tls.ok} detail={result.tls.peer_cert_subject || result.tls.error} ms={result.tls.ms} hint={result.tls.ok ? "隧道+握手成功" : "隧道/TLS 失败"} warn={!result.tls.ok} />
      )}
      {result.http && (
        <DiagRow icon={<Zap className="h-4 w-4" />} label="经代理 HTTP" ok={result.http.ok} detail={result.http.status ? `HTTP ${result.http.status}` : result.http.error} ms={result.http.ms} hint={result.http.status ? `状态码 ${result.http.status}` : "请求失败"} warn={result.http.status != null && result.http.status >= 400} />
      )}

      {!result.proxy_tcp.ok && (
        <Advice title="代理不可达" items={["确认代理软件（Clash/其它）已启动", "确认端口正确（如 7897/7890）", "确认代理地址格式，如 http://127.0.0.1:7897", "尝试在其它应用里验证该代理是否可用"]} />
      )}
      {result.proxy_tcp.ok && result.tls && !result.tls.ok && (
        <Advice title="代理未完成 CONNECT/TLS" items={["代理是否支持 HTTPS（需要 CONNECT 方法）", "目标是否在代理规则/白名单内", "企业代理可能需要认证（暂不支持）", "尝试直连诊断确认目标本身可达"]} />
      )}
      {result.proxy_tcp.ok && result.http && !result.http.ok && (!result.tls || result.tls.ok) && (
        <Advice title="经代理请求失败" items={["确认该代理能否访问目标（可能被规则拦截）", "目标若是内网地址，查看代理是否将其直连", "检查代理是否需要认证", "换目标或换代理后重试"]} />
      )}
    </div>
  );
}

function SummaryCard({ title, sub, totalMs }: { title: string; sub: string; totalMs: number }) {
  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-100 dark:border-gray-800 rounded-2xl p-4 shadow-sm">
      <div className="flex items-center justify-between">
        <div className="min-w-0">
          <div className="text-sm font-medium text-gray-800 dark:text-gray-200 font-mono truncate">{title}</div>
          <div className="text-[10px] text-gray-400 mt-0.5">{sub}</div>
        </div>
        <div className="text-right">
          <div className="text-lg font-semibold text-gray-900 dark:text-gray-100">{totalMs}ms</div>
          <div className="text-[10px] text-gray-400">总耗时</div>
        </div>
      </div>
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