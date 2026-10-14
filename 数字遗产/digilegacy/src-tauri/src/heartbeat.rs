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

#[derive(Default)]
struct HeartbeatConfig {
    enabled: bool,
    token: String,
    endpoint: String,
}

fn read_config() -> HeartbeatConfig {
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
    let output = std::process::Command::new("curl")
        .args([
            "-s", "-o", "/dev/null", "-m", &TIMEOUT_SECS.to_string(),
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
    if !cfg.enabled || cfg.token.is_empty() {
        let status = HeartbeatStatus {
            enabled: cfg.enabled,
            endpoint: cfg.endpoint,
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

    #[test]
    fn disabled_or_empty_token_never_sends() {
        std::env::remove_var("DMS_HEARTBEAT_ENABLED");
        let status = send_heartbeat();
        assert_eq!(status.last_result, "never");
        assert!(!status.enabled);
    }

    #[test]
    fn unreachable_server_tolerated() {
        std::env::set_var("DMS_HEARTBEAT_ENABLED", "1");
        std::env::set_var("DMS_HEARTBEAT_TOKEN", "test-token-123");
        std::env::set_var("DMS_HEARTBEAT_ENDPOINT", "https://127.0.0.1:9");
        let status = send_heartbeat();
        assert_eq!(status.last_result, "failed", "失联必须被容忍并记录，不得 panic");
        std::env::remove_var("DMS_HEARTBEAT_ENABLED");
        std::env::remove_var("DMS_HEARTBEAT_TOKEN");
        std::env::remove_var("DMS_HEARTBEAT_ENDPOINT");
    }

    #[test]
    fn status_defaults() {
        let s = current_status();
        assert!(matches!(s.last_result.as_str(), "never" | "ok" | "failed"));
    }
}
