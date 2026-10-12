use tauri::{command, Manager, AppHandle};
use crate::models::*;
use crate::db;

// ========== 平台 ==========

#[command]
pub fn get_all_platforms() -> Result<ApiResponse<Vec<Platform>>, String> {
    db::get_all_platforms_db().map(ApiResponse::ok)
}

// ========== 数字资产 ==========

#[command]
pub fn add_asset(req: AssetRequest) -> Result<ApiResponse<i64>, String> {
    let id = db::add_asset_db(&req)?;
    Ok(ApiResponse::ok_with_msg(id, "数字资产添加成功"))
}

#[command]
pub fn update_asset(id: i64, req: AssetRequest) -> Result<ApiResponse<()>, String> {
    let exists = db::get_asset_by_id_db(id)?.is_some();
    if !exists {
        return Ok(ApiResponse::fail("资产不存在或已被删除"));
    }
    db::update_asset_db(id, &req)?;
    Ok(ApiResponse::ok_with_msg((), "资产更新成功"))
}

#[command]
pub fn delete_asset(id: i64) -> Result<ApiResponse<()>, String> {
    db::delete_asset_db(id)?;
    Ok(ApiResponse::ok_with_msg((), "资产已删除"))
}

#[command]
pub fn get_all_assets() -> Result<ApiResponse<Vec<DigitalAsset>>, String> {
    db::get_all_assets_db().map(ApiResponse::ok)
}

#[command]
pub fn get_asset_by_id(id: i64) -> Result<ApiResponse<Option<DigitalAsset>>, String> {
    db::get_asset_by_id_db(id).map(ApiResponse::ok)
}

#[command]
pub fn get_assets_by_category(category: String) -> Result<ApiResponse<Vec<DigitalAsset>>, String> {
    db::get_assets_by_category_db(&category).map(ApiResponse::ok)
}

// ========== 处置意愿 ==========

#[command]
pub fn set_disposal_wish(
    asset_id: i64, 
    disposal_method: String, 
    heir_name: Option<String>, 
    heir_contact: Option<String>,
    notes: String
) -> Result<ApiResponse<()>, String> {
    let asset = db::get_asset_by_id_db(asset_id)?
        .ok_or("资产不存在")?;
    
    let req = AssetRequest {
        platform_id: asset.platform_id,
        account_identifier: asset.account_identifier,
        account_description: asset.account_description,
        category: asset.category,
        estimated_value: asset.estimated_value,
        disposal_method,
        heir_name,
        heir_contact,
        notes,
    };
    
    db::update_asset_db(asset_id, &req)?;
    Ok(ApiResponse::ok_with_msg((), "处置意愿已设定"))
}

// ========== 用户档案 ==========

#[command]
pub fn set_user_profile(profile: UserProfile) -> Result<ApiResponse<()>, String> {
    db::save_user_profile_db(&profile)?;
    Ok(ApiResponse::ok_with_msg((), "用户档案已保存"))
}

#[command]
pub fn get_user_profile() -> Result<ApiResponse<Option<UserProfile>>, String> {
    db::get_user_profile_db().map(ApiResponse::ok)
}

// ========== 继承报告 ==========

#[command]
pub fn get_inheritance_report() -> Result<ApiResponse<InheritanceReport>, String> {
    db::get_inheritance_report_db().map(ApiResponse::ok)
}

#[command]
pub fn export_report_to_file(app_handle: AppHandle, content: String) -> Result<ApiResponse<String>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;
    
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    
    let file_path = app_dir.join("继承报告.html");
    std::fs::write(&file_path, content).map_err(|e| format!("写入文件失败: {}", e))?;
    
    Ok(ApiResponse::ok_with_msg(
        file_path.to_string_lossy().to_string(),
        "报告已导出到应用数据目录"
    ))
}

// ========== 检查记录 / Dead Man's Switch ==========

#[command]
pub fn update_check_in() -> Result<ApiResponse<String>, String> {
    db::record_check_in_db()?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    Ok(ApiResponse::ok_with_msg(now, "检查完成，状态良好"))
}

#[command]
pub fn get_check_in_status() -> Result<ApiResponse<serde_json::Value>, String> {
    let status = db::get_check_in_status_db()?;
    let json = serde_json::json!({
        "last_check_in": status.last_check_in,
        "interval_days": status.interval_days,
        "dms_enabled": status.dms_enabled,
        "next_check_in_due": status.next_check_in_due,
        "days_until_due": status.days_until_due,
    });
    Ok(ApiResponse::ok(json))
}

#[command]
pub fn set_dead_mans_switch(enabled: bool, interval_days: i64) -> Result<ApiResponse<()>, String> {
    db::set_dms_config_db(enabled, interval_days)?;
    let msg = if enabled { 
        format!("Dead Man's Switch 已开启，每 {} 天检查一次", interval_days) 
    } else { 
        "Dead Man's Switch 已关闭".to_string() 
    };
    Ok(ApiResponse::ok_with_msg((), &msg))
}

// ========== 仪表板统计 ==========

#[command]
pub fn get_dashboard_stats() -> Result<ApiResponse<DashboardStats>, String> {
    db::get_dashboard_stats_db().map(ApiResponse::ok)
}
