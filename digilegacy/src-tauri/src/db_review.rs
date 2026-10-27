// 审核记录持久化 + 场景侧数据源（通用，不含业务名词）。
//
// 职责边界：
// - 本模块只管「读写审核留痕」与「列出某场景的待审条目」；
// - 判定如何产生由 `review.rs` 的状态机决定；
// - 某个具体场景要审什么，由 `review_profiles.rs` 声明。
//
// 这样新增场景（法律文书、家人说明、资产分类…）不需要动这里。

use chrono::Local;
use rusqlite::params;

use crate::db::get_conn;
use crate::review::{AiVerdict, HumanVerdict, ReviewItem, ReviewRecord, ReviewStatus};

/// 某场景下某条目的当前状态（由最新一条记录 + 内容指纹决定）。
#[derive(Debug, Clone, serde::Serialize)]
pub struct ItemState {
    pub key: String,
    pub title: String,
    pub status: ReviewStatus,
    /// 最新记录的 AI 结论与说明（没有记录时为 None）
    pub ai_verdict: Option<AiVerdict>,
    pub ai_note: String,
    pub human_note: String,
    /// 审核时间（最新记录）
    pub reviewed_at: Option<String>,
    /// 记录是否因内容变化而失效
    pub stale: bool,
}

fn map_ai(s: &str) -> AiVerdict {
    match s {
        "pass" => AiVerdict::Pass,
        "flag" => AiVerdict::Flag,
        "skipped" => AiVerdict::Skipped,
        _ => AiVerdict::Unknown,
    }
}

fn map_human(s: Option<String>) -> Option<HumanVerdict> {
    match s.as_deref() {
        Some("approved") => Some(HumanVerdict::Approved),
        Some("rejected") => Some(HumanVerdict::Rejected),
        Some("needs_edit") => Some(HumanVerdict::NeedsEdit),
        _ => None,
    }
}

fn ai_str(v: &AiVerdict) -> &'static str {
    match v {
        AiVerdict::Pass => "pass",
        AiVerdict::Flag => "flag",
        AiVerdict::Unknown => "unknown",
        AiVerdict::Skipped => "skipped",
    }
}

fn human_str(v: &HumanVerdict) -> &'static str {
    match v {
        HumanVerdict::Approved => "approved",
        HumanVerdict::Rejected => "rejected",
        HumanVerdict::NeedsEdit => "needs_edit",
    }
}

/// 追加一条审核记录（只插不改）。
pub fn append_review_record_db(record: &ReviewRecord) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    conn.execute(
        "INSERT INTO review_records (profile_id, item_key, fingerprint, ai_verdict, ai_note, human_verdict, human_note, decided_by, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            record.profile_id,
            record.item_key,
            record.fingerprint,
            ai_str(&record.ai_verdict),
            record.ai_note,
            record.human_verdict.as_ref().map(human_str),
            record.human_note,
            record.decided_by,
            if record.created_at.is_empty() {
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
            } else {
                record.created_at.clone()
            },
        ],
    )
    .map_err(|e| format!("写入审核记录失败: {}", e))?;
    Ok(())
}

/// 读取某场景某条目的全部历史（新的在前）。
pub fn list_review_records_db(
    profile_id: &str,
    item_key: &str,
) -> Result<Vec<ReviewRecord>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT profile_id, item_key, fingerprint, ai_verdict, ai_note, human_verdict, human_note, decided_by, created_at
             FROM review_records WHERE profile_id=?1 AND item_key=?2 ORDER BY id DESC",
        )
        .map_err(|e| format!("查询审核记录失败: {}", e))?;
    let rows = stmt
        .query_map(params![profile_id, item_key], |r| {
            Ok(ReviewRecord {
                profile_id: r.get(0)?,
                item_key: r.get(1)?,
                fingerprint: r.get(2)?,
                ai_verdict: map_ai(&r.get::<_, String>(3)?),
                ai_note: r.get::<_, String>(4)?,
                human_verdict: map_human(r.get::<_, Option<String>>(5)?),
                human_note: r.get::<_, String>(6)?,
                decided_by: r.get::<_, String>(7)?,
                created_at: r.get::<_, String>(8)?,
            })
        })
        .map_err(|e| format!("读取审核记录失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集审核记录失败: {}", e))?;
    Ok(rows)
}

/// 汇总一组条目的当前状态。
///
/// `items` 由场景提供（含内容指纹），本函数只负责与历史记录对账。
pub fn summarise_states_db(
    profile_id: &str,
    items: &[ReviewItem],
) -> Result<Vec<ItemState>, String> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let records = list_review_records_db(profile_id, &item.key)?;
        let latest = records.first();
        // 判定是否失效统一走 review::is_record_current，
        // 避免「这里比较 fingerprint、那里比较别的东西」两套口径。
        let stale = latest
            .map(|r| !crate::review::is_record_current(r, &item.fingerprint))
            .unwrap_or(false);
        let (status, ai_verdict, ai_note, human_note, reviewed_at) = match latest {
            None => (
                ReviewStatus::Pending,
                None,
                String::new(),
                String::new(),
                None,
            ),
            Some(r) if stale => (
                // 内容已变：旧判定作废，回到待审
                ReviewStatus::Pending,
                Some(r.ai_verdict.clone()),
                r.ai_note.clone(),
                r.human_note.clone(),
                Some(r.created_at.clone()),
            ),
            Some(r) => (
                ReviewStatus::from_verdicts(&r.ai_verdict, r.human_verdict.as_ref()),
                Some(r.ai_verdict.clone()),
                r.ai_note.clone(),
                r.human_note.clone(),
                Some(r.created_at.clone()),
            ),
        };
        out.push(ItemState {
            key: item.key.clone(),
            title: item.title.clone(),
            status,
            ai_verdict,
            ai_note,
            human_note,
            reviewed_at,
            stale,
        });
    }
    Ok(out)
}

/// 统计某场景各状态数量（界面摘要用）。
pub fn review_counts_db(
    profile_id: &str,
    items: &[ReviewItem],
) -> Result<(usize, usize, usize), String> {
    let states = summarise_states_db(profile_id, items)?;
    let approved = states
        .iter()
        .filter(|s| s.status == ReviewStatus::Approved)
        .count();
    let awaiting = states
        .iter()
        .filter(|s| s.status == ReviewStatus::AwaitingHuman)
        .count();
    let pending = states
        .iter()
        .filter(|s| s.status == ReviewStatus::Pending)
        .count();
    Ok((approved, awaiting, pending))
}
