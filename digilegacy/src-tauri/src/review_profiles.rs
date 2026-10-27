// 审核场景注册表：**加一个场景就是往这里加一个声明，不改管线**。
//
// 目前只注册了「平台政策」一个场景。这不是因为管线只支持它，
// 而是因为其余候选场景（法律文书、家人说明、资产分类）需要各自决定
// 「审什么、按什么标准、送审哪些字段」——那是产品判断，不是代码问题。
// 加场景时照着 `policy()` 写一个函数 + 在 `all_profiles()` 里登记即可。
//
// 每个场景都必须自己构造 `ReviewItem`：
// - `payload` 只放该场景允许出境的字段（见 ai_assist 的白名单）；
// - `fingerprint` 必须覆盖所有会影响结论的内容（否则「审旧用新」）。

use crate::family_faq::FAQ;
use crate::platform_data::get_all_platforms;
use crate::review::{fingerprint, ReviewItem, ReviewProfile};

/// 场景 id：与留痕表里的 `profile_id` 对应，**不可随意改名**（会让历史记录失联）。
pub const POLICY_REVIEW: &str = "platform-policy";

/// 平台政策初审。
///
/// 审的是「这条政策的表述是否自相矛盾、是否有可核对来源、是否缺失关键材料」，
/// 不是「这条政策对不对」——后者只能由人对着官方公告核对。
pub fn policy_profile() -> ReviewProfile {
    ReviewProfile {
        id: POLICY_REVIEW.to_string(),
        title_key: "review.profile.policy".to_string(),
        criteria: "检查下列问题并逐条指出：\n\
1) process_description 与 required_documents 是否互相矛盾（例如流程要求公证但材料里没有公证材料）；\n\
2) official_policy_url 是否为空，或明显不是该平台的官方域名；\n\
3) estimated_difficulty 与 estimated_time_days 是否与流程描述的复杂度明显不匹配；\n\
4) 是否存在绝对化承诺（如「一定能继承」「无需任何材料」）；\n\
5) 是否引用了法条编号；若引用，只能提示「需核对原文」，不得判断法条是否正确。"
            .to_string(),
        use_ai: true,
    }
}

/// 用平台库构造政策审核条目。
///
/// 送审字段严格限定为政策文本本身：不带平台 id 以外的任何内部状态，
/// 也不带用户数据（平台库是内置静态数据，与用户无关）。
pub fn policy_items() -> Vec<ReviewItem> {
    get_all_platforms()
        .into_iter()
        .map(|p| {
            let payload = format!(
                "平台：{}\n类别：{}\n可继承：{}\n所需材料：{}\n办理流程：{}\n难度：{}\n预计时长：{}\n官方链接：{}\n来源标注：{}\n备注：{}",
                p.name,
                p.category,
                if p.inheritable { "是" } else { "否" },
                p.required_documents,
                p.process_description,
                p.estimated_difficulty,
                p.estimated_time_days,
                p.official_policy_url,
                p.data_source,
                p.notes,
            );
            ReviewItem {
                key: p.id.to_string(),
                title: p.name.clone(),
                fingerprint: fingerprint(&payload),
                payload,
            }
        })
        .collect()
}

/// 场景列表（界面据此渲染选项卡）。新增场景只改这里。
pub fn all_profiles() -> Vec<ReviewProfile> {
    vec![policy_profile()]
}

/// 按 id 取场景。未知 id 返回 None，而不是回退到某个默认场景——
/// 静默回退会让「审查法律文书」实际跑了「审查政策」，是最坏的一种错。
pub fn profile_by_id(id: &str) -> Option<ReviewProfile> {
    all_profiles().into_iter().find(|p| p.id == id)
}

/// 按场景 id 取待审条目。
pub fn items_for(id: &str) -> Option<Vec<ReviewItem>> {
    match id {
        POLICY_REVIEW => Some(policy_items()),
        _ => None,
    }
}

/// 场景的「送审字段」说明，界面展示用：让用户明确知道 AI 看到了什么。
pub fn payload_fields_for(id: &str) -> Vec<String> {
    match id {
        POLICY_REVIEW => vec![
            "平台名称".to_string(),
            "类别".to_string(),
            "是否可继承".to_string(),
            "所需材料".to_string(),
            "办理流程".to_string(),
            "难度".to_string(),
            "预计时长".to_string(),
            "官方链接".to_string(),
            "来源标注".to_string(),
            "备注".to_string(),
        ],
        _ => vec!["未知场景".to_string()],
    }
}

/// 家人 FAQ 也可以作为审核场景：检查「答案是否与政策库矛盾」。
/// 这里保留构造器但**不默认注册**——FAQ 是静态文案，改动频率低，
/// 是否值得每次跑 AI 初审属于产品取舍。
#[allow(dead_code)]
pub fn faq_items() -> Vec<ReviewItem> {
    FAQ.iter()
        .map(|e| {
            let payload = format!("问题：{}\n回答：{}", e.question_zh, e.answer_zh);
            ReviewItem {
                key: e.id.to_string(),
                title: e.question_zh.to_string(),
                fingerprint: fingerprint(&payload),
                payload,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::review::ReviewStatus;

    #[test]
    fn policy_profile_is_registered() {
        let profiles = all_profiles();
        assert!(!profiles.is_empty(), "至少注册一个场景");
        assert!(profiles.iter().any(|p| p.id == POLICY_REVIEW));
        assert!(profile_by_id(POLICY_REVIEW).is_some());
    }

    /// 未知场景必须返回 None：静默回退会让审核跑错对象。
    #[test]
    fn unknown_profile_returns_none() {
        assert!(profile_by_id("no-such-scene").is_none());
        assert!(items_for("no-such-scene").is_none());
        assert!(profile_by_id("").is_none());
    }

    /// 政策条目必须覆盖整个平台库，且指纹对内容敏感。
    #[test]
    fn policy_items_cover_all_platforms_and_fingerprint_content() {
        let items = policy_items();
        assert_eq!(items.len(), 152, "必须覆盖全部内置平台");
        let first = &items[0];
        assert!(!first.payload.is_empty());
        assert_eq!(first.fingerprint.len(), 16);
        // 同一份数据两次构造指纹必须一致（否则每次刷新都变「待审」）
        let again = policy_items();
        assert_eq!(items[0].fingerprint, again[0].fingerprint);
    }

    /// 送审内容不得包含任何用户数据字段（平台库本身是静态数据）。
    #[test]
    fn policy_payload_has_no_user_data() {
        for item in policy_items() {
            for forbidden in [
                "account_identifier",
                "credential",
                "heir",
                "password",
                "sk-",
            ] {
                assert!(
                    !item.payload.contains(forbidden),
                    "平台 {} 的送审内容含敏感词 {}",
                    item.title,
                    forbidden
                );
            }
        }
    }

    /// 场景声明必须带明确标准，不能是空字符串（否则 AI 无从判断）。
    #[test]
    fn profile_criteria_are_actionable() {
        for p in all_profiles() {
            assert!(
                p.criteria.chars().count() > 30,
                "场景 {} 的审核标准过于笼统",
                p.id
            );
            assert!(!p.title_key.is_empty());
        }
    }

    /// FAQ 场景构造器可用（即使未注册），保证「以后要加」时结构就绪。
    #[test]
    fn faq_items_are_constructible() {
        let items = faq_items();
        assert_eq!(items.len(), FAQ.len());
        assert!(!items[0].payload.is_empty());
    }

    /// AI 初审不能放行：状态机层面再确认一次。
    #[test]
    fn profile_scenario_still_requires_human() {
        use crate::review::{AiVerdict, HumanVerdict};
        assert_eq!(
            ReviewStatus::from_verdicts(&AiVerdict::Pass, None),
            ReviewStatus::AwaitingHuman
        );
        assert_eq!(
            ReviewStatus::from_verdicts(&AiVerdict::Pass, Some(&HumanVerdict::Approved)),
            ReviewStatus::Approved
        );
    }
}
