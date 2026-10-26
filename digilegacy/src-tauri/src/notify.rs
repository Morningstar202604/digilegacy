//! DMS 通知链路：到期检测 → 分级 → 本地发件箱 → (可选) SMTP 投递
//!
//! 离线优先设计：无网络 / 无 SMTP 凭据时写入本地发件箱，不阻塞主流程。
//! 邮件投递复用系统 curl（成熟的 SMTP 实现，自带 STARTTLS 升级与 AUTH），
//! 不再自行实现 SMTP 协议。配置方式（环境变量，全部可选）：
//!   DMS_SMTP_HOST / DMS_SMTP_PORT / SMTP_FROM      服务器与发件人
//!   DMS_SMTP_USER / DMS_SMTP_PASS                  SMTP 登录凭据（建议启用）
//!   DMS_SMTP_ALLOW_INSECURE=1                      允许无 STARTTLS 的明文中继（默认拒绝）

use std::fs;
use std::path::Path;

use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::db;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmsNotification {
    pub stage: i64,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub sent_at: String,
    pub channel: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DmsStatus {
    pub enabled: bool,
    pub overdue: bool,
    pub current_stage: i64,
    pub days_overdue: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DmsEvaluation {
    pub enabled: bool,
    pub overdue: bool,
    pub current_stage: i64,
    pub days_overdue: i64,
    pub notifications: Vec<DmsNotification>,
}

/// 只读状态快照：不发邮件、不写发件箱。
/// 界面展示与后台投递必须分开——旧实现让前端直接调用 `evaluate_dms`，
/// 打开仪表板就等于执行一轮通知投递（可能真的发出 SMTP 邮件）。
pub fn status_snapshot() -> Result<DmsStatus, String> {
    let cfg = read_config()?;
    if !cfg.enabled {
        return Ok(DmsStatus {
            enabled: false,
            overdue: false,
            current_stage: 0,
            days_overdue: 0,
        });
    }
    let (overdue_days, current_stage) = stage_snapshot(&cfg);
    Ok(DmsStatus {
        enabled: true,
        overdue: current_stage > 0,
        current_stage,
        days_overdue: overdue_days,
    })
}

fn stage_snapshot(cfg: &DmsConfig) -> (i64, i64) {
    match &cfg.last_check_in {
        Some(last) => match days_since_last_check_in(last) {
            Some(days) => {
                let overdue = (days - cfg.interval_days).max(0);
                (overdue, compute_stage(overdue, cfg))
            }
            None => (0, 0),
        },
        None => (0, 0),
    }
}

/// 读取本地发件箱历史（按日期倒序，最多 `limit` 条）。
/// 界面的「通知记录」此前只显示当前这一轮评估的返回值，历史投递完全不可见。
pub fn outbox_history(
    app_handle: &AppHandle,
    limit: usize,
) -> Result<Vec<DmsNotification>, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?
        .join("dms_outbox");
    let mut files: Vec<std::path::PathBuf> = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("jsonl"))
            .collect(),
        Err(_) => return Ok(Vec::new()),
    };
    files.sort();
    files.reverse();

    let mut out = Vec::new();
    for file in files {
        if out.len() >= limit {
            break;
        }
        append_history_from(&file, limit, &mut out);
    }
    Ok(out)
}

fn append_history_from(path: &Path, limit: usize, out: &mut Vec<DmsNotification>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for line in content.lines().rev() {
        if out.len() >= limit {
            return;
        }
        if let Ok(entry) = serde_json::from_str::<DmsNotification>(line) {
            out.push(entry);
        }
    }
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
    let guard = db::get_conn()?;
    let conn = guard.as_ref().unwrap();
    let enabled: i64 = conn
        .query_row("SELECT enabled FROM dms_config WHERE id=1", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    let interval: i64 = conn
        .query_row(
            "SELECT check_in_interval_days FROM dms_config WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(30);
    let s1: i64 = conn
        .query_row(
            "SELECT stage1_wait_days FROM dms_config WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(3);
    let s2: i64 = conn
        .query_row(
            "SELECT stage2_wait_days FROM dms_config WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(7);
    let s3: i64 = conn
        .query_row(
            "SELECT stage3_wait_days FROM dms_config WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or(14);
    let last: Option<String> = conn
        .query_row("SELECT last_check_in FROM dms_config WHERE id=1", [], |r| {
            r.get(0)
        })
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
        line.contains(&format!("\"stage\":{}", stage))
            && line.contains(&format!("\"recipient\":\"{}\"", recipient))
    })
}

fn append_outbox(path: &std::path::Path, entry: &DmsNotification) -> Result<(), String> {
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("打开发件箱失败: {}", e))?;
    let line = serde_json::to_string(entry).map_err(|e| format!("序列化通知失败: {}", e))?;
    writeln!(file, "{}", line).map_err(|e| format!("写入发件箱失败: {}", e))?;
    Ok(())
}

fn try_smtp(
    host: &str,
    port: u16,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let host = host.trim();
    let url = format!("smtp://{}:{}", host, port);
    let mut cmd = Command::new("curl");
    cmd.args([
        "-s",
        "-S",
        "-m",
        "20",
        "--url",
        &url,
        "--mail-from",
        from,
        "--mail-rcpt",
        to,
        "--upload-file",
        "-",
    ]);
    // 默认要求 STARTTLS（curl --ssl-reqd：服务器不提供即中止，拒绝明文投递）
    if std::env::var("DMS_SMTP_ALLOW_INSECURE")
        .map(|v| v != "1")
        .unwrap_or(true)
    {
        cmd.arg("--ssl-reqd");
    }
    // 凭据经临时 netrc 文件传入：密码不出现在命令行参数里（同机进程可枚举 argv）
    let netrc_path = build_smtp_netrc(host);
    if let Ok(Some(path)) = &netrc_path {
        cmd.arg("--netrc")
            .arg("--netrc-optional")
            .arg("--netrc-file")
            .arg(path);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    let spawn_result = cmd.spawn();
    let mut child = match spawn_result {
        Ok(c) => c,
        Err(e) => {
            if let Ok(Some(p)) = &netrc_path {
                let _ = std::fs::remove_file(p);
            }
            return Err(format!("启动 curl 失败: {}", e));
        }
    };
    let write_result = (|| -> Result<(), String> {
        let mut stdin = child.stdin.take().ok_or("无法打开 curl 标准输入")?;
        stdin
            .write_all(smtp_message(from, to, subject, body).as_bytes())
            .map_err(|e| format!("写入邮件内容失败: {}", e))?;
        Ok(())
    })();
    if let Err(e) = write_result {
        let _ = child.kill();
        if let Ok(Some(p)) = &netrc_path {
            let _ = std::fs::remove_file(p);
        }
        return Err(e);
    } // drop stdin → curl 收到 EOF 后自行追加终止符并结束 DATA

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            if let Ok(Some(p)) = &netrc_path {
                let _ = std::fs::remove_file(p);
            }
            return Err(format!("等待 curl 退出失败: {}", e));
        }
    };
    if let Ok(Some(p)) = &netrc_path {
        let _ = std::fs::remove_file(p); // 凭据文件用完即删
    }
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "邮件投递失败 (curl exit {}): {}",
            output.status.code().unwrap_or(-1),
            err.trim()
        ));
    }
    Ok(())
}

/// 组装 netrc 单行凭据（供 curl --netrc-file 使用；netrc 无引号语法，凭据含空白时不适配，
/// 此时投递失败并降级到本地发件箱，属可接受降级）
fn netrc_entry(host: &str, user: &str, pass: &str) -> String {
    format!("machine {} login {} password {}\n", host, user, pass)
}

/// 若配置了 SMTP 凭据环境变量则生成临时 netrc 文件；调用方负责在 curl 结束后删除
fn build_smtp_netrc(host: &str) -> Result<Option<std::path::PathBuf>, String> {
    let (Ok(user), Ok(pass)) = (
        std::env::var("DMS_SMTP_USER"),
        std::env::var("DMS_SMTP_PASS"),
    ) else {
        return Ok(None);
    };
    if user.is_empty() || pass.is_empty() {
        return Ok(None);
    }
    let path = std::env::temp_dir().join(format!(
        "dms_netrc_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    // 该文件含明文 SMTP 密码：必须以仅本人可读的权限创建。
    crate::fsutil::write_private(&path, netrc_entry(host, &user, &pass).as_bytes())
        .map_err(|e| format!("写入 SMTP 凭据文件失败: {}", e))?;
    Ok(Some(path))
}

/// 邮件头净化：剥离 CR/LF，阻断 SMTP 头注入（头部拆分可注入 Bcc 等恶意头）
fn sanitize_header(value: &str) -> String {
    value
        .chars()
        .map(|c| if c == '\r' || c == '\n' { ' ' } else { c })
        .collect()
}

/// DATA 正文 dot-stuffing：以 . 开头的行前额外补一个点，避免提前终止 DATA；
/// 同时统一行尾为 CRLF（RFC 5321 要求）
fn dot_stuff_body(body: &str) -> String {
    body.split('\n')
        .map(|line| {
            let line = line.strip_suffix('\r').unwrap_or(line);
            if line.starts_with('.') {
                format!(".{}", line)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\r\n")
}

/// 收件人合法性：仅允许常规 ASCII 邮箱形态。非法值（如未填邮箱时的手机号回退）
/// 不允许进入 SMTP 会话，调用方直接写入本地发件箱
fn is_valid_email(addr: &str) -> bool {
    let s = addr.trim();
    if s.is_empty() || s.len() > 254 || s.matches('@').count() != 1 {
        return false;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-' | '+'))
    {
        return false;
    }
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    if local.is_empty() || local.starts_with('.') || local.ends_with('.') {
        return false;
    }
    if domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
    {
        return false;
    }
    true
}

/// 组装邮件报文（不含 DATA 终止符；curl 读取 stdin 结束后自动补 ".\r\n"）。
/// 头部统一净化、正文统一 dot-stuffing：发件人/收件人/主题均视为不可信输入
fn smtp_message(from: &str, to: &str, subject: &str, body: &str) -> String {
    format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=utf-8\r\n\r\n{}",
        sanitize_header(from),
        sanitize_header(to),
        sanitize_header(subject),
        dot_stuff_body(body)
    )
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
        if !is_valid_email(recipient) {
            // 收件人不是合法邮箱（如未填邮箱时的手机号回退）：
            // 不允许进入 SMTP 会话，直接写入本地发件箱供人工查看
            let _ = append_outbox(
                &path,
                &DmsNotification {
                    stage,
                    recipient: recipient.to_string(),
                    subject: subject.to_string(),
                    body: body.to_string(),
                    sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    channel: "local_outbox".to_string(),
                },
            );
            "local_outbox".to_string()
        } else {
            let port: u16 = port.parse().unwrap_or(25);
            match try_smtp(&host, port, &from, recipient, subject, body) {
                Ok(()) => "smtp".to_string(),
                Err(_) => {
                    let _ = append_outbox(
                        &path,
                        &DmsNotification {
                            stage,
                            recipient: recipient.to_string(),
                            subject: subject.to_string(),
                            body: body.to_string(),
                            sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            channel: "local_outbox".to_string(),
                        },
                    );
                    "local_outbox".to_string()
                }
            }
        }
    } else {
        let _ = append_outbox(
            &path,
            &DmsNotification {
                stage,
                recipient: recipient.to_string(),
                subject: subject.to_string(),
                body: body.to_string(),
                sent_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                channel: "local_outbox".to_string(),
            },
        );
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

/// 执行一轮通知投递并返回本轮结果。**只有后台循环应调用它**：
/// 它有副作用（写发件箱、发 SMTP）。界面读取状态请用 `get_dms_status`。
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

    let (overdue_days, current_stage) = stage_snapshot(&cfg);

    let overdue = current_stage > 0;
    let user = db::get_user_profile_db()?.unwrap_or_default();
    let mut notifications = Vec::new();

    if overdue {
        let name = if user.full_name.is_empty() {
            "用户".to_string()
        } else {
            user.full_name.clone()
        };
        // 二/三级收件人优先使用邮箱（SMTP 通道只认邮件地址）；未填邮箱时回退手机号，
        // 该值也会原样写入本地发件箱供人工查看，仅 SMTP 投递会失败并降级。
        let stage2_recipient = if user.emergency_contact_email.is_empty() {
            user.emergency_contact_phone.clone()
        } else {
            user.emergency_contact_email.clone()
        };
        let stage3_recipient = if user.heir_primary_email.is_empty() {
            user.heir_primary_phone.clone()
        } else {
            user.heir_primary_email.clone()
        };

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
                &stage2_recipient,
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
                &stage3_recipient,
                &format!("【归零】{} 的数字遗产继承指引", name),
                &format!(
                    "你好 {}，\n\n{} 已长时间未确认安全状态。\n请备齐死亡证明与继承公证材料（或法院判决书），\n在「归零」应用的继承人向导页面按平台逐一办理继承手续。\n本邮件不含任何账号凭据；账号信息请通过继承人本机的加密备份取得。\n\n—— 归零 DigiLegacy",
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

    #[test]
    fn smtp_message_contains_headers_and_body_without_terminator() {
        let msg = smtp_message("a@x.com", "b@y.com", "主题", "正文");
        assert!(msg.starts_with("From: a@x.com\r\nTo: b@y.com\r\nSubject: 主题\r\n"));
        assert!(
            msg.ends_with("正文"),
            "报文不得包含 DATA 终止符，由 curl 负责追加"
        );
        assert!(!msg.contains("\r\n.\r\n"));
    }

    /// 头注入防护：主题里夹带 CR/LF 不得拆出新的邮件头
    #[test]
    fn header_injection_is_neutralized_in_smtp_message() {
        let msg = smtp_message("a@x.com", "b@y.com", "s1\r\nBcc: evil@x.com", "正文");
        assert!(
            !msg.contains("\r\nBcc:"),
            "主题中的换行必须被剥离，不得拆出新头"
        );
        assert!(
            !msg.contains("\r\nSubject: s1\r\n"),
            "不得出现被拆分的重复 Subject 行"
        );
        // 头部行数固定：From/To/Subject/Content-Type + 空行分隔 + 单行正文
        assert_eq!(msg.matches("\r\n").count(), 5, "报文行数不得因注入而增加");
    }

    /// dot-stuffing：正文以点开头的行必须补点，避免提前终止 DATA
    #[test]
    fn body_lines_starting_with_dot_are_stuffed() {
        let msg = smtp_message("a@x.com", "b@y.com", "s", "第一行\r\n.危险行\r\n结束");
        assert!(
            msg.contains("\r\n..危险行\r\n"),
            "以点开头的行必须 dot-stuffing"
        );
        assert!(!msg.contains("\r\n.\r\n"), "不得出现提前的 DATA 终止符");

        // 混合 \n 行尾也必须被处理（正文来自 format! 拼接，常见 \n）
        let msg2 = smtp_message("a@x.com", "b@y.com", "s", "A\n.B\nC");
        assert!(msg2.contains("A\r\n..B\r\nC"));
    }

    /// 收件人校验：合法邮箱放行，手机号/注入形态/残缺地址一律拒绝
    #[test]
    fn smtp_recipient_validator() {
        assert!(is_valid_email("a@b.com"));
        assert!(is_valid_email("user.name+tag@sub.domain.cn"));
        assert!(is_valid_email("a_b-c@d.io"));

        assert!(!is_valid_email(""), "空值必须拒绝");
        assert!(!is_valid_email("13800000000"), "手机号回退必须拒绝");
        assert!(!is_valid_email("a@b"), "缺域名点号必须拒绝");
        assert!(!is_valid_email("a@b."), "域名以点结尾必须拒绝");
        assert!(!is_valid_email("a@b@c.com"), "双 @ 必须拒绝");
        assert!(!is_valid_email("a b@x.com"), "含空格必须拒绝");
        assert!(
            !is_valid_email("a@x.com\r\nBcc: e@x.com"),
            "含 CRLF 必须拒绝"
        );
        assert!(!is_valid_email(".a@x.com"), "本地部以点开头必须拒绝");
    }

    /// netrc 凭据文件格式：curl 按 machine 匹配 host 后取 login/password
    #[test]
    fn netrc_entry_format() {
        assert_eq!(
            netrc_entry("smtp.x.com", "u1", "p2"),
            "machine smtp.x.com login u1 password p2\n"
        );
    }

    /// 安全默认：服务器不提供 STARTTLS 时必须拒绝投递（--ssl-reqd 生效）。
    /// 起一个不广告 STARTTLS 的最小 SMTP mock，断言 try_smtp 返回 Err。
    #[test]
    fn try_smtp_refuses_without_starttls() {
        use std::io::{Read, Write};
        use std::net::TcpListener;

        // 测试环境不得放行明文中继
        std::env::remove_var("DMS_SMTP_ALLOW_INSECURE");

        let listener = TcpListener::bind("127.0.0.1:0").expect("bind 本地端口失败");
        let port = listener.local_addr().unwrap().port();
        let handle = std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf); // 收到 EHLO 前的问候读取
                let _ = stream.write_all(b"220 mock ESMTP\r\n");
                let _ = stream.flush();
                let mut buf = [0u8; 2048];
                let _ = stream.read(&mut buf); // EHLO
                                               // 不广告 STARTTLS
                let _ = stream.write_all(b"250-mock\r\n250 SIZE 10485760\r\n");
                let _ = stream.flush();
                // curl 因 --ssl-reqd 中止连接；无需更多应答
            }
        });

        let result = try_smtp(
            "127.0.0.1",
            port,
            "from@test.local",
            "to@test.local",
            "s",
            "b",
        );
        assert!(result.is_err(), "无 STARTTLS 能力时必须拒绝投递");
        let _ = handle.join();
    }

    /// 去重逻辑：同 stage + 同收件人只投递一次。
    ///
    /// 这条没有测试时，把 `stage` 改成 `stage + 1` 全部 65 个测试仍绿——
    /// 真实后果：去重永远查下一级记录，同一 stage 每 30 分钟重复投递，
    /// 收件人被反复轰炸。
    #[test]
    fn already_notified_dedups_same_stage_and_recipient() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("digilegacy_outbox_{}.jsonl", std::process::id()));
        let _ = fs::remove_file(&path);

        let entry = DmsNotification {
            stage: 2,
            recipient: "heir@example.com".to_string(),
            subject: "s".to_string(),
            body: "b".to_string(),
            sent_at: "2026-01-01 00:00:00".to_string(),
            channel: "local_outbox".to_string(),
        };
        append_outbox(&path, &entry).unwrap();

        assert!(
            already_notified(&path, 2, "heir@example.com"),
            "同 stage 同收件人必须判为已通知"
        );
        assert!(
            !already_notified(&path, 1, "heir@example.com"),
            "不同 stage 不得误判"
        );
        assert!(
            !already_notified(&path, 3, "heir@example.com"),
            "更高 stage 不得误判"
        );
        assert!(
            !already_notified(&path, 2, "other@example.com"),
            "不同收件人不得误判"
        );

        // 不存在的文件：视为未通知，允许首次投递
        let missing = dir.join("digilegacy_outbox_missing.jsonl");
        let _ = fs::remove_file(&missing);
        assert!(!already_notified(&missing, 2, "heir@example.com"));

        let _ = fs::remove_file(&path);
    }

    /// 只读状态快照必须与投递共用同一套分级，但绝不产生副作用。
    #[test]
    fn stage_snapshot_matches_compute_stage() {
        let cfg = DmsConfig {
            enabled: true,
            interval_days: 30,
            stage1_wait_days: 3,
            stage2_wait_days: 7,
            stage3_wait_days: 14,
            last_check_in: Some(
                (Local::now() - chrono::Duration::days(60))
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
            ),
        };
        let (overdue, stage) = stage_snapshot(&cfg);
        assert_eq!(overdue, 30);
        assert_eq!(stage, 3);
        assert_eq!(stage, compute_stage(overdue, &cfg));
    }

    /// 发件箱历史：按日期倒序聚合，坏行跳过而不是整体失败。
    #[test]
    fn outbox_history_reads_newest_first_and_skips_bad_lines() {
        let dir = std::env::temp_dir().join(format!("digilegacy_hist_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let old = dir.join("2026-01-01.jsonl");
        let new = dir.join("2026-01-02.jsonl");
        append_outbox(
            &old,
            &DmsNotification {
                stage: 1,
                recipient: "a@x.com".to_string(),
                subject: "old".to_string(),
                body: "b".to_string(),
                sent_at: "2026-01-01 00:00:00".to_string(),
                channel: "local_outbox".to_string(),
            },
        )
        .unwrap();
        append_outbox(
            &new,
            &DmsNotification {
                stage: 3,
                recipient: "b@x.com".to_string(),
                subject: "new".to_string(),
                body: "b".to_string(),
                sent_at: "2026-01-02 00:00:00".to_string(),
                channel: "smtp".to_string(),
            },
        )
        .unwrap();
        fs::write(new.with_file_name("2026-01-03.jsonl"), "{not json}\n").unwrap();

        let mut out = Vec::new();
        for file in [&new, &old] {
            append_history_from(file, 10, &mut out);
        }
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].stage, 3, "最新的文件先读");

        let _ = fs::remove_dir_all(&dir);
    }
}
