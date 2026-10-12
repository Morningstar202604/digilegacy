//! 领域模型：枚举定义、DTO、统一响应封装

use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
// 领域枚举：用于校验前端传入的合法值
// ──────────────────────────────────────────────

/// 数字资产类别
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetCategory {
    Property,    // 财产型
    Personality, // 人格型
    Mixed,       // 混合型
}

impl AssetCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "property" => Some(Self::Property),
            "personality" => Some(Self::Personality),
            "mixed" => Some(Self::Mixed),
            _ => None,
        }
    }
}

/// 处置方式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DisposalMethod {
    Inherit,
    Liquidate,
    Memorialize,
    AuthorizeView,
    Delete,
    Donate,
    NoAction,
}

impl DisposalMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "inherit" => Some(Self::Inherit),
            "liquidate" => Some(Self::Liquidate),
            "memorialize" => Some(Self::Memorialize),
            "authorize_view" => Some(Self::AuthorizeView),
            "delete" => Some(Self::Delete),
            "donate" => Some(Self::Donate),
            "no_action" => Some(Self::NoAction),
            _ => None,
        }
    }
}

/// 校验并规范化资产类别
pub fn validate_category(cat: &str) -> Result<String, String> {
    match AssetCategory::from_str(cat) {
        Some(_) => Ok(cat.to_string()),
        None => Err(format!("无效的资产类别: {}", cat)),
    }
}

/// 校验并规范化处置方式
pub fn validate_disposal(method: &str) -> Result<String, String> {
    match DisposalMethod::from_str(method) {
        Some(_) => Ok(method.to_string()),
        None => Err(format!("无效的处置方式: {}", method)),
    }
}

// ──────────────────────────────────────────────
// 数据传输对象（DTO）
// ──────────────────────────────────────────────

/// 平台信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Platform {
    pub id: i64,
    pub name: String,
    pub category: String,
    pub subtype: String,
    pub has_property: bool,
    pub has_personality: bool,
    pub inheritable: bool,
    pub required_documents: String,
    pub process_description: String,
    pub estimated_difficulty: String,
    pub official_policy_url: String,
    pub estimated_time_days: String,
    pub notes: String,
}

/// 数字资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalAsset {
    pub id: Option<i64>,
    pub platform_id: i64,
    pub platform_name: String,
    pub account_identifier: String,
    pub account_description: String,
    pub category: String,
    pub estimated_value: Option<f64>,
    pub value_currency: String,
    pub disposal_method: String,
    pub heir_name: Option<String>,
    pub heir_contact: Option<String>,
    pub notes: String,
    pub is_confirmed: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 创建/更新资产的请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRequest {
    pub platform_id: i64,
    pub account_identifier: String,
    pub account_description: String,
    pub category: String,
    pub estimated_value: Option<f64>,
    pub disposal_method: String,
    pub heir_name: Option<String>,
    pub heir_contact: Option<String>,
    pub notes: String,
}

/// 用户档案
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserProfile {
    pub id: Option<i64>,
    pub full_name: String,
    pub phone: String,
    pub email: String,
    pub id_last4: String,
    pub emergency_contact_name: String,
    pub emergency_contact_phone: String,
    pub emergency_contact_relation: String,
    pub heir_primary_name: String,
    pub heir_primary_phone: String,
    pub heir_primary_relation: String,
    pub check_in_interval_days: i64,
    pub last_check_in: Option<String>,
    pub dms_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 继承报告
#[derive(Debug, Serialize)]
pub struct InheritanceReport {
    pub user: UserProfile,
    pub assets: Vec<DigitalAsset>,
    pub total_estimated_value: f64,
    pub property_assets_count: i64,
    pub personality_assets_count: i64,
    pub mixed_assets_count: i64,
    pub generated_at: String,
}

/// 仪表板统计
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_assets: i64,
    pub property_count: i64,
    pub personality_count: i64,
    pub mixed_count: i64,
    pub confirmed_count: i64,
    pub unconfirmed_count: i64,
    pub total_estimated_value: f64,
    pub disposal_planned_count: i64,
    pub last_check_in: Option<String>,
    pub dms_enabled: bool,
    pub completion_percentage: i64,
}

// ──────────────────────────────────────────────
// API 响应统一封装
// ──────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self { success: true, message: "操作成功".into(), data: Some(data) }
    }

    pub fn ok_with_msg(data: T, msg: &str) -> Self {
        Self { success: true, message: msg.into(), data: Some(data) }
    }

    pub fn fail(msg: &str) -> Self {
        Self { success: false, message: msg.into(), data: None }
    }
}