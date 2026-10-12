//! DMS 通知链路：到期检测 → 分级 → 本地发件箱 → (可选) SMTP 投递
//!
//! 离线优先设计：无网络 / 无 SMTP 凭据时写入本地发件箱，不阻塞主流程。
//! 配置方式（环境变量，全部可选）：
//!   DMS_SMTP_HOST / DMS_SMTP_PORT / SMTP_USER / SMTP_PASS / SMTP_FROM

use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use chrono::{Local, NaiveDateTime};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::db;

#[derive(Debug, Clone, Serialize)]
pub struct DmsNotification {
    pub stage: i64,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub sent_at: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DmsEvaluation {
    pub enabled: bool,
    pub overdue: bool,
    pub current_stage: i64,
    pub days_overdue: i64,
    pub notifications: Vec<DmsNotification>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct DmsConfig {
    enabled: bool,
    interval_days: i64,
    stage1_wait_days: i64,
    stage2_wait_days: i64,
    stage3_wait_days: i64,
    last_check_in: Option<String>,
}

fn read_config() -> Result<DmsConfig, String> {
    let conn = db::get_conn()?;
    let enabled: i64 = conn
        .query_row("SELECT enabled FROM dms_config WHERE id=1", [], |r| r.get(0))
        .unwrap_or(0);
    let interval: i64 = conn
        .query_row("SELECT check_in_interval_days FROM dms_config WHERE id=1", [], |r| r.get(0))
        .unwrap_or(30);
    let s1: i64 = conn
        .query_row("SELECT stage1_wait_days FROM dms_config WHERE id=1", [], |r| r.get(0))
        .unwrap_or(3);
    let s2: i64 = conn
        .query_row("SELECT stage2_wait_days FROM dms_config WHERE id=1", [], |r| r.get(0))
        .unwrap_or(7);
    let s3: i64 = conn
        .query_row("SELECT stage3_wait_days FROM dms_config WHERE id=1", [], |r| r.get(0))
        .unwrap_or(14);
    let last: Option<String> = conn
        .query_row("SELECT last_check_in FROM dms_config WHERE id=1", [], |r| r.get(0))
        .ok();
    Ok(DmsConfig {
        enabled: enabled != 0,
        interval_days: interval,
        stage1_wait_days: s1,
        stage2_wait_days: s2,
        stage3_wait_days: s3,
        last_check_in: last,
    })
}

fn days_since_last_check_in(last: &str) -> Option<i64> {
    let dt = NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S").ok()?;
    let now = Local::now().naive_local();
    Some(now.signed_duration_since(dt).num_days())
}

fn compute_stage(overdue_days: i64, cfg: &DmsConfig) -> i64 {
    if overdue_days <= 0 {
        0
    } else if overdue_days <= cfg.stage1_wait_days {
        1
    } else if overdue_days <= cfg.stage1_wait_days + cfg.stage2_wait_days {
        2
    } else {
        3
    }
}

fn outbox_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?
        .join("dms_outbox");
    fs::create_dir_all(&dir).map_err(|e| format!("创建发件箱目录失败: {}", e))?;
    Ok(dir.join(format!("{}.jsonl", Local::now().format("%Y-%m-%d"))))
}

fn already_notified(path: &std::path::Path, stage: i64, recipient: &str) -> bool {
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content.lines().any(|line| {
        line.contains(&format!("\"stage\":{}", stage)) && line.contains(&format!("\"recipient\":\"{}\"", recipient))
    })
}

fn append_outbox(path: &std::path::Path, entry: &DmsNotification) -> Result<(), String> {
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开发件箱失败: {}", e))?;
    let line = serde_json::to_string(entry).map_err(|e| format!("序列化通知失败: {}", e))?;
    writeln!(file, "{}", line).map_err(|e| format!("写入发件箱失败: {}", e))?;
    Ok(())
}

fn try_smtp(host: &str, port: u16, from: &str, to: &str, subject: &str, body: &str) -> Result<(), String> {
    let mut stream = TcpStream::connect((host, port))
        .map_err(|e| format!("SMTP 连接失败: {}", e))?;
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

    let mut buf = String::new();
    read_smtp_line(&mut stream, &mut buf)?;
    if !buf.starts_with("220") {
        return Err(format!("SMTP 握手异常: {}", buf.trim()));
    }
    smtp_cmd(&mut stream, &mut buf, &format!("EHLO {}", host))?;
    smtp_cmd(&mut stream, &mut buf, &format!("MAIL FROM:<{}>", from))?;
    smtp_cmd(&mut stream, &mut buf, &format!("RCPT TO:<{}>", to))?;
    smtp_cmd(&mut stream, &mut buf, "DATA")?;

    let msg = format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}\r\n.\r\n",
        from, to, subject, body
    );
    stream.write_all(msg.as_bytes()).map_err(|e| format!("发送邮件内容失败: {}", e))?;
    stream.flush().ok();
    read_smtp_line(&mut stream, &mut buf)?;
    if !buf.starts_with("250") {
        return Err(format!("邮件被拒收: {}", buf.trim()));
    }
    let _ = smtp_cmd(&mut stream, &mut buf, "QUIT");
    Ok(())
}

fn smtp_cmd(stream: &mut TcpStream, buf: &mut String, cmd: &str) -> Result<(), String> {
    stream.write_all(format!("{}\r\n", cmd).as_bytes()).map_err(|e| format!("SMTP 命令发送失败: {}", e))?;
    stream.flush().ok();
    read_smtp_line(stream, buf)?;
    if !buf.starts_with("250") && !buf.starts_with("354") {
        return Err(format!("SMTP 命令被拒: {} -> {}", cmd, buf.trim()));
    }
    Ok(())
}

fn read_smtp_line(stream: &mut TcpStream, buf: &mut String) -> Result<(), String> {
    buf.clear();
    let mut byte = [0u8; 1];
    loop {
        let n = stream.read(&mut byte).map_err(|e| format!("SMTP 读取失败: {}", e))?;
        if n == 0 {
            return Err("SMTP 连接已关闭".to_string());
        }
        if byte[0] == b'\n' {
            break;
        }
        if byte[0] != b'\r' {
            buf.push(byte[0] as char);
        }
    }
    Ok(())
}

fn dispatch(
    app_handle: &AppHandle,
    stage: i64,
    recipient: &str,
    subject: &str,
    body: &str,
) -> Result<DmsNotification, String> {
    let path = outbox_path(app_handle)?;
    if already_notified(&path, stage, recipient) {
        return Ok(DmsNotification {
            stage,
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            channel: "dedup".to_string(),
        });
    }

    let channel = if let (Ok(host), Ok(port), Ok(from)) = (
        std::env::var("DMS_SMTP_HOST"),
        std::env::var("DMS_SMTP_PORT"),
        std::env::var("SMTP_FROM"),
    ) {
        let port: u16 = port.parse().unwrap_or(25);
        match try_smtp(&host, port, &from, recipient, subject, body) {
            Ok(()) => "smtp".to_string(),
            Err(_) => {
                let _ = append_outbox(&path, &DmsNotification {
                    stage,
                    recipient: recipient.to_string(),
                    subject: subject.to_string(),
                    body: body.to_string(),
                    sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    channel: "local_outbox".to_string(),
                });
                "local_outbox".to_string()
            }
        }
    } else {
        let _ = append_outbox(&path, &DmsNotification {
            stage,
            recipient: recipient.to_string(),
            subject: subject.to_string(),
            body: body.to_string(),
            sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            channel: "local_outbox".to_string(),
        });
        "local_outbox".to_string()
    };

    Ok(DmsNotification {
        stage,
        recipient: recipient.to_string(),
        subject: subject.to_string(),
        body: body.to_string(),
        sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        channel,
    })
}

#[tauri::command]
pub fn evaluate_dms(app_handle: AppHandle) -> Result<DmsEvaluation, String> {
    let cfg = read_config()?;

    if !cfg.enabled {
        return Ok(DmsEvaluation {
            enabled: false,
            overdue: false,
            current_stage: 0,
            days_overdue: 0,
            notifications: vec![],
        });
    }

    let (overdue_days, current_stage) = match &cfg.last_check_in {
        Some(last) => match days_since_last_check_in(last) {
            Some(days) => {
                let overdue = days - cfg.interval_days;
                (overdue.max(0), compute_stage(overdue, &cfg))
            }
            None => (0, 0),
        },
        None => (0, 0),
    };

    let overdue = current_stage > 0;
    let user = db::get_user_profile_db()?.unwrap_or_default();
    let mut notifications = Vec::new();

    if overdue {
        let name = if user.full_name.is_empty() { "用户".to_string() } else { user.full_name.clone() };

        if current_stage >= 1 {
            if let Ok(n) = dispatch(
                &app_handle,
                1,
                &user.email,
                &format!("【归零】{}，请确认你的安全状态", name),
                &format!(
                    "你好 {}，\n\n你已超过 {} 天未确认安全状态。请打开「归零」应用点击「我还安好」。\n若 {} 天后仍未确认，系统将通知你的紧急联系人。\n\n—— 归零 DigiLegacy",
                    name, overdue_days, cfg.stage1_wait_days
                ),
            ) {
                notifications.push(n);
            }
        }

        if current_stage >= 2 {
            if let Ok(n) = dispatch(
                &app_handle,
                2,
                &user.emergency_contact_phone,
                &format!("【归零】请确认 {} 的安全状态", name),
                &format!(
                    "你好，\n\n{} 已超过 {} 天未确认安全状态。请尝试联系本人确认是否安好。\n若确认失联，请联系遗产执行人。\n\n—— 归零 DigiLegacy",
                    name,
                    overdue_days
                ),
            ) {
                notifications.push(n);
            }
        }

        if current_stage >= 3 {
            if let Ok(n) = dispatch(
                &app_handle,
                3,
                &user.heir_primary_phone,
                &format!("【归零】{} 的数字遗产密钥信封", name),
                &format!(
                    "你好 {}，\n\n很遗憾地通知你，{} 已长时间未确认安全状态。\n请凭死亡证明与本邮件，前往「归零」应用继承人向导页面解锁数字遗产。\n\n—— 归零 DigiLegacy",
                    user.heir_primary_name, name
                ),
            ) {
                notifications.push(n);
            }
        }
    }

    Ok(DmsEvaluation {
        enabled: true,
        overdue,
        current_stage,
        days_overdue: overdue_days,
        notifications,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(s1: i64, s2: i64, s3: i64) -> DmsConfig {
        DmsConfig {
            enabled: true,
            interval_days: 30,
            stage1_wait_days: s1,
            stage2_wait_days: s2,
            stage3_wait_days: s3,
            last_check_in: None,
        }
    }

    #[test]
    fn stage_0_when_not_overdue() {
        let cfg = test_config(3, 7, 14);
        assert_eq!(compute_stage(0, &cfg), 0);
        assert_eq!(compute_stage(-5, &cfg), 0);
    }

    #[test]
    fn stage_1_within_first_window() {
        let cfg = test_config(3, 7, 14);
        assert_eq!(compute_stage(1, &cfg), 1);
        assert_eq!(compute_stage(3, &cfg), 1);
    }

    #[test]
    fn stage_2_within_second_window() {
        let cfg = test_config(3, 7, 14);
        assert_eq!(compute_stage(4, &cfg), 2);
        assert_eq!(compute_stage(10, &cfg), 2);
    }

    #[test]
    fn stage_3_beyond_all_windows() {
        let cfg = test_config(3, 7, 14);
        assert_eq!(compute_stage(11, &cfg), 3);
        assert_eq!(compute_stage(100, &cfg), 3);
    }

    #[test]
    fn days_since_last_check_in_parses_correctly() {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        assert_eq!(days_since_last_check_in(&now), Some(0));

        let past = (Local::now() - chrono::Duration::days(10))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        assert_eq!(days_since_last_check_in(&past), Some(10));
    }

    #[test]
    fn days_since_rejects_invalid_format() {
        assert_eq!(days_since_last_check_in("not-a-date"), None);
        assert_eq!(days_since_last_check_in(""), None);
    }
}