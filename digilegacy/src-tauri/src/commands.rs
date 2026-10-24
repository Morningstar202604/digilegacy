use crate::db;
use crate::db::CheckInStatus;
use crate::models::*;
use crate::platform_data::get_all_platforms as get_all_platforms_data;
use tauri::{command, AppHandle, Manager};
use zeroize::Zeroize;

// ========== 平台 ==========

#[command]
pub fn get_all_platforms() -> Result<ApiResponse<Vec<Platform>>, String> {
    db::get_all_platforms_db().map(ApiResponse::ok)
}

#[command]
pub fn get_all_platforms_extended() -> Result<ApiResponse<Vec<PlatformExtended>>, String> {
    // 必须从数据库读（而不是直接用内置 const 表），否则用户导入的政策补丁在
    // 继承人向导 / 变现路径卡里完全不生效——两个入口会显示不一致的政策。
    let platforms = db::get_all_platforms_db()?;
    // 扩展元数据先一次性建索引：原实现在 map 内调用 get_all_platforms_data()，
    // 每处理一个平台就重建整张 152 条内置表（O(n²) 且每次分配上千个 String）。
    let meta_by_id: std::collections::HashMap<i64, crate::platform_data::PlatformEntry> =
        get_all_platforms_data()
            .into_iter()
            .map(|m| (m.id, m))
            .collect();
    let extended = platforms
        .into_iter()
        .map(|p| {
            let meta = meta_by_id.get(&p.id).cloned().unwrap_or_else(|| {
                // 补丁新增的平台在内置库里没有扩展元数据，用空值占位
                crate::platform_data::PlatformEntry::new(
                    p.id,
                    &p.name,
                    "",
                    &p.category,
                    &p.subtype,
                    p.has_property,
                    p.has_personality,
                    p.inheritable,
                    &p.required_documents,
                    &p.process_description,
                    &p.estimated_difficulty,
                    &p.official_policy_url,
                    &p.estimated_time_days,
                    &p.notes,
                )
            });
            PlatformExtended {
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
                name_en: meta.name_en,
                data_source: meta.data_source,
                inheritance_form_url: meta.inheritance_form_url,
                monetization_path: meta.monetization_path,
                legal_template_ids: meta.legal_template_ids,
                risk_warnings: meta.risk_warnings,
                tags: meta.tags,
            }
        })
        .collect();
    Ok(ApiResponse::ok(extended))
}

// ========== 政策库补丁（离线更新通道） ==========

#[command]
pub fn import_platform_patch(content: String) -> Result<ApiResponse<String>, String> {
    // 与备份导入同一量级的上限：政策补丁是纯文本 JSON
    if content.len() > 8 * 1024 * 1024 {
        return Ok(ApiResponse::fail("政策补丁过大（上限 8MB）"));
    }
    match db::import_platform_overrides_db(&content) {
        Ok((added, updated)) => Ok(ApiResponse::ok_with_msg(
            format!("新增 {} 条，覆盖 {} 条", added, updated),
            "政策库补丁已导入",
        )),
        Err(e) => Ok(ApiResponse::fail(&e)),
    }
}

#[command]
pub fn export_platform_patch(app_handle: AppHandle) -> Result<ApiResponse<String>, String> {
    let json = db::export_platform_overrides_db()?;
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;
    let dir = app_dir.join("政策补丁");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    let name = format!(
        "digilegacy-platform-patch-{}.json",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
    let path = dir.join(name);
    // 与备份导出同样落在应用数据目录，不额外申请前端写文件权限
    std::fs::write(&path, json).map_err(|e| format!("写入补丁失败: {}", e))?;
    Ok(ApiResponse::ok_with_msg(
        path.to_string_lossy().to_string(),
        "政策补丁已导出",
    ))
}

#[command]
pub fn platform_patch_status() -> Result<ApiResponse<i64>, String> {
    db::count_platform_overrides_db().map(ApiResponse::ok)
}

#[command]
pub fn clear_platform_patch() -> Result<ApiResponse<usize>, String> {
    db::clear_platform_overrides_db().map(ApiResponse::ok)
}

// ========== 数字资产 ==========

/// IPC 入参上限：文本字段单条 8KB、ID 必须为正。
/// 没有上限时，一次失控调用就能把几十 MB 写进 SQLite（且会被每日备份复制 7 份）。
fn validate_asset_request(req: &AssetRequest) -> Result<(), String> {
    if req.platform_id <= 0 {
        return Err("平台 id 必须为正数".to_string());
    }
    for (name, value) in [
        ("账号标识", Some(req.account_identifier.as_str())),
        ("账号描述", Some(req.account_description.as_str())),
        ("备注", Some(req.notes.as_str())),
        ("继承人姓名", req.heir_name.as_deref()),
        ("继承人联系方式", req.heir_contact.as_deref()),
    ] {
        if let Some(v) = value {
            if v.chars().count() > 8_000 {
                return Err(format!("{}过长（上限 8000 字符）", name));
            }
        }
    }
    if let Some(v) = req.estimated_value {
        if !v.is_finite() || v < 0.0 || v > 1e12 {
            return Err("预估价值必须是 0 到 1 万亿之间的有限数值".to_string());
        }
    }
    Ok(())
}

#[command]
pub fn add_asset(req: AssetRequest) -> Result<ApiResponse<i64>, String> {
    if let Err(e) = validate_asset_request(&req) {
        return Ok(ApiResponse::fail(&e));
    }
    let id = db::add_asset_db(&req)?;
    Ok(ApiResponse::ok_with_msg(id, "数字资产添加成功"))
}

#[command]
pub fn update_asset(id: i64, req: AssetRequest) -> Result<ApiResponse<()>, String> {
    let exists = db::get_asset_by_id_db(id)?.is_some();
    if !exists {
        return Ok(ApiResponse::fail("资产不存在或已被删除"));
    }
    if let Err(e) = validate_asset_request(&req) {
        return Ok(ApiResponse::fail(&e));
    }
    db::update_asset_db(id, &req)?;
    Ok(ApiResponse::ok_with_msg((), "资产更新成功"))
}

#[command]
pub fn delete_asset(id: i64) -> Result<ApiResponse<()>, String> {
    if db::get_asset_by_id_db(id)?.is_none() {
        return Ok(ApiResponse::fail("资产不存在或已被删除"));
    }
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
    notes: String,
) -> Result<ApiResponse<()>, String> {
    let asset = db::get_asset_by_id_db(asset_id)?.ok_or("资产不存在")?;

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
    // 档案字段会进入继承报告与 DMS 邮件正文，必须有上限；
    // 其中紧急联系人邮箱还会作为 SMTP 收件人。
    for (name, value) in [
        ("姓名", profile.full_name.as_str()),
        ("手机号", profile.phone.as_str()),
        ("邮箱", profile.email.as_str()),
        ("紧急联系人姓名", profile.emergency_contact_name.as_str()),
        ("紧急联系人电话", profile.emergency_contact_phone.as_str()),
        ("紧急联系人邮箱", profile.emergency_contact_email.as_str()),
        ("继承人姓名", profile.heir_primary_name.as_str()),
        ("继承人电话", profile.heir_primary_phone.as_str()),
        ("继承人邮箱", profile.heir_primary_email.as_str()),
    ] {
        if value.chars().count() > 500 {
            return Ok(ApiResponse::fail(&format!("{}过长（上限 500 字符）", name)));
        }
    }
    if !(1..=365).contains(&profile.check_in_interval_days) {
        return Ok(ApiResponse::fail("检查间隔必须在 1 到 365 天之间"));
    }
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
pub fn export_report_to_file(
    app_handle: AppHandle,
    content: String,
) -> Result<ApiResponse<String>, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;

    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let file_path = app_dir.join(format!(
        "归零继承报告_{}.html",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    ));
    std::fs::write(&file_path, content).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(ApiResponse::ok_with_msg(
        file_path.to_string_lossy().to_string(),
        "报告已导出到应用数据目录",
    ))
}

// ========== ZIP 工具包导出 ==========

#[command]
pub fn export_zip_to_file(
    content: String,
    app_handle: AppHandle,
) -> Result<ApiResponse<String>, String> {
    use base64::{engine::general_purpose::STANDARD as base64, Engine};
    // IPC 入口参数上限：解码后 200MB（对应 base64 约 267MB）。
    // 无上限时恶意/失控前端可凭一次 IPC 请求拖垮进程内存。
    if content.len() > 270_000_000 {
        return Ok(ApiResponse::fail("工具包体积超出上限（200MB）"));
    }
    let decoded = base64
        .decode(&content)
        .map_err(|e| format!("Base64解码失败: {}", e))?;
    if decoded.len() > 200 * 1024 * 1024 {
        return Ok(ApiResponse::fail("工具包体积超出上限（200MB）"));
    }
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;

    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let zip_path = app_dir.join(format!(
        "归零继承工具包_{}.zip",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    ));
    std::fs::write(&zip_path, &decoded).map_err(|e| format!("写入ZIP失败: {}", e))?;

    Ok(ApiResponse::ok_with_msg(
        zip_path.to_string_lossy().to_string(),
        "继承工具包已导出",
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
    // 合法区间与前端下拉一致：1 天（测试/特殊场景）到 365 天。
    // 0 或负数会让「到期日」落在过去，第一天就触发三级通知；极端大值等于永不提醒。
    if !(1..=365).contains(&interval_days) {
        return Ok(ApiResponse::fail("检查间隔必须在 1 到 365 天之间"));
    }
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
pub fn set_master_password(
    mut password: String,
) -> Result<ApiResponse<MasterPasswordSetup>, String> {
    if password.chars().count() < 8 {
        password.zeroize();
        return Ok(ApiResponse::fail("主密码至少需要 8 个字符"));
    }
    if db::get_security_config_db()?.is_some() {
        password.zeroize();
        return Ok(ApiResponse::fail("已设置主密码，暂不支持修改"));
    }

    // v2 盐（自描述前缀）：新装一律 60 万次迭代
    let salt = crate::crypto::generate_salt_v2();
    let (salt_bytes, iterations) = crate::crypto::resolve_salt(&salt)?;
    let key = crate::crypto::derive_key_bytes_v(&password, &salt_bytes, iterations);
    password.zeroize(); // 派生完成后立即擦除明文密码
    let verifier = crate::vault::make_verifier(&key)?;
    let (recovery_code, recovery_seal) = crate::recovery::create_recovery_seal(&key)?;
    db::set_security_config_db(&salt, &verifier)?;
    db::set_recovery_seal_db(&recovery_seal)?;
    crate::vault::set_key(key);

    let encrypted_rows = db::encrypt_existing_data_db()?;

    Ok(ApiResponse {
        success: true,
        message: format!(
            "加密已启用，存量数据已加密 {} 条。请立即抄写恢复密钥。",
            encrypted_rows
        ),
        data: Some(MasterPasswordSetup {
            status: SecurityStatus {
                has_master_password: true,
                unlocked: true,
            },
            recovery_code,
        }),
    })
}

#[command]
pub fn unlock(mut password: String) -> Result<ApiResponse<SecurityStatus>, String> {
    let Some((salt_b64, verifier)) = db::get_security_config_db()? else {
        password.zeroize();
        return Ok(ApiResponse::fail("尚未设置主密码"));
    };
    let (salt_bytes, iterations) = crate::crypto::resolve_salt(&salt_b64)?;
    let key = crate::crypto::derive_key_bytes_v(&password, &salt_bytes, iterations);
    password.zeroize(); // 派生完成后立即擦除明文密码

    if crate::vault::verify_key(&verifier, &key) {
        crate::vault::set_key(key);
        Ok(ApiResponse::ok(SecurityStatus {
            has_master_password: true,
            unlocked: true,
        }))
    } else {
        Ok(ApiResponse::fail("主密码错误"))
    }
}

#[command]
pub fn lock() -> Result<ApiResponse<SecurityStatus>, String> {
    crate::vault::clear_key();
    Ok(ApiResponse::ok_with_msg(
        SecurityStatus {
            has_master_password: true,
            unlocked: false,
        },
        "已锁定",
    ))
}

#[command]
pub fn set_asset_credential(id: i64, credential: String) -> Result<ApiResponse<()>, String> {
    // IPC 入口参数上限：单条凭据 64KB，正常凭据远小于此
    if credential.len() > 64 * 1024 {
        return Ok(ApiResponse::fail("凭据内容过长（上限 64KB）"));
    }
    if db::get_asset_by_id_db(id)?.is_none() {
        return Ok(ApiResponse::fail("资产不存在或已被删除"));
    }
    db::set_asset_credential_db(id, &credential)?;
    let msg = if credential.is_empty() {
        "凭据已清除"
    } else {
        "凭据已加密保存"
    };
    Ok(ApiResponse::ok_with_msg((), msg))
}

#[command]
pub fn get_asset_credential(id: i64) -> Result<ApiResponse<String>, String> {
    db::get_asset_credential_db(id).map(ApiResponse::ok)
}

#[command]
pub fn change_master_password(
    mut old_password: String,
    mut new_password: String,
    mut confirm_password: String,
) -> Result<ApiResponse<MasterPasswordSetup>, String> {
    match db::change_master_password_db(&old_password, &new_password, &confirm_password) {
        Ok(recovery_code) => {
            old_password.zeroize();
            new_password.zeroize();
            confirm_password.zeroize();
            Ok(ApiResponse {
                success: true,
                message: "主密码已更新，全部数据已用新密钥重新加密。请立即抄写新的恢复密钥。"
                    .to_string(),
                data: Some(MasterPasswordSetup {
                    status: SecurityStatus {
                        has_master_password: true,
                        unlocked: true,
                    },
                    recovery_code,
                }),
            })
        }
        Err(e) => {
            old_password.zeroize();
            new_password.zeroize();
            confirm_password.zeroize();
            Ok(ApiResponse::fail(&e))
        }
    }
}

#[command]
pub fn export_backup(app_handle: AppHandle) -> Result<ApiResponse<String>, String> {
    let json = db::export_backup_json_db()?;
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取目录失败: {}", e))?;
    let dir = app_dir.join("备份");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;
    let name = format!(
        "归零加密备份_{}.json",
        chrono::Local::now().format("%Y%m%d_%H%M%S")
    );
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
            Ok(ApiResponse::ok(SecurityStatus {
                has_master_password: true,
                unlocked: true,
            }))
        }
        Err(_) => Ok(ApiResponse::fail("恢复密钥不正确")),
    }
}

/// 恢复流程第二步：处于恢复解锁态时设置新密码（重派生验证器并轮换恢复码）
#[command]
pub fn complete_recovery(
    mut new_password: String,
    mut confirm_password: String,
) -> Result<ApiResponse<String>, String> {
    if !crate::vault::is_unlocked() {
        new_password.zeroize();
        confirm_password.zeroize();
        return Ok(ApiResponse::fail("请先通过恢复密钥解锁"));
    }
    if new_password.chars().count() < 8 {
        new_password.zeroize();
        confirm_password.zeroize();
        return Ok(ApiResponse::fail("新主密码至少需要 8 个字符"));
    }
    if new_password != confirm_password {
        new_password.zeroize();
        confirm_password.zeroize();
        return Ok(ApiResponse::fail("两次输入的新主密码不一致"));
    }
    let Some(master_key) = crate::vault::get_key() else {
        new_password.zeroize();
        confirm_password.zeroize();
        return Ok(ApiResponse::fail("解锁状态异常"));
    };

    // v2 盐与迭代数；恢复封印必须封住"新数据密钥"（数据即将被重加密到 key_new）
    let salt_new = crate::crypto::generate_salt_v2();
    let (salt_new_bytes, iterations) = crate::crypto::resolve_salt(&salt_new)?;
    let key_new = crate::crypto::derive_key_bytes_v(&new_password, &salt_new_bytes, iterations);
    new_password.zeroize();
    confirm_password.zeroize();
    let verifier_new = crate::vault::make_verifier(&key_new)?;
    let (recovery_code_new, recovery_seal_new) = crate::recovery::create_recovery_seal(&key_new)?;

    db::reset_security_config_db(
        &salt_new,
        &verifier_new,
        &recovery_seal_new,
        &master_key,
        &key_new,
    )?;
    // 重加密完成后会话密钥切换为新密钥，保证当前解锁态下数据立即可读
    crate::vault::set_key(key_new);

    Ok(ApiResponse::ok_with_msg(
        recovery_code_new,
        "主密码已重置，请立即保存新的恢复密钥",
    ))
}

#[command]
pub fn get_heartbeat_status() -> Result<ApiResponse<crate::heartbeat::HeartbeatStatus>, String> {
    Ok(ApiResponse::ok(crate::heartbeat::current_status()))
}

/// 读取心跳配置（界面渲染用）。token 只回传「是否已配置」，不回传明文。
#[command]
pub fn get_heartbeat_config() -> Result<ApiResponse<serde_json::Value>, String> {
    let (enabled, token, endpoint) = db::get_heartbeat_config_db()?;
    Ok(ApiResponse::ok(serde_json::json!({
        "enabled": enabled,
        "has_token": !token.is_empty(),
        "endpoint": endpoint,
    })))
}

/// 保存心跳配置。token 传空串表示保持原值不变（避免界面把掩码值回写覆盖真令牌）。
#[command]
pub fn set_heartbeat_config(
    enabled: bool,
    token: String,
    endpoint: String,
) -> Result<ApiResponse<()>, String> {
    let (_, current_token, _) = db::get_heartbeat_config_db()?;
    let next_token = if token.trim().is_empty() {
        current_token
    } else {
        token.trim().to_string()
    };
    if enabled && next_token.is_empty() {
        return Ok(ApiResponse::fail("启用心跳前请先填写令牌"));
    }
    if enabled {
        let ep = endpoint.trim();
        if ep.is_empty() {
            return Ok(ApiResponse::fail(
                "启用心跳前请先填写服务器地址（可自建 server/heartbeat-server.mjs）",
            ));
        }
        if !(ep.starts_with("http://") || ep.starts_with("https://")) {
            return Ok(ApiResponse::fail(
                "服务器地址必须以 http:// 或 https:// 开头",
            ));
        }
    }
    db::set_heartbeat_config_db(enabled, &next_token, endpoint.trim())?;
    Ok(ApiResponse::ok_with_msg(
        (),
        if enabled {
            "心跳已启用"
        } else {
            "心跳已关闭"
        },
    ))
}

#[command]
pub fn send_heartbeat_now() -> Result<ApiResponse<crate::heartbeat::HeartbeatStatus>, String> {
    Ok(ApiResponse::ok(crate::heartbeat::send_heartbeat()))
}

#[command]
pub fn set_autostart(app_handle: AppHandle, enabled: bool) -> Result<ApiResponse<()>, String> {
    use tauri_plugin_autostart::ManagerExt as _;
    let manager = app_handle.autolaunch();
    let result = if enabled {
        manager.enable()
    } else {
        manager.disable()
    };
    match result {
        Ok(()) => Ok(ApiResponse::ok_with_msg(
            (),
            if enabled {
                "已设置开机自启"
            } else {
                "已关闭开机自启"
            },
        )),
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
            std::ffi::OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_master_password_returns_recovery_code_in_data_and_it_unseals() {
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        let res = set_master_password("test-master-123".to_string()).unwrap();
        assert!(res.success);
        // 恢复密钥必须放在 data 字段中交给 UI 展示，而不是塞进 message 里被丢弃
        let recovery_code = res
            .data
            .as_ref()
            .expect("data 必须存在")
            .recovery_code
            .clone();
        assert!(!recovery_code.is_empty(), "恢复密钥不得为空");
        assert_eq!(
            recovery_code.matches('-').count(),
            5,
            "恢复密钥应为 XXXX-…-XXXX 分组格式"
        );
        assert!(
            !res.message.contains(&recovery_code),
            "恢复密钥不得明文出现在 message 中"
        );

        // 用返回的恢复密钥应能解封主密钥（模拟用户忘记主密码后的恢复路径）
        let seal = db::get_recovery_seal_db().unwrap();
        let unsealed = crate::recovery::unseal_with_recovery(&recovery_code, &seal).unwrap();
        assert_eq!(
            unsealed,
            crate::vault::get_key().unwrap(),
            "恢复密钥必须能解出当前主密钥"
        );

        // 重复设置应被拒绝
        let second = set_master_password("another-pass-456".to_string()).unwrap();
        assert!(!second.success);
        assert!(second.data.is_none());

        crate::vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 凭据命令：锁定态必须拒绝读取，且不得把明文回给前端。
    #[test]
    fn credential_commands_refuse_while_locked() {
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_cred_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        // 未启用加密时可直接写入
        let req = crate::models::AssetRequest {
            platform_id: 1,
            account_identifier: "acc".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        };
        let id = db::add_asset_db(&req).unwrap();

        // 凭据的写入门槛比普通字段更严：即使未启用主密码也必须有会话密钥，
        // 否则「保存了凭据」实际是明文直通。这里先启用加密并解锁再写入。
        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("lock-master-123", &salt);
        let verifier = crate::vault::make_verifier(&key).unwrap();
        let salt_b64 = {
            use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
            b64.encode(salt)
        };
        db::set_security_config_db(&salt_b64, &verifier).unwrap();
        crate::vault::set_key(key);
        db::encrypt_existing_data_db().unwrap();

        set_asset_credential(id, "user:pass".to_string()).unwrap();
        assert_eq!(
            get_asset_credential(id).unwrap().data.as_deref(),
            Some("user:pass")
        );

        // 锁定后读取必须被拒绝（不能返回明文，也不能返回成功空串）
        crate::vault::clear_key();

        // 读凭据在锁定态下以 Err 拒绝（fail-closed），而不是返回成功空串——
        // 空串会和「用户确实没存凭据」混淆，Err 才是明确的安全信号。
        let res = get_asset_credential(id);
        assert!(res.is_err(), "锁定态读取凭据必须被拒绝");
        assert!(res.unwrap_err().contains("解锁"), "拒绝原因应提示需要解锁");

        // 写入同样 fail-closed：Err 而非 success:false
        let write = set_asset_credential(id, "new-secret".to_string());
        assert!(write.is_err(), "锁定态写入凭据必须被拒绝");
        assert!(
            write.unwrap_err().contains("解锁"),
            "拒绝原因应提示需要解锁"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 备份导入：非法内容必须拒绝，且不得破坏既有数据。
    #[test]
    fn import_backup_rejects_invalid_without_data_loss() {
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_imp_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        let req = crate::models::AssetRequest {
            platform_id: 1,
            account_identifier: "keep-me".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(9.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        };
        db::add_asset_db(&req).unwrap();

        for bad in ["{not json", "", r#"{"app":"other","version":1}"#] {
            let res = import_backup(bad.to_string()).unwrap();
            assert!(!res.success, "非法备份必须被拒绝: {}", bad);
        }

        let assets = db::get_all_assets_db().unwrap();
        assert_eq!(assets.len(), 1, "失败的导入不得清空既有数据");
        assert_eq!(assets[0].account_identifier, "keep-me");

        let _ = std::fs::remove_file(&path);
    }

    /// 平台补丁导入：超过 8MB 上限必须在解析前拒绝（防内存/CPU 放大）。
    #[test]
    fn import_platform_patch_rejects_oversized() {
        let _g = crate::vault::test_lock();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_patch_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        let huge = "x".repeat(8 * 1024 * 1024 + 1);
        let res = import_platform_patch(huge).unwrap();
        assert!(!res.success);
        assert!(
            res.message.contains("过大"),
            "应提示超过上限，实际: {}",
            res.message
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 心跳配置校验：命令层独有的规则（db 层只负责存取）。
    #[test]
    fn heartbeat_config_validates_before_saving() {
        let _g = crate::vault::test_lock();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_hb_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        // 启用但无令牌 → 拒绝
        let r = set_heartbeat_config(true, String::new(), "https://hb.example.org".to_string())
            .unwrap();
        assert!(!r.success);
        assert!(r.message.contains("令牌"));

        // 启用但无端点 → 拒绝
        let r = set_heartbeat_config(true, "tok".to_string(), String::new()).unwrap();
        assert!(!r.success);
        assert!(r.message.contains("服务器地址"));

        // 端点协议不对 → 拒绝
        for bad in ["ftp://x", "hb.example.org", "javascript:alert(1)"] {
            let r = set_heartbeat_config(true, "tok".to_string(), bad.to_string()).unwrap();
            assert!(!r.success, "非法端点必须拒绝: {}", bad);
        }

        // 合法 → 保存成功，且读取一致
        let r = set_heartbeat_config(
            true,
            "tok-1".to_string(),
            "https://hb.example.org".to_string(),
        )
        .unwrap();
        assert!(r.success);
        let (enabled, token, ep) = db::get_heartbeat_config_db().unwrap();
        assert!(enabled);
        assert_eq!(token, "tok-1");
        assert_eq!(ep, "https://hb.example.org");

        // 传空令牌表示「保持原值」（界面用掩码，不回写真令牌）
        let r = set_heartbeat_config(true, String::new(), "https://hb.example.org".to_string())
            .unwrap();
        assert!(r.success);
        assert_eq!(
            db::get_heartbeat_config_db().unwrap().1,
            "tok-1",
            "空令牌不得清空已有令牌"
        );

        // 关闭时不做端点校验，允许清空
        let r = set_heartbeat_config(false, String::new(), String::new()).unwrap();
        assert!(r.success);
        assert!(!db::get_heartbeat_config_db().unwrap().0);

        let _ = std::fs::remove_file(&path);
    }

    /// 未解锁时凭据类命令与锁定命令的状态一致性。
    #[test]
    fn lock_and_status_commands_are_consistent() {
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_lock_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        // 未设主密码：has_master_password=false。
        // 注意 unlocked 反映的是「内存里是否有主密钥」，此时必然为 false；
        // 「未设密码视为可用」是前端的呈现规则（security.ts: unlocked = has ? data.unlocked : true），
        // 两层的职责不同，这里只锁后端的真实语义。
        let st = security_status().unwrap().data.unwrap();
        assert!(!st.has_master_password);
        assert!(!st.unlocked, "无主密钥时 unlocked 必须为 false");

        // 锁定命令在无主密码时也不得崩溃
        let l = lock().unwrap();
        assert!(l.success);

        let _ = std::fs::remove_file(&path);
    }

    /// 解锁命令：正确密码放行、错误密码拒绝、未设密码时明确报错。
    #[test]
    fn unlock_command_accepts_only_correct_password() {
        use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_unlock_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        // 未设主密码
        let r = unlock("anything".to_string()).unwrap();
        assert!(!r.success);
        assert!(r.message.contains("尚未设置主密码"));

        // 设置主密码后再锁定
        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("unlock-master-1", &salt);
        let verifier = crate::vault::make_verifier(&key).unwrap();
        db::set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        crate::vault::clear_key();

        // 错误密码
        let r = unlock("wrong-password".to_string()).unwrap();
        assert!(!r.success);
        assert_eq!(r.message, "主密码错误");
        assert!(!crate::vault::is_unlocked(), "错误密码不得留下密钥");

        // 正确密码
        let r = unlock("unlock-master-1".to_string()).unwrap();
        assert!(r.success);
        assert!(r.data.as_ref().unwrap().unlocked);
        assert!(crate::vault::is_unlocked());

        crate::vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 恢复命令：未启用恢复密钥时明确拒绝；错误恢复码不得留下密钥。
    #[test]
    fn start_recovery_rejects_without_seal_and_on_bad_code() {
        let _g = crate::vault::test_lock();
        crate::vault::clear_key();
        let mut path = std::env::temp_dir();
        path.push(format!("digilegacy_cmd_rec_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);
        db::init_database_at(path.clone()).unwrap();

        // 未启用恢复密钥
        let r = start_recovery("AAAA-BBBB-CCCC-DDDD-EEEE-FFFF".to_string()).unwrap();
        assert!(!r.success);
        assert!(r.message.contains("未启用恢复密钥"));

        // 有封印但恢复码错误。
        // 前置：set_recovery_seal_db 是 UPDATE，必须先有 security_config 行。
        {
            use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
            let salt = crate::crypto::generate_salt();
            let key = crate::crypto::derive_key_bytes("rec-master-1", &salt);
            let verifier = crate::vault::make_verifier(&key).unwrap();
            db::set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        }
        let master = [5u8; 32];
        let (code, seal) = crate::recovery::create_recovery_seal(&master).unwrap();
        db::set_recovery_seal_db(&seal).unwrap();

        let r = start_recovery("AAAA-BBBB-CCCC-DDDD-EEEE-FFFF".to_string()).unwrap();
        assert!(!r.success, "错误恢复码必须拒绝");
        assert!(!crate::vault::is_unlocked(), "失败不得留下密钥");

        // 正确恢复码
        let r = start_recovery(code).unwrap();
        assert!(r.success);
        assert_eq!(
            *crate::vault::get_key().unwrap(),
            master,
            "必须解出原主密钥"
        );

        crate::vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 资产入参必须在命令层被拦下：超长文本、非有限估值、非法平台 id。
    /// 这些值会写入 SQLite 并被每日备份复制 7 份，不能只靠前端约束。
    #[test]
    fn asset_commands_reject_oversized_and_invalid_payloads() {
        let _g = crate::vault::test_lock();
        db::init_database_at(
            std::env::temp_dir().join(format!("digilegacy_cmd_val_{}.db", std::process::id())),
        )
        .unwrap();

        let base = |over: AssetRequest| over;
        let mk = || AssetRequest {
            platform_id: 1,
            account_identifier: "acc".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        };

        let too_long = base(AssetRequest {
            account_identifier: "x".repeat(8_001),
            ..mk()
        });
        let r = add_asset(too_long).unwrap();
        assert!(!r.success, "超长账号标识必须被拒绝");

        let nan_value = base(AssetRequest {
            estimated_value: Some(f64::NAN),
            ..mk()
        });
        let r = add_asset(nan_value).unwrap();
        assert!(!r.success, "NaN 估值必须被拒绝");

        let bad_platform = base(AssetRequest {
            platform_id: 0,
            ..mk()
        });
        let r = add_asset(bad_platform).unwrap();
        assert!(!r.success, "非法平台 id 必须被拒绝");

        // 合法请求仍应成功（防止校验过严把正常路径也拦掉）
        let r = add_asset(mk()).unwrap();
        assert!(r.success, "合法请求必须通过: {}", r.message);
    }

    /// DMS 间隔必须有界：0/负数会立刻触发三级通知，超大值等于永不提醒。
    #[test]
    fn dms_interval_must_be_within_range() {
        let _g = crate::vault::test_lock();
        db::init_database_at(
            std::env::temp_dir().join(format!("digilegacy_cmd_dms_{}.db", std::process::id())),
        )
        .unwrap();

        for bad in [0i64, -1, 366, 100_000] {
            let r = set_dead_mans_switch(true, bad).unwrap();
            assert!(!r.success, "间隔 {} 必须被拒绝", bad);
        }
        let r = set_dead_mans_switch(true, 30).unwrap();
        assert!(r.success);
        assert_eq!(db::get_check_in_status_db().unwrap().interval_days, 30);
    }
}
