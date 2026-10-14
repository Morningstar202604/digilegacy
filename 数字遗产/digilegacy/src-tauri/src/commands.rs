use tauri::{command, Manager, AppHandle};
use crate::models::*;
use crate::db;
use crate::db::CheckInStatus;
use crate::platform_data::get_all_platforms as get_all_platforms_data;

// ========== 平台 ==========

#[command]
pub fn get_all_platforms() -> Result<ApiResponse<Vec<Platform>>, String> {
    db::get_all_platforms_db().map(ApiResponse::ok)
}

#[command]
pub fn get_all_platforms_extended() -> Result<ApiResponse<Vec<PlatformExtended>>, String> {
    let platforms = get_all_platforms_data()
        .into_iter()
        .map(|p| PlatformExtended {
            base: Platform {
                id: p.id,
                name: p.name,
                category: p.category,
                subtype: p.subtype,
                has_property: p.has_property,
                has_personality: p.has_personality,
                inheritable: p.inheritable,
                required_documents: p.required_documents,
                process_description: p.process_description,
                estimated_difficulty: p.estimated_difficulty,
                official_policy_url: p.official_policy_url,
                estimated_time_days: p.estimated_time_days,
                notes: p.notes,
            },
            name_en: p.name_en,
            data_source: p.data_source,
            inheritance_form_url: p.inheritance_form_url,
            monetization_path: p.monetization_path,
            legal_template_ids: p.legal_template_ids,
            risk_warnings: p.risk_warnings,
            tags: p.tags,
        })
        .collect();
    Ok(ApiResponse::ok(platforms))
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

// ========== ZIP 工具包导出 ==========

#[command]
pub fn export_zip_to_file(content: String, app_handle: AppHandle) -> Result<ApiResponse<String>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD as base64};
    let decoded = base64.decode(&content).map_err(|e| format!("Base64解码失败: {}", e))?;
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;
    
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建目录失败: {}", e))?;
    
    let zip_path = app_dir.join("继承工具包.zip");
    std::fs::write(&zip_path, &decoded).map_err(|e| format!("写入ZIP失败: {}", e))?;
    
    Ok(ApiResponse::ok_with_msg(
        zip_path.to_string_lossy().to_string(),
        "继承工具包已导出"
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
pub fn get_check_in_status() -> Result<ApiResponse<CheckInStatus>, String> {
    db::get_check_in_status_db().map(ApiResponse::ok)
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

// ========== 安全与凭据保管 ==========

#[command]
pub fn security_status() -> Result<ApiResponse<SecurityStatus>, String> {
    let has = db::get_security_config_db()?.is_some();
    Ok(ApiResponse::ok(SecurityStatus {
        has_master_password: has,
        unlocked: crate::vault::is_unlocked(),
    }))
}

#[command]
pub fn set_master_password(password: String) -> Result<ApiResponse<SecurityStatus>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD as base64};

    if password.chars().count() < 8 {
        return Ok(ApiResponse::fail("主密码至少需要 8 个字符"));
    }
    if db::get_security_config_db()?.is_some() {
        return Ok(ApiResponse::fail("已设置主密码，暂不支持修改"));
    }

    let salt = crate::crypto::generate_salt();
    let key = crate::crypto::derive_key_bytes(&password, &salt);
    let verifier = crate::vault::make_verifier(&key)?;
    let (recovery_code, recovery_seal) = crate::recovery::create_recovery_seal(&key)?;
    db::set_security_config_db(&base64.encode(salt), &verifier)?;
    db::set_recovery_seal_db(&recovery_seal)?;
    crate::vault::set_key(key);

    let encrypted_rows = db::encrypt_existing_data_db()?;

    return Ok(ApiResponse {
        success: true,
        message: format!("加密已启用，存量数据已加密 {} 条。请立即抄写恢复密钥：{}", encrypted_rows, recovery_code),
        data: Some(SecurityStatus { has_master_password: true, unlocked: true }),
    });
}

#[command]
pub fn unlock(password: String) -> Result<ApiResponse<SecurityStatus>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD as base64};

    let Some((salt_b64, verifier)) = db::get_security_config_db()? else {
        return Ok(ApiResponse::fail("尚未设置主密码"));
    };
    let salt = base64.decode(&salt_b64).map_err(|_| "安全配置损坏".to_string())?;
    let key = crate::crypto::derive_key_bytes(&password, &salt);

    if crate::vault::verify_key(&verifier, &key) {
        crate::vault::set_key(key);
        Ok(ApiResponse::ok(SecurityStatus { has_master_password: true, unlocked: true }))
    } else {
        Ok(ApiResponse::fail("主密码错误"))
    }
}

#[command]
pub fn lock() -> Result<ApiResponse<SecurityStatus>, String> {
    crate::vault::clear_key();
    Ok(ApiResponse::ok_with_msg(
        SecurityStatus { has_master_password: true, unlocked: false },
        "已锁定",
    ))
}

#[command]
pub fn set_asset_credential(id: i64, credential: String) -> Result<ApiResponse<()>, String> {
    if db::get_asset_by_id_db(id)?.is_none() {
        return Ok(ApiResponse::fail("资产不存在或已被删除"));
    }
    db::set_asset_credential_db(id, &credential)?;
    let msg = if credential.is_empty() { "凭据已清除" } else { "凭据已加密保存" };
    Ok(ApiResponse::ok_with_msg((), msg))
}

#[command]
pub fn get_asset_credential(id: i64) -> Result<ApiResponse<String>, String> {
    db::get_asset_credential_db(id).map(ApiResponse::ok)
}

#[command]
pub fn change_master_password(
    old_password: String,
    new_password: String,
    confirm_password: String,
) -> Result<ApiResponse<SecurityStatus>, String> {
    match db::change_master_password_db(&old_password, &new_password, &confirm_password) {
        Ok(()) => Ok(ApiResponse::ok_with_msg(
            SecurityStatus { has_master_password: true, unlocked: true },
            "主密码已更新，全部数据已用新密钥重新加密",
        )),
        Err(e) => Ok(ApiResponse::fail(&e)),
    }
}

#[command]
pub fn export_backup(app_handle: AppHandle) -> Result<ApiResponse<String>, String> {
    let json = db::export_backup_json_db()?;
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;
    let dir = app_dir.join("备份");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    let name = format!("归零加密备份_{}.json", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let path = dir.join(name);
    std::fs::write(&path, json).map_err(|e| format!("写入备份失败: {}", e))?;
    Ok(ApiResponse::ok_with_msg(
        path.to_string_lossy().to_string(),
        "备份已导出（含密文与安全配置，可与其他设备互导）",
    ))
}

#[command]
pub fn import_backup(content: String) -> Result<ApiResponse<String>, String> {
    match db::import_backup_json_db(&content) {
        Ok(n) => Ok(ApiResponse::ok_with_msg(
            format!("已恢复 {} 条资产，请用该备份对应的主密码重新解锁", n),
            "备份导入完成",
        )),
        Err(e) => Ok(ApiResponse::fail(&e)),
    }
}

#[command]
pub fn get_recovery_seal_exists() -> Result<ApiResponse<bool>, String> {
    Ok(ApiResponse::ok(!db::get_recovery_seal_db()?.is_empty()))
}

/// 恢复流程第一步：用恢复码解出主密钥（不设新密码），返回临时解锁态
#[command]
pub fn start_recovery(recovery_code: String) -> Result<ApiResponse<SecurityStatus>, String> {
    let seal = db::get_recovery_seal_db()?;
    if seal.is_empty() {
        return Ok(ApiResponse::fail("该安装未启用恢复密钥"));
    }
    match crate::recovery::unseal_with_recovery(&recovery_code, &seal) {
        Ok(master_key) => {
            crate::vault::set_key(master_key);
            Ok(ApiResponse::ok(SecurityStatus { has_master_password: true, unlocked: true }))
        }
        Err(_) => Ok(ApiResponse::fail("恢复密钥不正确")),
    }
}

/// 恢复流程第二步：处于恢复解锁态时设置新密码（重派生验证器并轮换恢复码）
#[command]
pub fn complete_recovery(
    new_password: String,
    confirm_password: String,
) -> Result<ApiResponse<String>, String> {
    use base64::{Engine, engine::general_purpose::STANDARD as base64};

    if !crate::vault::is_unlocked() {
        return Ok(ApiResponse::fail("请先通过恢复密钥解锁"));
    }
    if new_password.chars().count() < 8 {
        return Ok(ApiResponse::fail("新主密码至少需要 8 个字符"));
    }
    if new_password != confirm_password {
        return Ok(ApiResponse::fail("两次输入的新主密码不一致"));
    }
    let Some(master_key) = crate::vault::get_key() else {
        return Ok(ApiResponse::fail("解锁状态异常"));
    };

    let salt_new = crate::crypto::generate_salt();
    let key_new = crate::crypto::derive_key_bytes(&new_password, &salt_new);
    let verifier_new = crate::vault::make_verifier(&key_new)?;
    let (recovery_code_new, recovery_seal_new) = crate::recovery::create_recovery_seal(&master_key)?;

    db::reset_security_config_db(&base64.encode(salt_new), &verifier_new, &recovery_seal_new, &master_key, &key_new)?;

    Ok(ApiResponse::ok_with_msg(
        recovery_code_new,
        "主密码已重置，请立即保存新的恢复密钥",
    ))
}

#[command]
pub fn get_heartbeat_status() -> Result<ApiResponse<crate::heartbeat::HeartbeatStatus>, String> {
    Ok(ApiResponse::ok(crate::heartbeat::current_status()))
}

#[command]
pub fn send_heartbeat_now() -> Result<ApiResponse<crate::heartbeat::HeartbeatStatus>, String> {
    Ok(ApiResponse::ok(crate::heartbeat::send_heartbeat()))
}

#[command]
pub fn set_autostart(app_handle: AppHandle, enabled: bool) -> Result<ApiResponse<()>, String> {
    use tauri_plugin_autostart::ManagerExt as _;
    let manager = app_handle.autolaunch();
    let result = if enabled { manager.enable() } else { manager.disable() };
    match result {
        Ok(()) => Ok(ApiResponse::ok_with_msg((), if enabled { "已设置开机自启" } else { "已关闭开机自启" })),
        Err(e) => Ok(ApiResponse::fail(&format!("设置自启失败: {}", e))),
    }
}

#[command]
pub fn hide_main_window(app_handle: AppHandle) -> Result<ApiResponse<()>, String> {
    use tauri::Manager as _;
    if let Some(win) = app_handle.get_webview_window("main") {
        let _ = win.hide();
    }
    Ok(ApiResponse::ok_with_msg((), "已隐藏到托盘"))
}

#[command]
pub fn open_external(url: String) -> Result<ApiResponse<()>, String> {
    let checked = crate::vault::validate_external_url(&url)?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                hwnd: *mut core::ffi::c_void,
                operation: *const u16,
                file: *const u16,
                parameters: *const u16,
                directory: *const u16,
                show_cmd: i32,
            ) -> *mut core::ffi::c_void;
        }
        let to_wide = |s: &str| -> Vec<u16> {
            std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
        };
        let op = to_wide("open");
        let target = to_wide(&checked);
        let result = unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                op.as_ptr(),
                target.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                1,
            )
        };
        if (result as isize) <= 32 {
            return Err("无法打开链接".to_string());
        }
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&checked)
            .spawn()
            .map_err(|e| format!("无法打开链接: {}", e))?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&checked)
            .spawn()
            .map_err(|e| format!("无法打开链接: {}", e))?;
    }

    Ok(ApiResponse::ok_with_msg((), "已在浏览器打开"))
}
