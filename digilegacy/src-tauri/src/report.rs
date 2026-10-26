//! 继承报告 HTML 生成：**后端唯一事实源**。
//!
//! 为什么不让前端抓 DOM：旧实现用 `document.querySelector('.report-page').outerHTML`
//! 拼导出文件，带来三个问题——
//! 1. 锁定态下页面里是「【加密数据 · 需解锁】」占位符，会被原样打印进报告；
//! 2. 页面样式（含 Naive UI 注入）与导出样式两套维护，容易漂移；
//! 3. 同一份内容在「单文件导出」和「ZIP 工具包」里走两条代码路径，曾导致
//!    ZIP 里是裸片段（缺 DOCTYPE/打印样式）。
//!
//! 现在由 Rust 直接渲染：不可信内容统一 HTML 转义，字体/打印样式集中在此，
//! 前端只负责请求与展示，导出物与预览的结构性差异降到零。

use crate::models::{DigitalAsset, InheritanceReport};

const STYLE: &str = r#"
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI','PingFang SC','Microsoft YaHei',sans-serif;max-width:800px;margin:0 auto;padding:44px 48px;color:#232019;background:#fff;line-height:1.6}
h1,h2,h3{margin:0}
table{width:100%;border-collapse:collapse;font-size:14px}
td,th{border-bottom:1px solid #e8e2d8;padding:10px 12px;text-align:left;vertical-align:middle;word-break:break-word}
th{background:#f6f3ee;color:#6b6157;font-weight:600;font-size:13px;border-bottom:2px solid #e4ddd3}
.muted{color:#6b6157}
.doc-header{display:flex;flex-direction:column;gap:6px;padding-bottom:12px;border-bottom:1px solid #e4ddd3;margin-bottom:40px}
.doc-brand{display:flex;align-items:center;gap:8px;font-size:13px;font-weight:600;color:#6b6157;letter-spacing:.5px}
.doc-no{text-align:right;font-size:11px;color:#a49a8d;letter-spacing:.5px}
.doc-title{text-align:center;font-size:25px;font-weight:700;margin-bottom:12px;letter-spacing:6px}
.doc-meta{text-align:center;color:#8a8073;font-size:12px;margin-bottom:40px}
.section{margin-bottom:28px}
.section-title{font-size:15px;font-weight:600;padding-bottom:9px;border-bottom:2px solid #e4ddd3;margin-bottom:14px;letter-spacing:.5px}
.stats{display:grid;grid-template-columns:repeat(4,1fr);gap:14px;margin:16px 0}
.stats div{padding:12px;border:1px solid #e4ddd3;border-radius:8px;text-align:center}
.stats .num{font-size:32px;font-weight:700}
.note{padding:12px;background:#f6f3ee;border-radius:8px;border-left:3px solid #b4594a;margin-bottom:12px}
.doc-footer{display:flex;justify-content:space-between;margin-top:44px;padding-top:14px;border-top:1px solid #e4ddd3;font-size:11px;color:#a49a8d}
@page{margin:18mm 16mm}
@media print{body{padding:0}.section{break-inside:avoid}.no-print{display:none}}
"#;

/// 占位符：未解密的值绝不能出现在任何导出物里。
const PLACEHOLDER: &str = "【加密数据 · 需解锁】";

/// HTML 转义：报告里所有来自数据库/用户输入的文本都必须经过它。
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

/// 报告是否含有未解密占位符。含占位符时禁止生成导出物——
/// 一份写着「需解锁」的报告对继承人毫无价值，且会误导读者以为数据就是这样。
pub fn contains_placeholder(report: &InheritanceReport) -> bool {
    let user_fields = [
        &report.user.full_name,
        &report.user.phone,
        &report.user.email,
        &report.user.emergency_contact_name,
        &report.user.emergency_contact_phone,
        &report.user.heir_primary_name,
        &report.user.heir_primary_phone,
    ];
    if user_fields.iter().any(|v| v.as_str() == PLACEHOLDER) {
        return true;
    }
    report.assets.iter().any(|a| {
        a.account_identifier == PLACEHOLDER
            || a.account_description == PLACEHOLDER
            || a.notes == PLACEHOLDER
            || a.heir_name.as_deref() == Some(PLACEHOLDER)
            || a.heir_contact.as_deref() == Some(PLACEHOLDER)
    })
}

fn category_label(cat: &str) -> &'static str {
    match cat {
        "property" => "财产型",
        "personality" => "人格型",
        "mixed" => "混合型",
        _ => "未知",
    }
}

fn disposal_label(method: &str) -> &'static str {
    match method {
        "inherit" => "指定继承",
        "liquidate" => "变现分配",
        "memorialize" => "保留纪念",
        "authorize_view" => "授权查阅",
        "delete" => "彻底删除",
        "donate" => "捐赠",
        "no_action" => "待设定",
        _ => "未知",
    }
}

fn asset_row(asset: &DigitalAsset) -> String {
    let value = asset
        .estimated_value
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| format!("¥{:.2}", v))
        .unwrap_or_else(|| "-".to_string());
    format!(
        "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
        escape_html(&asset.platform_name),
        escape_html(&asset.account_identifier),
        category_label(&asset.category),
        value,
        disposal_label(&asset.disposal_method),
        escape_html(asset.heir_name.as_deref().unwrap_or("-")),
    )
}

/// 渲染完整、可独立打开的 HTML 文档（单文件导出与 ZIP 工具包共用同一份）。
pub fn render_report_html(report: &InheritanceReport) -> String {
    let doc_no: String = report
        .generated_at
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect();

    let rows = if report.assets.is_empty() {
        "<tr><td colspan=\"6\" class=\"muted\">尚未登记任何数字资产</td></tr>".to_string()
    } else {
        report
            .assets
            .iter()
            .map(asset_row)
            .collect::<Vec<_>>()
            .join("")
    };

    let inheritable = report
        .assets
        .iter()
        .filter(|a| a.disposal_method != "no_action")
        .map(|a| {
            format!(
                "<div class=\"note\"><strong>{}</strong><br><span class=\"muted\">{}</span></div>",
                escape_html(&a.platform_name),
                escape_html(&format!(
                    "处置：{}；继承人：{}",
                    disposal_label(&a.disposal_method),
                    a.heir_name.as_deref().unwrap_or("未指定")
                ))
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN"><head><meta charset="UTF-8"><title>数字遗产继承报告</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<style>{style}</style></head>
<body>
<div class="doc-header">
  <div class="doc-brand">归零 DigiLegacy</div>
  <div class="doc-no">编号 {doc_no}</div>
</div>
<div class="doc-title">数字遗产继承报告</div>
<div class="doc-meta">生成时间 {generated} ｜ 仅供继承人内部使用</div>

<div class="section">
  <div class="section-title">一、持有人信息</div>
  <table><tbody>
    <tr><td>姓名</td><td>{name}</td></tr>
    <tr><td>联系电话</td><td>{phone}</td></tr>
    <tr><td>邮箱</td><td>{email}</td></tr>
    <tr><td>主要继承人</td><td>{heir}（{relation}）</td></tr>
    <tr><td>紧急联系人</td><td>{emergency}（{emergency_relation}）</td></tr>
  </tbody></table>
</div>

<div class="section">
  <div class="section-title">二、资产概览</div>
  <div class="stats">
    <div><div class="num">{total}</div><div class="muted">资产总数</div></div>
    <div><div class="num">{property}</div><div class="muted">财产型</div></div>
    <div><div class="num">{personality}</div><div class="muted">人格型</div></div>
    <div><div class="num">¥{value:.2}</div><div class="muted">预估总值</div></div>
  </div>
</div>

<div class="section">
  <div class="section-title">三、资产清单</div>
  <table><thead><tr><th>平台</th><th>账号标识</th><th>类别</th><th>预估价值</th><th>处置方式</th><th>继承人</th></tr></thead>
  <tbody>{rows}</tbody></table>
</div>

<div class="section">
  <div class="section-title">四、处置安排</div>
  {inheritable}
</div>

<div class="section">
  <div class="section-title">五、声明</div>
  <p>1. 本报告由「归零 DigiLegacy」根据持有人登记的信息生成，不含任何账号密码或密钥。</p>
  <p>2. 账号继承须遵守各平台用户协议与适用法律，通常需提供死亡证明与继承权公证书。</p>
  <p>3. 报告为明文文件，请与纸质文书同等保管。</p>
</div>

<div class="doc-footer">
  <span>归零 DigiLegacy · 本地生成</span>
  <span>第 1 页 / 共 1 页</span>
</div>
</body></html>"#,
        style = STYLE,
        doc_no = escape_html(&doc_no),
        generated = escape_html(&report.generated_at),
        name = escape_html(&report.user.full_name),
        phone = escape_html(&report.user.phone),
        email = escape_html(&report.user.email),
        heir = escape_html(&report.user.heir_primary_name),
        relation = escape_html(&report.user.heir_primary_relation),
        emergency = escape_html(&report.user.emergency_contact_name),
        emergency_relation = escape_html(&report.user.emergency_contact_relation),
        total = report.assets.len(),
        property = report.property_assets_count,
        personality = report.personality_assets_count,
        value = report.total_estimated_value,
        rows = rows,
        inheritable = inheritable,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::UserProfile;

    fn user() -> UserProfile {
        UserProfile {
            id: None,
            full_name: "张三".to_string(),
            phone: "13800000000".to_string(),
            email: "a@b.com".to_string(),
            id_last4: String::new(),
            emergency_contact_name: "李四".to_string(),
            emergency_contact_phone: "13900000000".to_string(),
            emergency_contact_relation: "子女".to_string(),
            emergency_contact_email: String::new(),
            heir_primary_name: "王五".to_string(),
            heir_primary_phone: "13700000000".to_string(),
            heir_primary_relation: "子女".to_string(),
            heir_primary_email: String::new(),
            check_in_interval_days: 30,
            last_check_in: None,
            dms_enabled: false,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn asset(over: impl FnOnce(&mut DigitalAsset)) -> DigitalAsset {
        let mut a = DigitalAsset {
            id: Some(1),
            platform_id: 1,
            platform_name: "支付宝".to_string(),
            account_identifier: "acc".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(100.0),
            value_currency: "CNY".to_string(),
            disposal_method: "inherit".to_string(),
            heir_name: Some("王五".to_string()),
            heir_contact: None,
            notes: String::new(),
            has_credential: false,
            is_confirmed: true,
            created_at: "2026-01-01 00:00:00".to_string(),
            updated_at: "2026-01-01 00:00:00".to_string(),
        };
        over(&mut a);
        a
    }

    fn report(assets: Vec<DigitalAsset>) -> InheritanceReport {
        InheritanceReport {
            user: user(),
            assets,
            total_estimated_value: 100.0,
            property_assets_count: 1,
            personality_assets_count: 0,
            mixed_assets_count: 0,
            generated_at: "2026-10-26 12:00:00".to_string(),
        }
    }

    #[test]
    fn renders_standalone_document_with_print_styles() {
        let html = render_report_html(&report(vec![asset(|_| {})]));
        assert!(html.starts_with("<!DOCTYPE html>"), "必须是完整文档");
        assert!(html.contains("@media print"), "必须带打印样式");
        assert!(html.contains("数字遗产继承报告"));
        assert!(html.contains("张三"));
        assert!(html.contains("支付宝"));
        assert!(html.contains("指定继承"));
        assert!(html.contains("¥100.00"));
    }

    /// 报告里的任何用户输入都必须转义——否则一个平台名就能注入标签。
    #[test]
    fn escapes_all_untrusted_values() {
        let evil = r#"<script>alert('x')</script>"#;
        let mut r = report(vec![asset(|a| {
            a.platform_name = evil.to_string();
            a.account_identifier = evil.to_string();
            a.heir_name = Some(evil.to_string());
        })]);
        r.user.full_name = evil.to_string();
        r.user.heir_primary_name = evil.to_string();

        let html = render_report_html(&r);
        assert!(!html.contains("<script>"), "不得出现未转义脚本标签");
        assert!(html.contains("&lt;script&gt;"), "必须转义为实体");
        assert!(html.contains("&#39;"), "单引号也必须转义");
    }

    /// 含占位符的报告禁止导出：写着「需解锁」的报告会误导继承人。
    #[test]
    fn placeholder_report_is_flagged() {
        assert!(!contains_placeholder(&report(vec![asset(|_| {})])));
        assert!(contains_placeholder(&report(vec![asset(|a| {
            a.account_identifier = PLACEHOLDER.to_string();
        })])));
        assert!(contains_placeholder(&report(vec![asset(|a| {
            a.heir_name = Some(PLACEHOLDER.to_string());
        })])));

        let mut broken_user = report(vec![]);
        broken_user.user.full_name = PLACEHOLDER.to_string();
        assert!(contains_placeholder(&broken_user), "档案占位符也必须被拦");
    }

    /// 空资产清单不能渲染成空表格（继承人会以为文件坏了）。
    #[test]
    fn empty_report_explains_itself() {
        let html = render_report_html(&report(vec![]));
        assert!(html.contains("尚未登记任何数字资产"));
    }

    /// 金额格式化：负值/NaN/None 一律显示 "-"，不得出现 ¥-5 或 ¥NaN。
    #[test]
    fn value_column_never_shows_invalid_amounts() {
        for bad in [None, Some(-5.0), Some(f64::NAN), Some(0.0)] {
            let html = render_report_html(&report(vec![asset(|a| a.estimated_value = bad)]));
            assert!(!html.contains("NaN"), "不得出现 NaN");
            assert!(!html.contains("¥-"), "不得出现负金额");
        }
    }
}
