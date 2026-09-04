//! 路由追踪（Route Diagnostic）
//!
//! 使用 Windows ICMP API（IcmpSendEcho）做逐跳路由探测（tracert 同款）：
//! 对目标 IPv4 依次递增 TTL 发送 ICMP echo，捕获 `TTL 超时` 的中间路由器与最终回显。
//! IcmpSendEcho 属于 IpHelper，**无需管理员权限**。

use std::net::Ipv4Addr;
use std::time::Instant;

use serde::Serialize;

use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::NetworkManagement::IpHelper::{
    IcmpCloseHandle, IcmpCreateFile, IcmpSendEcho, ICMP_ECHO_REPLY, IP_OPTION_INFORMATION,
    IP_REQ_TIMED_OUT, IP_SUCCESS, IP_TTL_EXPIRED_TRANSIT,
};

/// 单个跳点的探测结果
#[derive(Debug, Clone, Serialize)]
pub struct RouteHop {
    pub ttl: u8,
    /// 该跳响应的 IP（中间路由器或最终目标）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,
    pub ms: u64,
    pub ok: bool,
    /// 到达最终目标（Traceroute 终点的回显回复）
    pub reached: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u32>,
}

/// 路由追踪结果
#[derive(Debug, Clone, Serialize)]
pub struct RouteResult {
    pub target: String,
    pub resolved_ip: Option<String>,
    pub hops: Vec<RouteHop>,
    /// 是否在最大 TTL 内到达目标
    pub reached: bool,
    pub max_ttl: u8,
    pub total_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

const MAX_TTL: u8 = 30;
const PER_HOP_TIMEOUT_MS: u32 = 900;
/// 连续无响应的跳点数达到该值则提前停止（CGNAT / 静默丢弃段可能跨越多跳，故取较大值）
const MAX_CONSECUTIVE_TIMEOUTS: usize = 8;
const REQ_BUFFER: &[u8] = b"zeroapi-route-probe";
const IP_DEST_UNREACHABLE: [u32; 4] = [11_002, 11_003, 11_017, 11_018];

/// 对目标执行逐跳路由追踪。阻塞式 ICMP 调用放在 `spawn_blocking`，避免卡住事件循环。
pub async fn traceroute(target: &str) -> RouteResult {
    let trimmed = target.trim().to_string();
    let fallback_target = trimmed.clone();
    match tokio::task::spawn_blocking(move || traceroute_sync(&trimmed)).await {
        Ok(r) => r,
        Err(_) => RouteResult {
            target: fallback_target,
            resolved_ip: None,
            hops: Vec::new(),
            reached: false,
            max_ttl: MAX_TTL,
            total_ms: 0,
            error: Some("路由追踪任务异常终止".to_string()),
        },
    }
}

fn traceroute_sync(target: &str) -> RouteResult {
    let start = Instant::now();
    let host = host_of(target);

    // 只支持 IPv4：先解析目标主机为 IPv4 地址
    let resolved = match resolve_ipv4(&host) {
        Some(ip) => ip,
        None => {
            return RouteResult {
                target: target.to_string(),
                resolved_ip: None,
                hops: Vec::new(),
                reached: false,
                max_ttl: MAX_TTL,
                total_ms: 0,
                error: Some("无法解析目标为 IPv4 地址（当前仅支持 IPv4 路由追踪）".to_string()),
            }
        }
    };

    let icmp = unsafe { IcmpCreateFile() };
    if icmp.is_null() || icmp == INVALID_HANDLE_VALUE {
        return RouteResult {
            target: target.to_string(),
            resolved_ip: Some(resolved.to_string()),
            hops: Vec::new(),
            reached: false,
            max_ttl: MAX_TTL,
            total_ms: 0,
            error: Some(format!("创建 ICMP 句柄失败（地址 {}）", resolved)),
        };
    }

    let dest = ipv4_to_network_u32(resolved);
    let mut hops = Vec::new();
    let mut reached = false;
    let mut consecutive_timeouts = 0usize;

    for ttl in 1..=MAX_TTL {
        match probe_hop(icmp, dest, ttl) {
            ProbeHop::Reply(ms, hop_ip) => {
                consecutive_timeouts = 0;
                let finished = hop_ip == resolved;
                reached = finished;
                hops.push(RouteHop {
                    ttl,
                    ip: Some(hop_ip.to_string()),
                    ms,
                    ok: true,
                    reached: finished,
                    status: Some(IP_SUCCESS),
                });
                if finished {
                    break;
                }
            }
            ProbeHop::TtlExpired(ms, router_ip) => {
                consecutive_timeouts = 0;
                hops.push(RouteHop {
                    ttl,
                    ip: Some(router_ip.to_string()),
                    ms,
                    ok: true,
                    reached: false,
                    status: Some(IP_TTL_EXPIRED_TRANSIT),
                });
            }
            ProbeHop::Timeout => {
                consecutive_timeouts += 1;
                hops.push(RouteHop {
                    ttl,
                    ip: None,
                    ms: 0,
                    ok: false,
                    reached: false,
                    status: Some(IP_REQ_TIMED_OUT),
                });
                if consecutive_timeouts >= MAX_CONSECUTIVE_TIMEOUTS {
                    break;
                }
            }
            ProbeHop::Unreachable(ms, status) => {
                consecutive_timeouts = 0;
                hops.push(RouteHop {
                    ttl,
                    ip: None,
                    ms,
                    ok: false,
                    reached: false,
                    status: Some(status),
                });
                break;
            }
        }
    }

    unsafe { IcmpCloseHandle(icmp) };

    RouteResult {
        target: target.to_string(),
        resolved_ip: Some(resolved.to_string()),
        hops,
        reached,
        max_ttl: MAX_TTL,
        total_ms: start.elapsed().as_millis() as u64,
        error: None,
    }
}

enum ProbeHop {
    /// 最终回显（到达目标）
    Reply(u64, Ipv4Addr),
    /// 中间路由器 TTL 超时
    TtlExpired(u64, Ipv4Addr),
    /// 请求超时（该跳无应答）
    Timeout,
    /// 目标不可达（终止）
    Unreachable(u64, u32),
}

fn probe_hop(icmp: windows_sys::Win32::Foundation::HANDLE, dest: u32, ttl: u8) -> ProbeHop {
    let options = IP_OPTION_INFORMATION {
        Ttl: ttl,
        Tos: 0,
        Flags: 0,
        OptionsSize: 0,
        OptionsData: std::ptr::null_mut(),
    };

    let mut reply = vec![0u8; 512 + REQ_BUFFER.len()];

    let count = unsafe {
        IcmpSendEcho(
            icmp,
            dest,
            REQ_BUFFER.as_ptr() as *const std::ffi::c_void,
            REQ_BUFFER.len() as u16,
            &options,
            reply.as_mut_ptr() as *mut std::ffi::c_void,
            reply.len() as u32,
            PER_HOP_TIMEOUT_MS,
        )
    };

    if count == 0 {
        return ProbeHop::Timeout;
    }

    // 紧凑拷贝首条 ICMP_ECHO_REPLY 字段，避免对齐问题
    let (addr, status, rtt) = unsafe {
        let r = reply.as_ptr() as *const ICMP_ECHO_REPLY;
        (
            std::ptr::read_unaligned(std::ptr::addr_of!((*r).Address)),
            std::ptr::read_unaligned(std::ptr::addr_of!((*r).Status)),
            std::ptr::read_unaligned(std::ptr::addr_of!((*r).RoundTripTime)),
        )
    };

    let ms = rtt as u64;
    match status {
        IP_SUCCESS => ProbeHop::Reply(ms, ipv4_from_network_u32(addr)),
        IP_TTL_EXPIRED_TRANSIT => ProbeHop::TtlExpired(ms, ipv4_from_network_u32(addr)),
        s if IP_DEST_UNREACHABLE.contains(&s) => ProbeHop::Unreachable(ms, s),
        _ => ProbeHop::Timeout,
    }
}

/// 从 target（可为完整 URL）提取 host
fn host_of(target: &str) -> String {
    if let Ok(url) = url::Url::parse(target) {
        if let Some(h) = url.host_str() {
            return h.to_string();
        }
    }
    target.to_string()
}

/// 解析 host 为 IPv4（lookup 可能含 IPv6，只取第一个 IPv4）
fn resolve_ipv4(host: &str) -> Option<Ipv4Addr> {
    if let Ok(parsed) = host.parse::<Ipv4Addr>() {
        return Some(parsed);
    }
    use std::net::ToSocketAddrs;
    (host, 0u16).to_socket_addrs().ok()?.map(|s| s.ip()).find_map(|ip| match ip {
        std::net::IpAddr::V4(v4) => Some(v4),
        _ => None,
    })
}

fn ipv4_to_network_u32(ip: Ipv4Addr) -> u32 {
    u32::from_be_bytes(ip.octets())
}

/// IpHelper 回包中的地址以本机字节序存放：u32 原值直接取其内存字节即为点分四段的网络地址。
fn ipv4_from_network_u32(net: u32) -> Ipv4Addr {
    Ipv4Addr::from(net.to_ne_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dest_address_uses_network_byte_order() {
        let ip: Ipv4Addr = "1.2.3.4".parse().unwrap();
        assert_eq!(ipv4_to_network_u32(ip), 0x0102_0304);
    }

    #[test]
    fn reply_address_reads_le_host_bytes() {
        // IpHelper 回包地址以本机字节序 u32 存放：原值内存字节即实际点分段
        assert_eq!(ipv4_from_network_u32(0x0101_A8C0u32), "192.168.1.1".parse::<Ipv4Addr>().unwrap());
        assert_eq!(ipv4_from_network_u32(0x0403_0201u32), "1.2.3.4".parse::<Ipv4Addr>().unwrap());
    }

    #[test]
    fn host_of_parses_url_and_bare() {
        assert_eq!(host_of("https://api.example.com/v1"), "api.example.com");
        assert_eq!(host_of("10.0.0.1"), "10.0.0.1");
    }

    #[test]
    fn resolve_ipv4_prefers_ipv4() {
        assert_eq!(resolve_ipv4("8.8.8.8"), Some("8.8.8.8".parse().unwrap()));
    }

    #[test]
    #[ignore]
    fn live_traceroute_baidu() {
        let r = traceroute_sync("https://www.baidu.com");
        println!("target={} resolved={:?} reached={} hops={} err={:?}", r.target, r.resolved_ip, r.reached, r.hops.len(), r.error);
        for h in &r.hops {
            println!("  ttl={} ip={:?} ok={} ms={} reached={} status={:?}", h.ttl, h.ip, h.ok, h.ms, h.reached, h.status);
        }
        assert!(r.hops.iter().any(|h| h.reached), "应存在到达目标的跳点");
    }
}