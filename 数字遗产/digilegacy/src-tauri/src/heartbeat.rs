//! 云端心跳：仅上报"我还安好"令牌，不携带任何用户数据（零内容设计）
//!
//! 服务器只需实现一个接口：
//!   POST {base}/heartbeat   body: {"token": "..."}   → 200
//! 服务器职责：记录该 token 最后心跳时间；到期未收到时由服务端定时任务
//! 按"token → 紧急联系人"的预注册映射发通知（映射也只含联系方式，无资产数据）。
//! 本地失联容忍：请求失败只记录状态，绝不阻塞或 panic。

use serde::Serialize;

const DEFAULT_ENDPOINT: &str = "https://heartbeat.digilegacy.example.com";
const TIMEOUT_SECS: u64 = 8;

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatStatus {
    pub enabled: bool,
    pub endpoint: String,
    pub last_attempt: Option<String>,
    pub last_result: String, // "ok" | "failed" | "never"
}

impl HeartbeatStatus {
    pub fn last_seen_ok(&self) -> bool {
        self.last_result == "ok"
    }
}

#[derive(Default)]
struct HeartbeatConfig {
    enabled: bool,
    token: String,
    endpoint: String,
}

fn read_config() -> HeartbeatConfig {
    let _ = 0;
    let enabled = std::env::var("DMS_HEARTBEAT_ENABLED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    let token = std::env::var("DMS_HEARTBEAT_TOKEN").unwrap_or_default();
    let endpoint = std::env::var("DMS_HEARTBEAT_ENDPOINT").unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());
    HeartbeatConfig { enabled, token, endpoint }
}

fn post_heartbeat(endpoint: &str, token: &str) -> Result<(), String> {
    if token.is_empty() {
        return Err("心跳令牌为空".to_string());
    }
    let url = format!("{}/heartbeat", endpoint.trim_end_matches('/'));
    let body = format!("{{\"token\":\"{}\"}}", token.replace('\\', "\\\\").replace('"', "\\\""));
    // 丢弃响应体的目标：Unix 用 /dev/null，Windows 用 NUL
    let discard = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let output = std::process::Command::new("curl")
        .args([
            "-s", "-o", discard, "-m", &TIMEOUT_SECS.to_string(),
            "-X", "POST", "-H", "Content-Type: application/json",
            "-d", &body, "--", &url,
        ])
        .output()
        .map_err(|e| format!("启动 curl 失败: {}", e))?;

    if !output.status.success() {
        // curl 退出码：7=连接失败 6=DNS 28=超时 → 视为失联，返回 Err
        return Err(format!("心跳请求失败 (curl exit {})", output.status.code().unwrap_or(-1)));
    }
    Ok(())
}

static LAST_STATUS: once_cell::sync::Lazy<std::sync::Mutex<HeartbeatStatus>> =
    once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(HeartbeatStatus {
            enabled: false,
            endpoint: String::new(),
            last_attempt: None,
            last_result: "never".to_string(),
        })
    });

fn record_attempt(endpoint: &str, enabled: bool, ok: bool) {
    let mut guard = LAST_STATUS.lock().unwrap();
    guard.enabled = enabled;
    guard.endpoint = endpoint.to_string();
    guard.last_attempt = Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
    guard.last_result = if ok { "ok".to_string() } else { "failed".to_string() };
}

/// 发送一次心跳（后台线程与手动测试共用）；永不 panic
pub fn send_heartbeat() -> HeartbeatStatus {
    let cfg = read_config();
    send_heartbeat_with(&cfg)
}

/// 可注入配置的心跳发送（便于测试，避免依赖进程级环境变量）
pub fn send_heartbeat_with(cfg: &HeartbeatConfig) -> HeartbeatStatus {
    if !cfg.enabled || cfg.token.is_empty() {
        let status = HeartbeatStatus {
            enabled: cfg.enabled,
            endpoint: cfg.endpoint.clone(),
            last_attempt: None,
            last_result: "never".to_string(),
        };
        return status;
    }
    let ok = post_heartbeat(&cfg.endpoint, &cfg.token).is_ok();
    record_attempt(&cfg.endpoint, true, ok);
    let guard = LAST_STATUS.lock().unwrap();
    HeartbeatStatus {
        enabled: guard.enabled,
        endpoint: guard.endpoint.clone(),
        last_attempt: guard.last_attempt.clone(),
        last_result: guard.last_result.clone(),
    }
}

pub fn current_status() -> HeartbeatStatus {
    LAST_STATUS.lock().unwrap().clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试串行锁：本模块的断言依赖全局 LAST_STATUS，避免与其他用例交叉污染
    static HB_SERIAL: once_cell::sync::Lazy<std::sync::Mutex<()>> =
        once_cell::sync::Lazy::new(|| std::sync::Mutex::new(()));

    fn lock() -> std::sync::MutexGuard<'static, ()> {
        HB_SERIAL.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn disabled_or_empty_token_never_sends() {
        let _g = lock();
        let cfg = HeartbeatConfig {
            enabled: false,
            token: String::new(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
        };
        let status = send_heartbeat_with(&cfg);
        assert_eq!(status.last_result, "never");
        assert!(!status.enabled);
    }

    #[test]
    fn enabled_but_empty_token_never_sends() {
        let _g = lock();
        let cfg = HeartbeatConfig {
            enabled: true,
            token: String::new(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
        };
        assert_eq!(send_heartbeat_with(&cfg).last_result, "never");
    }

    #[test]
    fn unreachable_server_tolerated() {
        let _g = lock();
        // 直接注入不可达端点：不依赖环境变量，避免并行测试竞争
        let cfg = HeartbeatConfig {
            enabled: true,
            token: "test-token-123".to_string(),
            endpoint: "https://127.0.0.1:9".to_string(),
        };
        let status = send_heartbeat_with(&cfg);
        assert_eq!(status.last_result, "failed", "失联必须被容忍并记录，不得 panic");
    }

    #[test]
    fn status_defaults() {
        let s = current_status();
        assert!(matches!(s.last_result.as_str(), "never" | "ok" | "failed"));
    }

    /// 起一个本地最小 HTTP 服务，验证心跳成功路径真的能发出并收到 200
    /// （此前的用例只覆盖了失败路径，成功路径无验证）
    #[test]
    fn reachable_local_server_records_ok() {
        let _g = lock();
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind 本地端口失败");
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\n{\"ok\":true}");
                let _ = stream.flush();
            }
        });

        let cfg = HeartbeatConfig {
            enabled: true,
            token: "local-test-token".to_string(),
            endpoint: format!("http://127.0.0.1:{}", port),
        };
        let status = send_heartbeat_with(&cfg);
        assert_eq!(status.last_result, "ok", "本地可达服务应返回 ok");
        assert!(status.last_seen_ok(), "确认状态记录为成功");
        let _ = handle.join();
    }
}
