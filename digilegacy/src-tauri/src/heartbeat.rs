//! 云端心跳：仅上报"我还安好"令牌，不携带任何用户数据（零内容设计）
//!
//! 服务器只需实现一个接口：
//!   POST {base}/heartbeat   body: {"token": "..."}   → 200
//! 服务器职责：记录该 token 最后心跳时间；到期未收到时由服务端定时任务
//! 按"token → 紧急联系人"的预注册映射发通知（映射也只含联系方式，无资产数据）。
//! 本地失联容忍：请求失败只记录状态，绝不阻塞或 panic。

use serde::Serialize;

const TIMEOUT_SECS: u64 = 8;

#[derive(Debug, Clone, Serialize)]
pub struct HeartbeatStatus {
    pub enabled: bool,
    pub endpoint: String,
    pub last_attempt: Option<String>,
    pub last_result: String, // "ok" | "failed" | "never"
}

impl HeartbeatStatus {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn last_seen_ok(&self) -> bool {
        self.last_result == "ok"
    }
}

#[derive(Default)]
pub(crate) struct HeartbeatConfig {
    enabled: bool,
    token: String,
    endpoint: String,
}

/// 读取心跳配置：本地库为权威来源（用户在界面里可配），
/// 环境变量仅作后备（便于自托管部署与自动化场景）。
/// 旧实现只认环境变量，普通用户装完应用后无法开启该功能。
fn read_config() -> HeartbeatConfig {
    let from_db =
        crate::db::get_heartbeat_config_db().unwrap_or((false, String::new(), String::new()));
    let (db_enabled, db_token, db_endpoint) = from_db;

    let enabled = if db_enabled {
        true
    } else {
        std::env::var("DMS_HEARTBEAT_ENABLED")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    };
    let token = if !db_token.is_empty() {
        db_token
    } else {
        std::env::var("DMS_HEARTBEAT_TOKEN").unwrap_or_default()
    };
    let endpoint = if !db_endpoint.is_empty() {
        db_endpoint
    } else {
        std::env::var("DMS_HEARTBEAT_ENDPOINT").unwrap_or_default()
    };
    HeartbeatConfig {
        enabled,
        token,
        endpoint,
    }
}

fn post_heartbeat(endpoint: &str, token: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    if token.is_empty() {
        return Err("心跳令牌为空".to_string());
    }
    if endpoint.trim().is_empty() {
        return Err("未配置心跳服务器地址".to_string());
    }
    let url = format!("{}/heartbeat", endpoint.trim_end_matches('/'));
    let body = format!(
        "{{\"token\":\"{}\"}}",
        token.replace('\\', "\\\\").replace('"', "\\\"")
    );
    // 丢弃响应体的目标：Unix 用 /dev/null，Windows 用 NUL
    let discard = if cfg!(windows) { "NUL" } else { "/dev/null" };
    // 请求体经标准输入传入（--data-binary @-）：令牌不出现在进程命令行参数里
    let mut child = Command::new("curl")
        .args([
            "-s",
            "-o",
            discard,
            "-m",
            &TIMEOUT_SECS.to_string(),
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data-binary",
            "@-",
            "--",
            &url,
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动 curl 失败: {}", e))?;
    {
        let mut stdin = child.stdin.take().ok_or("无法打开 curl 标准输入")?;
        stdin
            .write_all(body.as_bytes())
            .map_err(|e| format!("写入心跳内容失败: {}", e))?;
    } // drop stdin → curl 收到 EOF 后发送请求体

    let output = child
        .wait_with_output()
        .map_err(|e| format!("等待 curl 退出失败: {}", e))?;

    if !output.status.success() {
        // curl 退出码：7=连接失败 6=DNS 28=超时 → 视为失联，返回 Err
        return Err(format!(
            "心跳请求失败 (curl exit {})",
            output.status.code().unwrap_or(-1)
        ));
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
    guard.last_result = if ok {
        "ok".to_string()
    } else {
        "failed".to_string()
    };
}

/// 发送一次心跳（后台线程与手动测试共用）；永不 panic
pub fn send_heartbeat() -> HeartbeatStatus {
    let cfg = read_config();
    send_heartbeat_with(&cfg)
}

/// 可注入配置的心跳发送（便于测试，避免依赖进程级环境变量）
pub fn send_heartbeat_with(cfg: &HeartbeatConfig) -> HeartbeatStatus {
    // 端点为空表示尚未配置服务器：这不是「发送失败」，而是「没有可发往的目标」。
    if !cfg.enabled || cfg.token.is_empty() || cfg.endpoint.trim().is_empty() {
        let status = HeartbeatStatus {
            enabled: cfg.enabled && !cfg.token.is_empty() && !cfg.endpoint.trim().is_empty(),
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

    /// 测试串行锁：本模块的断言依赖全局 LAST_STATUS，避免与其他用例交叉污染。
    ///
    /// 必须复用 `vault::test_lock()` 而不是另起一把锁：`heartbeat_config_roundtrip_in_db`
    /// 会调用 `db::init_database_at` 改写全局 DB_PATH/CONN，而 db 模块的用例都持这把锁。
    /// 两把锁各管一半全局状态时会并行竞争（旧实现的隐患）。
    fn lock() -> std::sync::MutexGuard<'static, ()> {
        crate::vault::test_lock()
    }

    #[test]
    fn disabled_or_empty_token_never_sends() {
        let _g = lock();
        let cfg = HeartbeatConfig {
            enabled: false,
            token: String::new(),
            endpoint: String::new(),
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
            endpoint: String::new(),
        };
        assert_eq!(send_heartbeat_with(&cfg).last_result, "never");
    }

    /// 端点未配置时必须报 never 而不是 failed：
    /// 旧实现里默认端点是 example.com 保留域名，任何启用都会「永远失败」，
    /// 用户看到的是坏掉的功能而不是「尚未配置」。
    #[test]
    fn enabled_without_endpoint_reports_never_not_failed() {
        let _g = lock();
        let cfg = HeartbeatConfig {
            enabled: true,
            token: "tok".to_string(),
            endpoint: String::new(),
        };
        let status = send_heartbeat_with(&cfg);
        assert_eq!(status.last_result, "never");
        assert!(!status.enabled, "未配置端点不构成可用的心跳");
    }

    #[test]
    fn post_heartbeat_rejects_blank_endpoint() {
        assert!(post_heartbeat("", "tok").is_err());
        assert!(post_heartbeat("   ", "tok").is_err());
    }

    /// 配置持久化：写入后能原样读回，且 upsert 不会产生多行。
    #[test]
    fn heartbeat_config_roundtrip_in_db() {
        let _g = lock();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_hb_cfg_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);

        crate::db::init_database_at(path.clone()).unwrap();

        assert_eq!(
            crate::db::get_heartbeat_config_db().unwrap(),
            (false, String::new(), String::new())
        );

        crate::db::set_heartbeat_config_db(true, "tok-1", "https://hb.example.org").unwrap();
        assert_eq!(
            crate::db::get_heartbeat_config_db().unwrap(),
            (
                true,
                "tok-1".to_string(),
                "https://hb.example.org".to_string()
            )
        );

        crate::db::set_heartbeat_config_db(false, "tok-2", "").unwrap();
        assert_eq!(
            crate::db::get_heartbeat_config_db().unwrap(),
            (false, "tok-2".to_string(), String::new())
        );
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
        assert_eq!(
            status.last_result, "failed",
            "失联必须被容忍并记录，不得 panic"
        );
    }

    #[test]
    fn status_defaults() {
        let s = current_status();
        assert!(matches!(s.last_result.as_str(), "never" | "ok" | "failed"));
    }

    /// 起一个本地最小 HTTP 服务，验证心跳成功路径真的能发出并收到 200
    /// （此前的用例只覆盖了失败路径，成功路径无验证）。
    /// 端口绑定与 curl 子进程受系统负载/安全软件首扫影响，存在偶发超时，
    /// 故做最多 3 次有界重试，全部失败才判定不通过。
    #[test]
    fn reachable_local_server_records_ok() {
        let _g = lock();
        use std::io::{Read, Write};
        use std::net::TcpListener;

        let mut last_status = String::new();
        for _attempt in 0..3 {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind 本地端口失败");
            let port = listener.local_addr().unwrap().port();
            let handle = std::thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut buf = [0u8; 1024];
                    let _ = stream.read(&mut buf);
                    let _ = stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\n{\"ok\":true}");
                    let _ = stream.flush();
                }
            });

            let cfg = HeartbeatConfig {
                enabled: true,
                token: "local-test-token".to_string(),
                endpoint: format!("http://127.0.0.1:{}", port),
            };
            let status = send_heartbeat_with(&cfg);
            let _ = handle.join();
            if status.last_seen_ok() {
                return; // 成功路径已验证
            }
            last_status = status.last_result;
        }
        panic!("本地可达服务应返回 ok，实际 last_result={}", last_status);
    }
}
