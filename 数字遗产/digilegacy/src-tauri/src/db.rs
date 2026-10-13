use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use once_cell::sync::Lazy;
use rusqlite::{Connection, params};
use tauri::{AppHandle, Manager};
use chrono::Local;

use crate::models::*;
use crate::platform_data::get_all_platforms;

/// 全局数据库连接路径
static DB_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

/// 全局共享数据库连接（启用 WAL + busy_timeout，避免并发锁死）
static CONN: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// 初始化数据库
pub fn init_database(app_handle: &AppHandle) -> Result<(), String> {
    let app_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;
    
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建应用目录失败: {}", e))?;
    
    let db_path = app_dir.join("digilegacy.db");
    
    {
        let mut path_guard = DB_PATH.lock().unwrap();
        *path_guard = Some(db_path.clone());
    }

    let conn = Connection::open(&db_path).map_err(|e| format!("打开数据库失败: {}", e))?;
    
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("设置 WAL 模式失败: {}", e))?;
    conn.pragma_update(None, "busy_timeout", "5000")
        .map_err(|e| format!("设置 busy_timeout 失败: {}", e))?;
    conn.pragma_update(None, "foreign_keys", "ON")
        .map_err(|e| format!("启用外键约束失败: {}", e))?;

    create_tables(&conn).map_err(|e| format!("创建表失败: {}", e))?;
    seed_platforms(&conn)?;
    
    conn.execute(
        "INSERT OR IGNORE INTO dms_config (id, enabled, check_in_interval_days, stage1_wait_days, stage2_wait_days, stage3_wait_days, last_check_in) VALUES (1, 0, 30, 3, 7, 14, NULL)",
        [],
    ).map_err(|e| format!("初始化DMS失败: {}", e))?;
    
    {
        let mut conn_guard = CONN.lock().unwrap();
        *conn_guard = Some(conn);
    }
    
    Ok(())
}

/// 创建数据库表
fn create_tables(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS platforms (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            name_en TEXT,
            category TEXT NOT NULL,
            subtype TEXT,
            has_property INTEGER DEFAULT 0,
            has_personality INTEGER DEFAULT 0,
            inheritable INTEGER DEFAULT 0,
            required_documents TEXT,
            process_description TEXT,
            estimated_difficulty TEXT,
            official_policy_url TEXT,
            estimated_time_days TEXT,
            notes TEXT
        );
        
        CREATE TABLE IF NOT EXISTS digital_assets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            platform_id INTEGER NOT NULL,
            account_identifier TEXT,
            account_description TEXT,
            category TEXT NOT NULL,
            estimated_value REAL DEFAULT 0,
            value_currency TEXT DEFAULT 'CNY',
            disposal_method TEXT NOT NULL DEFAULT 'no_action',
            heir_name TEXT,
            heir_contact TEXT,
            notes TEXT DEFAULT '',
            is_confirmed INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (platform_id) REFERENCES platforms(id)
        );
        
        CREATE TABLE IF NOT EXISTS user_profile (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            full_name TEXT,
            phone TEXT,
            email TEXT,
            id_last4 TEXT,
            emergency_contact_name TEXT,
            emergency_contact_phone TEXT,
            emergency_contact_relation TEXT,
            heir_primary_name TEXT,
            heir_primary_phone TEXT,
            heir_primary_relation TEXT,
            check_in_interval_days INTEGER DEFAULT 30,
            last_check_in TEXT,
            dms_enabled INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS check_in_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checked_at TEXT NOT NULL,
            status TEXT NOT NULL,
            note TEXT
        );
        
        CREATE TABLE IF NOT EXISTS dms_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            enabled INTEGER DEFAULT 0,
            check_in_interval_days INTEGER DEFAULT 30,
            stage1_wait_days INTEGER DEFAULT 3,
            stage2_wait_days INTEGER DEFAULT 7,
            stage3_wait_days INTEGER DEFAULT 14,
            last_check_in TEXT
        );
        
        CREATE INDEX IF NOT EXISTS idx_assets_category ON digital_assets(category);
        CREATE INDEX IF NOT EXISTS idx_assets_platform ON digital_assets(platform_id);
        CREATE INDEX IF NOT EXISTS idx_assets_disposal ON digital_assets(disposal_method);
    ")
}

/// 初始化平台数据
fn seed_platforms(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM platforms", [], |row| row.get(0))
        .map_err(|e| format!("查询平台数量失败: {}", e))?;
    
    if count > 0 { return Ok(()); }

    let platforms = get_all_platforms();
    
    for platform in platforms {
        conn.execute(
            "INSERT OR IGNORE INTO platforms (id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                platform.id,
                platform.name,
                platform.name_en,
                platform.category,
                platform.subtype,
                if platform.has_property { 1 } else { 0 },
                if platform.has_personality { 1 } else { 0 },
                if platform.inheritable { 1 } else { 0 },
                platform.required_documents,
                platform.process_description,
                platform.estimated_difficulty,
                platform.official_policy_url,
                platform.estimated_time_days,
                platform.notes,
            ],
        ).map_err(|e| format!("插入平台数据失败: {}", e))?;
    }
    
    Ok(())
}

/// 获取数据库连接（共享连接，避免并发锁死）
pub(crate) fn get_conn() -> Result<MutexGuard<'static, Option<Connection>>, String> {
    let guard = CONN.lock().map_err(|_| "数据库锁中毒".to_string())?;
    if guard.is_none() {
        return Err("数据库未初始化".to_string());
    }
    Ok(guard)
}

/// 辅助函数：解析 rusqlite 行到 Platform
fn row_to_platform(row: &rusqlite::Row) -> rusqlite::Result<Platform> {
    Ok(Platform {
        id: row.get(0)?,
        name: row.get(1)?,
        category: row.get(3)?,
        subtype: row.get(4)?,
        has_property: row.get::<_, i64>(5)? != 0,
        has_personality: row.get::<_, i64>(6)? != 0,
        inheritable: row.get::<_, i64>(7)? != 0,
        required_documents: row.get(8)?,
        process_description: row.get(9)?,
        estimated_difficulty: row.get(10)?,
        official_policy_url: row.get(11)?,
        estimated_time_days: row.get(12)?,
        notes: row.get(13)?,
    })
}

/// 辅助函数：解析 rusqlite 行到 DigitalAsset
fn row_to_asset(row: &rusqlite::Row) -> rusqlite::Result<DigitalAsset> {
    Ok(DigitalAsset {
        id: row.get(0)?,
        platform_id: row.get(1)?,
        platform_name: row.get(2)?,
        account_identifier: row.get(3)?,
        account_description: row.get(4)?,
        category: row.get(5)?,
        estimated_value: row.get(6)?,
        value_currency: row.get(7)?,
        disposal_method: row.get(8)?,
        heir_name: row.get(9)?,
        heir_contact: row.get(10)?,
        notes: row.get(11)?,
        is_confirmed: row.get::<_, i64>(12)? != 0,
        created_at: row.get(13)?,
        updated_at: row.get(14)?,
    })
}

// ========== 平台查询 ==========

pub fn get_all_platforms_db() -> Result<Vec<Platform>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes FROM platforms ORDER BY category, name")
        .map_err(|e| format!("查询平台失败: {}", e))?;
    
    let platforms = stmt.query_map([], row_to_platform)
        .map_err(|e| format!("平台数据解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("平台数据收集失败: {}", e))?;
    
    Ok(platforms)
}

// ========== 数字资产 CRUD ==========

pub fn add_asset_db(req: &AssetRequest) -> Result<i64, String> {
    validate_category(&req.category)?;
    validate_disposal(&req.disposal_method)?;
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO digital_assets (platform_id, account_identifier, account_description, category, estimated_value, value_currency, disposal_method, heir_name, heir_contact, notes, is_confirmed, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'CNY', ?6, ?7, ?8, ?9, 1, ?10, ?10)",
        params![
            req.platform_id,
            req.account_identifier,
            req.account_description,
            req.category,
            req.estimated_value.unwrap_or(0.0),
            req.disposal_method,
            req.heir_name.as_deref().unwrap_or(""),
            req.heir_contact.as_deref().unwrap_or(""),
            req.notes,
            now,
        ],
    ).map_err(|e| format!("添加资产失败: {}", e))?;
    
    Ok(conn.last_insert_rowid())
}

pub fn update_asset_db(id: i64, req: &AssetRequest) -> Result<(), String> {
    validate_category(&req.category)?;
    validate_disposal(&req.disposal_method)?;
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "UPDATE digital_assets SET platform_id=?1, account_identifier=?2, account_description=?3, category=?4, estimated_value=?5, disposal_method=?6, heir_name=?7, heir_contact=?8, notes=?9, is_confirmed=1, updated_at=?10 WHERE id=?11",
        params![
            req.platform_id,
            req.account_identifier,
            req.account_description,
            req.category,
            req.estimated_value.unwrap_or(0.0),
            req.disposal_method,
            req.heir_name.as_deref().unwrap_or(""),
            req.heir_contact.as_deref().unwrap_or(""),
            req.notes,
            now,
            id,
        ],
    ).map_err(|e| format!("更新资产失败: {}", e))?;
    
    Ok(())
}

pub fn delete_asset_db(id: i64) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    conn.execute("DELETE FROM digital_assets WHERE id=?1", params![id])
        .map_err(|e| format!("删除资产失败: {}", e))?;
    Ok(())
}

pub fn get_all_assets_db() -> Result<Vec<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, p.name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at 
         FROM digital_assets a LEFT JOIN platforms p ON a.platform_id = p.id ORDER BY a.created_at DESC"
    ).map_err(|e| format!("查询资产失败: {}", e))?;
    
    let assets = stmt.query_map([], row_to_asset)
        .map_err(|e| format!("解析资产数据失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集资产失败: {}", e))?;
    
    Ok(assets)
}

pub fn get_asset_by_id_db(id: i64) -> Result<Option<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, p.name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at 
         FROM digital_assets a LEFT JOIN platforms p ON a.platform_id = p.id WHERE a.id = ?1"
    ).map_err(|e| format!("查询资产失败: {}", e))?;
    
    let mut rows = stmt.query(params![id]).map_err(|e| format!("查询执行失败: {}", e))?;
    
    if let Some(row) = rows.next().map_err(|e| format!("读取行失败: {}", e))? {
        Ok(Some(row_to_asset(&row).map_err(|e| format!("解析资产失败: {}", e))?))
    } else {
        Ok(None)
    }
}

pub fn get_assets_by_category_db(category: &str) -> Result<Vec<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, p.name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at 
         FROM digital_assets a LEFT JOIN platforms p ON a.platform_id = p.id WHERE a.category = ?1 ORDER BY a.estimated_value DESC"
    ).map_err(|e| format!("按类别查询失败: {}", e))?;
    
    let assets = stmt.query_map(params![category], row_to_asset)
        .map_err(|e| format!("解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集失败: {}", e))?;
    
    Ok(assets)
}

// ========== 用户档案 ==========

pub fn get_user_profile_db() -> Result<Option<UserProfile>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare("SELECT id, full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, heir_primary_name, heir_primary_phone, heir_primary_relation, check_in_interval_days, last_check_in, dms_enabled, created_at, updated_at FROM user_profile ORDER BY id DESC LIMIT 1")
        .map_err(|e| format!("查询用户档案失败: {}", e))?;
    
    let mut rows = stmt.query([]).map_err(|e| format!("查询执行失败: {}", e))?;
    
    if let Some(row) = rows.next().map_err(|e| format!("读取失败: {}", e))? {
        Ok(Some(UserProfile {
            id: row.get(0).map_err(|e| format!("{}", e))?,
            full_name: row.get::<_, Option<String>>(1).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            phone: row.get::<_, Option<String>>(2).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            email: row.get::<_, Option<String>>(3).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            id_last4: row.get::<_, Option<String>>(4).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            emergency_contact_name: row.get::<_, Option<String>>(5).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            emergency_contact_phone: row.get::<_, Option<String>>(6).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            emergency_contact_relation: row.get::<_, Option<String>>(7).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            heir_primary_name: row.get::<_, Option<String>>(8).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            heir_primary_phone: row.get::<_, Option<String>>(9).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            heir_primary_relation: row.get::<_, Option<String>>(10).map_err(|e| format!("{}", e))?.unwrap_or_default(),
            check_in_interval_days: row.get(11).map_err(|e| format!("{}", e))?,
            last_check_in: row.get(12).ok(),
            dms_enabled: row.get::<_, i64>(13).map_err(|e| format!("{}", e))? != 0,
            created_at: row.get(14).map_err(|e| format!("{}", e))?,
            updated_at: row.get(15).map_err(|e| format!("{}", e))?,
        }))
    } else {
        Ok(None)
    }
}

pub fn save_user_profile_db(profile: &UserProfile) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    match profile.id {
        Some(id) => {
            conn.execute(
                "UPDATE user_profile SET full_name=?1, phone=?2, email=?3, id_last4=?4, emergency_contact_name=?5, emergency_contact_phone=?6, emergency_contact_relation=?7, heir_primary_name=?8, heir_primary_phone=?9, heir_primary_relation=?10, check_in_interval_days=?11, dms_enabled=?12, updated_at=?13 WHERE id=?14",
                params![
                    profile.full_name, profile.phone, profile.email, profile.id_last4,
                    profile.emergency_contact_name, profile.emergency_contact_phone, profile.emergency_contact_relation,
                    profile.heir_primary_name, profile.heir_primary_phone, profile.heir_primary_relation,
                    profile.check_in_interval_days,
                    if profile.dms_enabled { 1 } else { 0 },
                    now, id,
                ],
            ).map_err(|e| format!("更新档案失败: {}", e))?;
        }
        None => {
            conn.execute(
                "INSERT INTO user_profile (full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, heir_primary_name, heir_primary_phone, heir_primary_relation, check_in_interval_days, dms_enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?13)",
                params![
                    profile.full_name, profile.phone, profile.email, profile.id_last4,
                    profile.emergency_contact_name, profile.emergency_contact_phone, profile.emergency_contact_relation,
                    profile.heir_primary_name, profile.heir_primary_phone, profile.heir_primary_relation,
                    profile.check_in_interval_days,
                    if profile.dms_enabled { 1 } else { 0 },
                    now,
                ],
            ).map_err(|e| format!("保存档案失败: {}", e))?;
        }
    }
    
    Ok(())
}

// ========== Check-in / Dead Man's Switch ==========

pub fn record_check_in_db() -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO check_in_log (checked_at, status, note) VALUES (?1, 'ok', '用户手动检查')",
        params![now],
    ).map_err(|e| format!("记录检查失败: {}", e))?;
    
    conn.execute(
        "UPDATE dms_config SET last_check_in=?1 WHERE id=1",
        params![now],
    ).map_err(|e| format!("更新DMS状态失败: {}", e))?;
    
    conn.execute(
        "UPDATE user_profile SET last_check_in=?1 WHERE id=(SELECT MAX(id) FROM user_profile)",
        params![now],
    ).map_err(|e| format!("更新用户签到时间失败: {}", e))?;
    
    Ok(())
}

pub fn get_check_in_status_db() -> Result<CheckInStatus, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    
    let last_check_in: Option<String> = conn.query_row(
        "SELECT last_check_in FROM dms_config WHERE id=1",
        [],
        |row| row.get(0),
    ).ok();
    
    let interval: i64 = conn.query_row(
        "SELECT check_in_interval_days FROM dms_config WHERE id=1",
        [],
        |row| row.get(0),
    ).unwrap_or(30);
    
    let dms_enabled: bool = conn.query_row(
        "SELECT enabled FROM dms_config WHERE id=1",
        [],
        |row| row.get::<_, i64>(0),
    ).map(|v| v != 0).unwrap_or(false);
    
    let next_check_in_due = if let Some(ref last) = last_check_in {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S") {
            let next = dt + chrono::Duration::days(interval);
            Some(next.format("%Y-%m-%d").to_string())
        } else { None }
    } else { None };
    
    let days_until_due = if let Some(ref last) = last_check_in {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S") {
            let next = dt + chrono::Duration::days(interval);
            let now = chrono::Local::now().naive_local();
            let duration = next.signed_duration_since(now);
            Some(duration.num_days())
        } else { None }
    } else { None };
    
    Ok(CheckInStatus {
        last_check_in,
        interval_days: interval,
        dms_enabled,
        next_check_in_due,
        days_until_due,
    })
}

pub fn set_dms_config_db(enabled: bool, interval_days: i64) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    
    conn.execute(
        "INSERT OR IGNORE INTO dms_config (id, enabled, check_in_interval_days, stage1_wait_days, stage2_wait_days, stage3_wait_days, last_check_in) VALUES (1, 0, 30, 3, 7, 14, NULL)",
        [],
    ).map_err(|e| format!("初始化DMS配置失败: {}", e))?;
    
    conn.execute(
        "UPDATE dms_config SET enabled=?1, check_in_interval_days=?2 WHERE id=1",
        params![if enabled { 1 } else { 0 }, interval_days],
    ).map_err(|e| format!("设置DMS失败: {}", e))?;
    
    Ok(())
}

pub fn get_dashboard_stats_db() -> Result<DashboardStats, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets", [], |row| row.get(0))
        .map_err(|e| format!("统计失败: {}", e))?;
    
    let property_count: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets WHERE category='property'", [], |row| row.get(0)).unwrap_or(0);
    let personality_count: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets WHERE category='personality'", [], |row| row.get(0)).unwrap_or(0);
    let mixed_count: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets WHERE category='mixed'", [], |row| row.get(0)).unwrap_or(0);
    let confirmed_count: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets WHERE is_confirmed=1", [], |row| row.get(0)).unwrap_or(0);
    let total_value: f64 = conn.query_row("SELECT COALESCE(SUM(estimated_value), 0.0) FROM digital_assets", [], |row| row.get(0)).unwrap_or(0.0);
    let disposal_planned: i64 = conn.query_row("SELECT COUNT(*) FROM digital_assets WHERE disposal_method != 'no_action'", [], |row| row.get(0)).unwrap_or(0);
    
    let last_check_in: Option<String> = conn.query_row("SELECT MAX(checked_at) FROM check_in_log", [], |row| row.get(0)).ok();
    let dms_enabled: bool = conn.query_row("SELECT enabled FROM dms_config WHERE id=1", [], |row| row.get::<_, i64>(0)).map(|v| v != 0).unwrap_or(false);
    
    let completion = if total > 0 { (disposal_planned * 100 / total) as i64 } else { 0 };
    
    Ok(DashboardStats {
        total_assets: total,
        property_count, personality_count, mixed_count,
        confirmed_count, unconfirmed_count: total - confirmed_count,
        total_estimated_value: total_value,
        disposal_planned_count: disposal_planned,
        last_check_in, dms_enabled,
        completion_percentage: completion,
    })
}

/// 检查状态
#[derive(Debug, serde::Serialize)]
pub struct CheckInStatus {
    pub last_check_in: Option<String>,
    pub interval_days: i64,
    pub dms_enabled: bool,
    pub next_check_in_due: Option<String>,
    pub days_until_due: Option<i64>,
}

// ========== 继承报告 ==========

pub fn get_inheritance_report_db() -> Result<InheritanceReport, String> {
    let user = get_user_profile_db()?.unwrap_or(UserProfile {
        id: None,
        full_name: "未设置".to_string(),
        phone: String::new(), email: String::new(), id_last4: String::new(),
        emergency_contact_name: String::new(), emergency_contact_phone: String::new(), emergency_contact_relation: String::new(),
        heir_primary_name: String::new(), heir_primary_phone: String::new(), heir_primary_relation: String::new(),
        check_in_interval_days: 30, last_check_in: None, dms_enabled: false,
        created_at: String::new(), updated_at: String::new(),
    });
    
    let assets = get_all_assets_db()?;
    let total_value: f64 = assets.iter().filter_map(|a| a.estimated_value).sum();
    
    let property_count = assets.iter().filter(|a| a.category == "property").count() as i64;
    let personality_count = assets.iter().filter(|a| a.category == "personality").count() as i64;
    let mixed_count = assets.iter().filter(|a| a.category == "mixed").count() as i64;
    
    Ok(InheritanceReport {
        user, assets,
        total_estimated_value: total_value,
        property_assets_count: property_count,
        personality_assets_count: personality_count,
        mixed_assets_count: mixed_count,
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}