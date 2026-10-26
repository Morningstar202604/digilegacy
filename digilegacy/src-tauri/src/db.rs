use chrono::Local;
use once_cell::sync::Lazy;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tauri::{AppHandle, Manager};

use crate::models::*;
use crate::platform_data::get_all_platforms;

/// 全局数据库连接路径
static DB_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

/// 全局共享数据库连接（启用 WAL + busy_timeout，避免并发锁死）
static CONN: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

/// 当前 schema 版本，写入 `PRAGMA user_version`。
/// 没有版本号时，未来任何一次加列/改表都无法判断「该不该迁移」，
/// 只能靠 PRAGMA table_info 逐列猜；版本号是后续升级的安全阀。
const SCHEMA_VERSION: i64 = 2;

/// 初始化数据库
pub fn init_database(app_handle: &AppHandle) -> Result<(), String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("获取应用数据目录失败: {}", e))?;

    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建应用目录失败: {}", e))?;

    init_database_at(app_dir.join("digilegacy.db"))
}

/// 在指定路径初始化数据库（供启动流程与测试复用）
pub(crate) fn init_database_at(db_path: PathBuf) -> Result<(), String> {
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
    migrate_schema(&conn).map_err(|e| format!("数据库迁移失败: {}", e))?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(|e| format!("写入 schema 版本失败: {}", e))?;
    sync_platforms(&conn)?;

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
    conn.execute_batch(
        "
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
            credential_enc TEXT DEFAULT '',
            is_confirmed INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (platform_id) REFERENCES platforms(id)
        );

        CREATE TABLE IF NOT EXISTS security_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            kdf_salt TEXT NOT NULL,
            verifier TEXT NOT NULL,
            recovery_seal TEXT DEFAULT '',
            created_at TEXT NOT NULL
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
            emergency_contact_email TEXT,
            heir_primary_name TEXT,
            heir_primary_phone TEXT,
            heir_primary_relation TEXT,
            heir_primary_email TEXT,
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

        CREATE TABLE IF NOT EXISTS heartbeat_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            enabled INTEGER DEFAULT 0,
            token TEXT DEFAULT '',
            endpoint TEXT DEFAULT ''
        );

        -- 政策库补丁：内置平台数据每次启动都会被 sync_platforms 覆盖，
        -- 用户导入的更新必须单独存这里，并在读取时叠加到内置数据之上。
        CREATE TABLE IF NOT EXISTS platform_overrides (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            name_en TEXT DEFAULT '',
            category TEXT NOT NULL,
            subtype TEXT DEFAULT '',
            has_property INTEGER DEFAULT 0,
            has_personality INTEGER DEFAULT 0,
            inheritable INTEGER DEFAULT 0,
            required_documents TEXT DEFAULT '',
            process_description TEXT DEFAULT '',
            estimated_difficulty TEXT DEFAULT '',
            official_policy_url TEXT DEFAULT '',
            estimated_time_days TEXT DEFAULT '',
            notes TEXT DEFAULT '',
            updated_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_assets_category ON digital_assets(category);
        CREATE INDEX IF NOT EXISTS idx_assets_platform ON digital_assets(platform_id);
        CREATE INDEX IF NOT EXISTS idx_assets_disposal ON digital_assets(disposal_method);
    ",
    )
}

/// 增量迁移：为既有数据库补齐新列（幂等）
fn migrate_schema(conn: &Connection) -> rusqlite::Result<()> {
    let has_credential: bool = conn
        .prepare("PRAGMA table_info(digital_assets)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|name| name == "credential_enc");

    if !has_credential {
        conn.execute_batch("ALTER TABLE digital_assets ADD COLUMN credential_enc TEXT DEFAULT ''")?;
    }

    let has_recovery: bool = conn
        .prepare("PRAGMA table_info(security_config)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .any(|name| name == "recovery_seal");

    if !has_recovery {
        conn.execute_batch("ALTER TABLE security_config ADD COLUMN recovery_seal TEXT DEFAULT ''")?;
    }

    let profile_columns: Vec<String> = conn
        .prepare("PRAGMA table_info(user_profile)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .filter_map(|r| r.ok())
        .collect();
    for col in ["emergency_contact_email", "heir_primary_email"] {
        if !profile_columns.iter().any(|name| name == col) {
            conn.execute_batch(&format!(
                "ALTER TABLE user_profile ADD COLUMN {} TEXT DEFAULT ''",
                col
            ))?;
        }
    }
    Ok(())
}

/// 幂等同步平台数据：每次启动 upsert 全部平台（含政策与官方链接），
/// 让存量数据库跟随平台库更新，用户自建资产数据不受影响。
fn sync_platforms(conn: &Connection) -> Result<(), String> {
    let platforms = get_all_platforms();

    // 单事务 + prepare 复用：逐条 autocommit 会让 152 个平台各走一次磁盘落盘，
    // 产生可感知的冷启动耗时方差；包进单事务并复用预编译语句消除之。
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启平台同步事务失败: {}", e))?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO platforms (id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
             ON CONFLICT(id) DO UPDATE SET
                name=excluded.name, name_en=excluded.name_en, category=excluded.category,
                subtype=excluded.subtype, has_property=excluded.has_property,
                has_personality=excluded.has_personality, inheritable=excluded.inheritable,
                required_documents=excluded.required_documents,
                process_description=excluded.process_description,
                estimated_difficulty=excluded.estimated_difficulty,
                official_policy_url=excluded.official_policy_url,
                estimated_time_days=excluded.estimated_time_days,
                notes=excluded.notes",
        ).map_err(|e| format!("准备平台同步语句失败: {}", e))?;
        for platform in platforms {
            stmt.execute(params![
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
            ])
            .map_err(|e| format!("同步平台数据失败: {}", e))?;
        }
    }
    tx.commit()
        .map_err(|e| format!("提交平台同步事务失败: {}", e))?;

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

/// 辅助函数：解析 rusqlite 行到 DigitalAsset（敏感字段自动解密）
fn row_to_asset(row: &rusqlite::Row) -> rusqlite::Result<DigitalAsset> {
    let dec = |idx: usize| -> rusqlite::Result<String> {
        let raw: String = row.get::<_, Option<String>>(idx)?.unwrap_or_default();
        Ok(crate::vault::decrypt_or_placeholder(&raw))
    };
    Ok(DigitalAsset {
        id: row.get(0)?,
        platform_id: row.get(1)?,
        platform_name: row.get(2)?,
        account_identifier: dec(3)?,
        account_description: dec(4)?,
        category: row.get(5)?,
        estimated_value: row.get(6)?,
        value_currency: row.get(7)?,
        disposal_method: row.get(8)?,
        heir_name: {
            let v = dec(9)?;
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        },
        heir_contact: {
            let v = dec(10)?;
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        },
        notes: dec(11)?,
        has_credential: row.get::<_, i64>(15).unwrap_or(0) != 0,
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

    let mut platforms = stmt
        .query_map([], row_to_platform)
        .map_err(|e| format!("平台数据解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("平台数据收集失败: {}", e))?;

    apply_platform_overrides(conn, &mut platforms)?;

    Ok(platforms)
}

// ========== 政策库补丁（离线更新通道） ==========

/// 把用户导入的政策补丁叠加到内置平台列表上。
/// 同一 id 的补丁整体覆盖内置条目；补丁里出现内置库没有的 id 则新增。
fn apply_platform_overrides(
    conn: &Connection,
    platforms: &mut Vec<Platform>,
) -> Result<(), String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes FROM platform_overrides ORDER BY id",
        )
        .map_err(|e| format!("查询政策补丁失败: {}", e))?;

    let overrides: Vec<Platform> = stmt
        .query_map([], row_to_platform)
        .map_err(|e| format!("政策补丁解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("政策补丁收集失败: {}", e))?;

    for ov in overrides {
        match platforms.iter_mut().find(|p| p.id == ov.id) {
            Some(existing) => *existing = ov,
            None => platforms.push(ov),
        }
    }
    Ok(())
}

/// 导入政策补丁。返回 (新增数, 覆盖数)。
/// 只接受结构完整、id 为正、字段非空的条目；整体事务，任一条不合法则全部回滚。
pub fn import_platform_overrides_db(payload: &str) -> Result<(usize, usize), String> {
    #[derive(serde::Deserialize)]
    struct PatchFile {
        #[serde(default)]
        app: String,
        platforms: Vec<PlatformPatch>,
    }
    #[derive(serde::Deserialize)]
    struct PlatformPatch {
        id: i64,
        name: String,
        #[serde(default)]
        name_en: String,
        category: String,
        #[serde(default)]
        subtype: String,
        #[serde(default)]
        has_property: bool,
        #[serde(default)]
        has_personality: bool,
        #[serde(default)]
        inheritable: bool,
        #[serde(default)]
        required_documents: String,
        #[serde(default)]
        process_description: String,
        #[serde(default)]
        estimated_difficulty: String,
        #[serde(default)]
        official_policy_url: String,
        #[serde(default)]
        estimated_time_days: String,
        #[serde(default)]
        notes: String,
    }

    let parsed: PatchFile = serde_json::from_str(payload)
        .map_err(|e| format!("政策补丁解析失败（需为 JSON）: {}", e))?;
    if !parsed.app.is_empty() && parsed.app != "digilegacy" {
        return Err("不是归零政策库补丁（app 字段不匹配）".to_string());
    }
    if parsed.platforms.is_empty() {
        return Err("政策补丁中没有平台条目".to_string());
    }

    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 「新增 vs 覆盖」的判定要把已存在的补丁也算进去，
    // 否则重复导入同一份导出的补丁会被误报成全部新增。
    let existing_ids: std::collections::HashSet<i64> = {
        let mut stmt = tx
            .prepare("SELECT id FROM platforms UNION SELECT id FROM platform_overrides")
            .map_err(|e| format!("查询平台失败: {}", e))?;
        let ids: Vec<i64> = stmt
            .query_map([], |r| r.get::<_, i64>(0))
            .map_err(|e| format!("平台 id 读取失败: {}", e))?
            .filter_map(|r| r.ok())
            .collect();
        ids.into_iter().collect()
    };

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let mut added = 0usize;
    let mut updated = 0usize;
    {
        let mut stmt = tx
            .prepare(
                "INSERT INTO platform_overrides (id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes, updated_at)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)
                 ON CONFLICT(id) DO UPDATE SET
                    name=excluded.name, name_en=excluded.name_en, category=excluded.category,
                    subtype=excluded.subtype, has_property=excluded.has_property,
                    has_personality=excluded.has_personality, inheritable=excluded.inheritable,
                    required_documents=excluded.required_documents,
                    process_description=excluded.process_description,
                    estimated_difficulty=excluded.estimated_difficulty,
                    official_policy_url=excluded.official_policy_url,
                    estimated_time_days=excluded.estimated_time_days,
                    notes=excluded.notes, updated_at=excluded.updated_at",
            )
            .map_err(|e| format!("准备补丁写入失败: {}", e))?;

        for p in &parsed.platforms {
            if p.id <= 0 {
                return Err(format!("平台 id 必须为正数（收到 {}）", p.id));
            }
            if p.name.trim().is_empty() || p.category.trim().is_empty() {
                return Err(format!("平台 {} 缺少名称或类别", p.id));
            }
            stmt.execute(params![
                p.id,
                p.name,
                p.name_en,
                p.category,
                p.subtype,
                if p.has_property { 1 } else { 0 },
                if p.has_personality { 1 } else { 0 },
                if p.inheritable { 1 } else { 0 },
                p.required_documents,
                p.process_description,
                p.estimated_difficulty,
                p.official_policy_url,
                p.estimated_time_days,
                p.notes,
                now,
            ])
            .map_err(|e| format!("写入政策补丁失败: {}", e))?;
            if existing_ids.contains(&p.id) {
                updated += 1
            } else {
                added += 1
            }
        }
    }
    tx.commit()
        .map_err(|e| format!("提交政策补丁失败: {}", e))?;

    // 补丁里「内置库没有」的平台只写进 platform_overrides，但 digital_assets 的外键
    // 指向 platforms 表——不镜像过去的话，界面上能选到、一登记资产就
    // "FOREIGN KEY constraint failed"（已由 repro_patch_only_platform_cannot_hold_assets 复现）。
    // 这里把补丁条目同步进 platforms 作为占位行，让外键成立；
    // 读取时 apply_platform_overrides 仍会用补丁内容覆盖展示。
    sync_platform_overrides_into_platforms(conn)?;

    Ok((added, updated))
}

/// 把 platform_overrides 里的条目镜像到 platforms 表（幂等）。
///
/// 只镜像外键所需的行；字段内容以补丁为准，所以这里写入的就是补丁值。
fn sync_platform_overrides_into_platforms(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "INSERT INTO platforms (id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes)
         SELECT id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes
         FROM platform_overrides
         WHERE id NOT IN (SELECT id FROM platforms)
         ON CONFLICT(id) DO NOTHING",
        [],
    )
    .map_err(|e| format!("同步政策补丁平台失败: {}", e))?;
    Ok(())
}

/// 导出当前全部政策补丁（供用户备份或分发给他人）。
pub fn export_platform_overrides_db() -> Result<String, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes FROM platform_overrides ORDER BY id")
        .map_err(|e| format!("查询政策补丁失败: {}", e))?;
    let rows: Vec<serde_json::Value> = stmt
        .query_map([], |r| {
            Ok(serde_json::json!({
                "id": r.get::<_, i64>(0)?,
                "name": r.get::<_, String>(1)?,
                "name_en": r.get::<_, String>(2)?,
                "category": r.get::<_, String>(3)?,
                "subtype": r.get::<_, String>(4)?,
                "has_property": r.get::<_, i64>(5)? != 0,
                "has_personality": r.get::<_, i64>(6)? != 0,
                "inheritable": r.get::<_, i64>(7)? != 0,
                "required_documents": r.get::<_, String>(8)?,
                "process_description": r.get::<_, String>(9)?,
                "estimated_difficulty": r.get::<_, String>(10)?,
                "official_policy_url": r.get::<_, String>(11)?,
                "estimated_time_days": r.get::<_, String>(12)?,
                "notes": r.get::<_, String>(13)?,
            }))
        })
        .map_err(|e| format!("政策补丁解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("政策补丁收集失败: {}", e))?;

    serde_json::to_string_pretty(&serde_json::json!({
        "app": "digilegacy",
        "kind": "platform-policy-patch",
        "version": 1,
        "exported_at": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "platforms": rows,
    }))
    .map_err(|e| format!("序列化政策补丁失败: {}", e))
}

/// 清除全部政策补丁，回到内置库状态。
///
/// 必须同时撤掉 `sync_platform_overrides_into_platforms` 写入的镜像行，
/// 否则「清除补丁」后平台列表仍留着补丁新增的平台（已由场景矩阵抓到）。
/// 内置平台 id 由 `platform_data` 决定，这里直接删「不在内置集合里」的行，
/// 并把被补丁覆盖过的内置行恢复为内置值。
pub fn clear_platform_overrides_db() -> Result<usize, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let n = conn
        .execute("DELETE FROM platform_overrides", [])
        .map_err(|e| format!("清除政策补丁失败: {}", e))?;

    let builtin = get_all_platforms();
    let builtin_ids: std::collections::HashSet<i64> = builtin.iter().map(|p| p.id).collect();
    let mut stmt = conn
        .prepare("SELECT id FROM platforms")
        .map_err(|e| format!("查询平台失败: {}", e))?;
    let extra: Vec<i64> = stmt
        .query_map([], |r| r.get::<_, i64>(0))
        .map_err(|e| format!("读取平台 id 失败: {}", e))?
        .filter_map(|r| r.ok())
        .filter(|id| !builtin_ids.contains(id))
        .collect();
    drop(stmt);
    for id in extra {
        // 仍有资产挂在这些补丁平台上时不能直接删父行（外键会拦），
        // 也不该连资产一起删。保留一行「已归档」的平台占位，等用户自己处理
        // 这些资产（改挂到内置平台或删除），占用方是用户数据，不是补丁。
        let in_use: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM digital_assets WHERE platform_id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if in_use > 0 {
            // 改名标注来源，避免用户以为它还是可用平台
            conn.execute(
                "UPDATE platforms SET name = name || '（补丁已清除）' WHERE id=?1",
                params![id],
            )
            .map_err(|e| format!("标记补丁平台失败: {}", e))?;
            continue;
        }
        conn.execute("DELETE FROM platforms WHERE id=?1", params![id])
            .map_err(|e| format!("清理补丁镜像平台失败: {}", e))?;
    }
    // 被补丁覆盖过的内置行恢复为内置内容（重新 upsert 一次即可）
    crate::db::sync_platforms(conn)?;

    Ok(n)
}

/// 当前补丁条目数（界面显示用）。
pub fn count_platform_overrides_db() -> Result<i64, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    conn.query_row("SELECT COUNT(*) FROM platform_overrides", [], |r| r.get(0))
        .map_err(|e| format!("统计政策补丁失败: {}", e))
}

// ========== 数字资产 CRUD ==========

/// 写敏感字段前的前置校验：已设置主密码但处于锁定状态时，禁止写入。
/// 此时 vault::encrypt_if_needed 会静默明文直通，用户以为已加密实则明文落库——
/// 这是历史缺陷，必须在此处拦截。尚未启用加密（无 security_config）时放行：
/// 存量明文模式由 encrypt_existing_data_db 在启用加密后统一收编。
fn ensure_vault_writable(conn: &Connection) -> Result<(), String> {
    if crate::vault::is_unlocked() {
        return Ok(());
    }
    let configured: i64 = conn
        .query_row("SELECT COUNT(*) FROM security_config WHERE id=1", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    if configured > 0 {
        return Err("主密码已设置但处于锁定状态，请先解锁后再保存".to_string());
    }
    Ok(())
}

/// 写路径的统一守卫：拒绝把「解密失败的占位符」当作真实内容写回。
///
/// 背景：`decrypt_or_placeholder` 在锁定/错钥匙时返回占位符，界面会把它渲染到表单里；
/// 用户点保存时整张表单原样提交，占位符就被当成明文重新加密，原始数据永久丢失。
/// 只拦截占位符本身，不影响用户真的想把字段清空（空串）或手动输入同款文本之外的内容。
fn reject_placeholder(field: &str, value: &str) -> Result<(), String> {
    if crate::vault::is_placeholder(value) {
        return Err(format!(
            "{}当前处于加密锁定状态，无法保存。请先解锁后重试（本次未写入任何数据）",
            field
        ));
    }
    Ok(())
}

pub fn add_asset_db(req: &AssetRequest) -> Result<i64, String> {
    validate_category(&req.category)?;
    validate_disposal(&req.disposal_method)?;
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    ensure_vault_writable(conn)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let account_identifier = crate::vault::encrypt_if_needed(&req.account_identifier)?;
    let account_description = crate::vault::encrypt_if_needed(&req.account_description)?;
    let heir_name = crate::vault::encrypt_if_needed(req.heir_name.as_deref().unwrap_or(""))?;
    let heir_contact = crate::vault::encrypt_if_needed(req.heir_contact.as_deref().unwrap_or(""))?;
    let notes = crate::vault::encrypt_if_needed(&req.notes)?;

    conn.execute(
        "INSERT INTO digital_assets (platform_id, account_identifier, account_description, category, estimated_value, value_currency, disposal_method, heir_name, heir_contact, notes, is_confirmed, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'CNY', ?6, ?7, ?8, ?9, 1, ?10, ?10)",
        params![
            req.platform_id,
            account_identifier,
            account_description,
            req.category,
            req.estimated_value.unwrap_or(0.0),
            req.disposal_method,
            heir_name,
            heir_contact,
            notes,
            now,
        ],
    ).map_err(|e| format!("添加资产失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

pub fn update_asset_db(id: i64, req: &AssetRequest) -> Result<(), String> {
    validate_category(&req.category)?;
    validate_disposal(&req.disposal_method)?;
    // 占位符检查必须在任何写操作之前：这是「数据被覆盖」而非「写入失败」的风险。
    reject_placeholder("账号标识", &req.account_identifier)?;
    reject_placeholder("账号描述", &req.account_description)?;
    reject_placeholder("备注", &req.notes)?;
    if let Some(v) = req.heir_name.as_deref() {
        reject_placeholder("继承人姓名", v)?;
    }
    if let Some(v) = req.heir_contact.as_deref() {
        reject_placeholder("继承人联系方式", v)?;
    }
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    ensure_vault_writable(conn)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let account_identifier = crate::vault::encrypt_if_needed(&req.account_identifier)?;
    let account_description = crate::vault::encrypt_if_needed(&req.account_description)?;
    let heir_name = crate::vault::encrypt_if_needed(req.heir_name.as_deref().unwrap_or(""))?;
    let heir_contact = crate::vault::encrypt_if_needed(req.heir_contact.as_deref().unwrap_or(""))?;
    let notes = crate::vault::encrypt_if_needed(&req.notes)?;

    conn.execute(
        "UPDATE digital_assets SET platform_id=?1, account_identifier=?2, account_description=?3, category=?4, estimated_value=?5, disposal_method=?6, heir_name=?7, heir_contact=?8, notes=?9, is_confirmed=1, updated_at=?10 WHERE id=?11",
        params![
            req.platform_id,
            account_identifier,
            account_description,
            req.category,
            req.estimated_value.unwrap_or(0.0),
            req.disposal_method,
            heir_name,
            heir_contact,
            notes,
            now,
            id,
        ],
    ).map_err(|e| format!("更新资产失败: {}", e))?;

    Ok(())
}

pub fn delete_asset_db(id: i64) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    // 删除同样是写操作：已设主密码的库在锁定态不得删除，
    // 否则「锁定」只挡住了读取，没挡住破坏。
    ensure_vault_writable(conn)?;
    conn.execute("DELETE FROM digital_assets WHERE id=?1", params![id])
        .map_err(|e| format!("删除资产失败: {}", e))?;
    Ok(())
}

pub fn get_all_assets_db() -> Result<Vec<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, COALESCE(o.name, p.name) AS platform_name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at, CASE WHEN a.credential_enc IS NOT NULL AND a.credential_enc != '' THEN 1 ELSE 0 END AS has_cred
         FROM digital_assets a
         LEFT JOIN platforms p ON a.platform_id = p.id
         LEFT JOIN platform_overrides o ON a.platform_id = o.id ORDER BY a.created_at DESC"
    ).map_err(|e| format!("查询资产失败: {}", e))?;

    let assets = stmt
        .query_map([], row_to_asset)
        .map_err(|e| format!("解析资产数据失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集资产失败: {}", e))?;

    Ok(assets)
}

pub fn get_asset_by_id_db(id: i64) -> Result<Option<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, COALESCE(o.name, p.name) AS platform_name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at, CASE WHEN a.credential_enc IS NOT NULL AND a.credential_enc != '' THEN 1 ELSE 0 END AS has_cred
         FROM digital_assets a
         LEFT JOIN platforms p ON a.platform_id = p.id
         LEFT JOIN platform_overrides o ON a.platform_id = o.id WHERE a.id = ?1"
    ).map_err(|e| format!("查询资产失败: {}", e))?;

    let mut rows = stmt
        .query(params![id])
        .map_err(|e| format!("查询执行失败: {}", e))?;

    if let Some(row) = rows.next().map_err(|e| format!("读取行失败: {}", e))? {
        Ok(Some(
            row_to_asset(row).map_err(|e| format!("解析资产失败: {}", e))?,
        ))
    } else {
        Ok(None)
    }
}

pub fn get_assets_by_category_db(category: &str) -> Result<Vec<DigitalAsset>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare(
        "SELECT a.id, a.platform_id, COALESCE(o.name, p.name) AS platform_name, a.account_identifier, a.account_description, a.category, a.estimated_value, a.value_currency, a.disposal_method, a.heir_name, a.heir_contact, a.notes, a.is_confirmed, a.created_at, a.updated_at, CASE WHEN a.credential_enc IS NOT NULL AND a.credential_enc != '' THEN 1 ELSE 0 END AS has_cred
         FROM digital_assets a
         LEFT JOIN platforms p ON a.platform_id = p.id
         LEFT JOIN platform_overrides o ON a.platform_id = o.id WHERE a.category = ?1 ORDER BY a.estimated_value DESC"
    ).map_err(|e| format!("按类别查询失败: {}", e))?;

    let assets = stmt
        .query_map(params![category], row_to_asset)
        .map_err(|e| format!("解析失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集失败: {}", e))?;

    Ok(assets)
}

// ========== 用户档案 ==========

/// 档案中需要加密的字段，顺序即 SELECT / UPDATE 的参数顺序。
/// 「启用加密」「改密」「恢复重设」三条重加密路径共用同一份列清单：
/// 历史缺陷是 `encrypt_existing_data_db` 自持一份少了一列的副本，导致索引越界 panic。
const PROFILE_SENSITIVE_COLUMNS: [&str; 12] = [
    "full_name",
    "phone",
    "email",
    "id_last4",
    "emergency_contact_name",
    "emergency_contact_phone",
    "emergency_contact_relation",
    "emergency_contact_email",
    "heir_primary_name",
    "heir_primary_phone",
    "heir_primary_relation",
    "heir_primary_email",
];

/// 读取最新一条档案的 (id, 敏感字段原文)。无档案返回 None。
fn select_profile_sensitive(conn: &Connection) -> Result<Option<(i64, Vec<String>)>, String> {
    let sql = format!(
        "SELECT id, {} FROM user_profile ORDER BY id DESC LIMIT 1",
        PROFILE_SENSITIVE_COLUMNS.join(", ")
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("查询档案失败: {}", e))?;
    let row = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let mut vals = Vec::with_capacity(PROFILE_SENSITIVE_COLUMNS.len());
            for i in 1..=PROFILE_SENSITIVE_COLUMNS.len() {
                vals.push(row.get::<_, Option<String>>(i)?.unwrap_or_default());
            }
            Ok((id, vals))
        })
        .map_err(|e| format!("读取档案失败: {}", e))?
        .next()
        .transpose()
        .map_err(|e| format!("收集档案失败: {}", e))?;
    Ok(row)
}

/// 按下标顺序写回档案敏感字段，参数个数由列清单决定，不会与 UPDATE 占位符错位。
fn update_profile_sensitive(conn: &Connection, id: i64, vals: &[String]) -> Result<(), String> {
    if vals.len() != PROFILE_SENSITIVE_COLUMNS.len() {
        return Err(format!(
            "档案字段数量不一致：期望 {}，实际 {}",
            PROFILE_SENSITIVE_COLUMNS.len(),
            vals.len()
        ));
    }
    let sql = format!(
        "UPDATE user_profile SET {} WHERE id=?{}",
        PROFILE_SENSITIVE_COLUMNS
            .iter()
            .enumerate()
            .map(|(i, col)| format!("{}=?{}", col, i + 1))
            .collect::<Vec<_>>()
            .join(", "),
        PROFILE_SENSITIVE_COLUMNS.len() + 1
    );
    let mut params_vec: Vec<&dyn rusqlite::ToSql> =
        vals.iter().map(|v| v as &dyn rusqlite::ToSql).collect();
    params_vec.push(&id);
    conn.execute(&sql, params_vec.as_slice())
        .map_err(|e| format!("写入档案失败: {}", e))?;
    Ok(())
}

pub fn get_user_profile_db() -> Result<Option<UserProfile>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn.prepare("SELECT id, full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email, check_in_interval_days, last_check_in, dms_enabled, created_at, updated_at FROM user_profile ORDER BY id DESC LIMIT 1")
        .map_err(|e| format!("查询用户档案失败: {}", e))?;

    let mut rows = stmt.query([]).map_err(|e| format!("查询执行失败: {}", e))?;

    if let Some(row) = rows.next().map_err(|e| format!("读取失败: {}", e))? {
        let dec = |idx: usize| -> String {
            let raw: String = row
                .get::<_, Option<String>>(idx)
                .ok()
                .flatten()
                .unwrap_or_default();
            crate::vault::decrypt_or_placeholder(&raw)
        };
        Ok(Some(UserProfile {
            id: row.get(0).map_err(|e| format!("{}", e))?,
            full_name: dec(1),
            phone: dec(2),
            email: dec(3),
            id_last4: dec(4),
            emergency_contact_name: dec(5),
            emergency_contact_phone: dec(6),
            emergency_contact_relation: dec(7),
            emergency_contact_email: dec(8),
            heir_primary_name: dec(9),
            heir_primary_phone: dec(10),
            heir_primary_relation: dec(11),
            heir_primary_email: dec(12),
            check_in_interval_days: row.get(13).map_err(|e| format!("{}", e))?,
            last_check_in: row.get(14).ok(),
            dms_enabled: row.get::<_, i64>(15).map_err(|e| format!("{}", e))? != 0,
            created_at: row.get(16).map_err(|e| format!("{}", e))?,
            updated_at: row.get(17).map_err(|e| format!("{}", e))?,
        }))
    } else {
        Ok(None)
    }
}

pub fn save_user_profile_db(profile: &UserProfile) -> Result<(), String> {
    for (name, value) in [
        ("姓名", profile.full_name.as_str()),
        ("手机号", profile.phone.as_str()),
        ("邮箱", profile.email.as_str()),
        ("身份证后四位", profile.id_last4.as_str()),
        ("紧急联系人姓名", profile.emergency_contact_name.as_str()),
        ("紧急联系人电话", profile.emergency_contact_phone.as_str()),
        (
            "紧急联系人关系",
            profile.emergency_contact_relation.as_str(),
        ),
        ("紧急联系人邮箱", profile.emergency_contact_email.as_str()),
        ("继承人姓名", profile.heir_primary_name.as_str()),
        ("继承人电话", profile.heir_primary_phone.as_str()),
        ("继承人关系", profile.heir_primary_relation.as_str()),
        ("继承人邮箱", profile.heir_primary_email.as_str()),
    ] {
        reject_placeholder(name, value)?;
    }
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    ensure_vault_writable(conn)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let full_name = crate::vault::encrypt_if_needed(&profile.full_name)?;
    let phone = crate::vault::encrypt_if_needed(&profile.phone)?;
    let email = crate::vault::encrypt_if_needed(&profile.email)?;
    let id_last4 = crate::vault::encrypt_if_needed(&profile.id_last4)?;
    let ec_name = crate::vault::encrypt_if_needed(&profile.emergency_contact_name)?;
    let ec_phone = crate::vault::encrypt_if_needed(&profile.emergency_contact_phone)?;
    let ec_rel = crate::vault::encrypt_if_needed(&profile.emergency_contact_relation)?;
    let ec_email = crate::vault::encrypt_if_needed(&profile.emergency_contact_email)?;
    let h_name = crate::vault::encrypt_if_needed(&profile.heir_primary_name)?;
    let h_phone = crate::vault::encrypt_if_needed(&profile.heir_primary_phone)?;
    let h_rel = crate::vault::encrypt_if_needed(&profile.heir_primary_relation)?;
    let h_email = crate::vault::encrypt_if_needed(&profile.heir_primary_email)?;

    match profile.id {
        Some(id) => {
            conn.execute(
                "UPDATE user_profile SET full_name=?1, phone=?2, email=?3, id_last4=?4, emergency_contact_name=?5, emergency_contact_phone=?6, emergency_contact_relation=?7, emergency_contact_email=?8, heir_primary_name=?9, heir_primary_phone=?10, heir_primary_relation=?11, heir_primary_email=?12, check_in_interval_days=?13, dms_enabled=?14, updated_at=?15 WHERE id=?16",
                params![
                    full_name, phone, email, id_last4,
                    ec_name, ec_phone, ec_rel, ec_email,
                    h_name, h_phone, h_rel, h_email,
                    profile.check_in_interval_days,
                    if profile.dms_enabled { 1 } else { 0 },
                    now, id,
                ],
            ).map_err(|e| format!("更新档案失败: {}", e))?;
        }
        None => {
            conn.execute(
                "INSERT INTO user_profile (full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email, check_in_interval_days, dms_enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?15)",
                params![
                    full_name, phone, email, id_last4,
                    ec_name, ec_phone, ec_rel, ec_email,
                    h_name, h_phone, h_rel, h_email,
                    profile.check_in_interval_days,
                    if profile.dms_enabled { 1 } else { 0 },
                    now,
                ],
            ).map_err(|e| format!("保存档案失败: {}", e))?;
        }
    }

    Ok(())
}

// ========== 安全配置与凭据保管 ==========

/// 读取安全配置：(kdf_salt_base64, verifier)
pub fn get_security_config_db() -> Result<Option<(String, String)>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut stmt = conn
        .prepare("SELECT kdf_salt, verifier FROM security_config WHERE id = 1")
        .map_err(|e| format!("查询安全配置失败: {}", e))?;
    let mut rows = stmt.query([]).map_err(|e| format!("查询执行失败: {}", e))?;
    if let Some(row) = rows.next().map_err(|e| format!("读取失败: {}", e))? {
        let salt: String = row.get(0).map_err(|e| format!("{}", e))?;
        let verifier: String = row.get(1).map_err(|e| format!("{}", e))?;
        Ok(Some((salt, verifier)))
    } else {
        Ok(None)
    }
}

/// 读取恢复封印（主密钥被恢复码加密后的密文；空表示未启用恢复）
pub fn get_recovery_seal_db() -> Result<String, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let seal: String = conn
        .query_row(
            "SELECT COALESCE(recovery_seal, '') FROM security_config WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    Ok(seal)
}

pub fn set_recovery_seal_db(seal: &str) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    conn.execute(
        "UPDATE security_config SET recovery_seal=?1 WHERE id=1",
        params![seal],
    )
    .map_err(|e| format!("保存恢复密钥失败: {}", e))?;
    Ok(())
}

/// 心跳配置（启用开关 / 令牌 / 端点）存本地库，替代只认环境变量的旧实现：
/// 环境变量对普通用户不可达，等于该功能在发布版里无法开启。
pub fn get_heartbeat_config_db() -> Result<(bool, String, String), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let row = conn.query_row(
        "SELECT enabled, COALESCE(token,''), COALESCE(endpoint,'') FROM heartbeat_config WHERE id=1",
        [],
        |r| Ok((r.get::<_, i64>(0)? != 0, r.get::<_, String>(1)?, r.get::<_, String>(2)?)),
    );
    match row {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok((false, String::new(), String::new())),
        Err(e) => Err(format!("读取心跳配置失败: {}", e)),
    }
}

pub fn set_heartbeat_config_db(enabled: bool, token: &str, endpoint: &str) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    conn.execute(
        "INSERT INTO heartbeat_config (id, enabled, token, endpoint) VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET enabled=?1, token=?2, endpoint=?3",
        params![enabled as i64, token, endpoint],
    )
    .map_err(|e| format!("保存心跳配置失败: {}", e))?;
    Ok(())
}

pub fn set_security_config_db(kdf_salt: &str, verifier: &str) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO security_config (id, kdf_salt, verifier, created_at) VALUES (1, ?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET kdf_salt=?1, verifier=?2",
        params![kdf_salt, verifier, now],
    )
    .map_err(|e| format!("保存安全配置失败: {}", e))?;
    Ok(())
}

/// 保存资产凭据（非空时需解锁；空字符串表示清除）
pub fn set_asset_credential_db(id: i64, credential: &str) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let stored = if credential.is_empty() {
        String::new()
    } else {
        if !crate::vault::is_unlocked() {
            return Err("请先解锁后再保存凭据".to_string());
        }
        crate::vault::encrypt_if_needed(credential)?
    };
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE digital_assets SET credential_enc=?1, updated_at=?2 WHERE id=?3",
        params![stored, now, id],
    )
    .map_err(|e| format!("保存凭据失败: {}", e))?;
    Ok(())
}

/// 读取资产凭据（需解锁；未保存返回空字符串）
pub fn get_asset_credential_db(id: i64) -> Result<String, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let raw: String = conn
        .query_row(
            "SELECT COALESCE(credential_enc, '') FROM digital_assets WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询凭据失败: {}", e))?;
    if raw.is_empty() {
        return Ok(String::new());
    }
    crate::vault::decrypt_if_needed(&raw)
}

/// 修改主密码：校验旧密码后全库重加密并更新安全配置（单事务，失败自动回滚）。
/// 返回轮换后的新恢复密钥——改密即轮换恢复码，UI 必须展示给用户抄写，
/// 否则旧恢复码随旧封印一起失效、恢复链路断裂（历史缺陷）。
pub fn change_master_password_db(
    old_password: &str,
    new_password: &str,
    confirm_password: &str,
) -> Result<String, String> {
    if new_password.chars().count() < 8 {
        return Err("新主密码至少需要 8 个字符".to_string());
    }
    if new_password != confirm_password {
        return Err("两次输入的新主密码不一致".to_string());
    }

    let Some((salt_b64, verifier)) = get_security_config_db()? else {
        return Err("尚未设置主密码".to_string());
    };
    // 兼容 v1/v2 盐前缀选择迭代数
    let (salt_old_bytes, old_iterations) = crate::crypto::resolve_salt(&salt_b64)?;
    let key_old = crate::crypto::derive_key_bytes_v(old_password, &salt_old_bytes, old_iterations);
    if !crate::vault::verify_key(&verifier, &key_old) {
        return Err("当前主密码错误".to_string());
    }

    // 新配置一律写 v2（60 万次迭代）
    let salt_new = crate::crypto::generate_salt_v2();
    let (salt_new_bytes, new_iterations) = crate::crypto::resolve_salt(&salt_new)?;
    let key_new = crate::crypto::derive_key_bytes_v(new_password, &salt_new_bytes, new_iterations);
    let verifier_new = crate::vault::make_verifier(&key_new)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 事务块：guard 在块尾释放后再切换会话密钥
    let recovery_code_new = {
        let guard = get_conn()?;
        let conn = guard.as_ref().unwrap();
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("开启事务失败: {}", e))?;

        let txn_result = (|| -> Result<String, String> {
            let mut stmt = conn.prepare(
                "SELECT id, account_identifier, account_description, heir_name, heir_contact, notes, credential_enc FROM digital_assets",
            ).map_err(|e| format!("查询资产失败: {}", e))?;
            let rows: Vec<(i64, String, String, String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    ))
                })
                .map_err(|e| format!("读取资产失败: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("收集资产失败: {}", e))?;

            for (id, ai, ad, hn, hc, nt, cr) in rows {
                conn.execute(
                    "UPDATE digital_assets SET account_identifier=?1, account_description=?2, heir_name=?3, heir_contact=?4, notes=?5, credential_enc=?6 WHERE id=?7",
                    params![
                        crate::vault::rekey_value(&ai, &key_old, &key_new)?,
                        crate::vault::rekey_value(&ad, &key_old, &key_new)?,
                        crate::vault::rekey_value(&hn, &key_old, &key_new)?,
                        crate::vault::rekey_value(&hc, &key_old, &key_new)?,
                        crate::vault::rekey_value(&nt, &key_old, &key_new)?,
                        crate::vault::rekey_value(&cr, &key_old, &key_new)?,
                        id,
                    ],
                ).map_err(|e| format!("重加密资产失败: {}", e))?;
            }

            if let Some((pid, vals)) = select_profile_sensitive(conn)? {
                let mut new_vals = Vec::with_capacity(vals.len());
                for v in &vals {
                    new_vals.push(crate::vault::rekey_value(v, &key_old, &key_new)?);
                }
                update_profile_sensitive(conn, pid, &new_vals)
                    .map_err(|e| format!("重加密档案失败: {}", e))?;
            }

            let (recovery_code_new, recovery_seal_new) =
                crate::recovery::create_recovery_seal(&key_new)
                    .map_err(|e| format!("轮换恢复密钥失败: {}", e))?;
            conn.execute(
                "INSERT INTO security_config (id, kdf_salt, verifier, recovery_seal, created_at) VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET kdf_salt=?1, verifier=?2, recovery_seal=?3",
                params![salt_new, verifier_new, recovery_seal_new, now],
            ).map_err(|e| format!("更新安全配置失败: {}", e))?;

            Ok(recovery_code_new)
        })();

        match txn_result {
            Ok(code) => {
                conn.execute_batch("COMMIT")
                    .map_err(|e| format!("提交事务失败: {}", e))?;
                code
            }
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    };

    crate::vault::set_key(key_new);
    Ok(recovery_code_new)
}

/// 恢复流程落库：重写安全配置（验证器+恢复封印）并把全库字段重加密到主密钥对应的新密码密钥
pub fn reset_security_config_db(
    kdf_salt: &str,
    verifier: &str,
    recovery_seal: &str,
    master_key: &[u8; 32],
    new_key: &[u8; 32],
) -> Result<(), String> {
    {
        let guard = get_conn()?;
        let conn = guard.as_ref().unwrap();
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("开启事务失败: {}", e))?;

        let result = (|| -> Result<(), String> {
            let mut stmt = conn.prepare(
                "SELECT id, account_identifier, account_description, heir_name, heir_contact, notes, credential_enc FROM digital_assets",
            ).map_err(|e| format!("查询资产失败: {}", e))?;
            let rows: Vec<(i64, String, String, String, String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get(0)?,
                        row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                        row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    ))
                })
                .map_err(|e| format!("读取资产失败: {}", e))?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| format!("收集资产失败: {}", e))?;

            for (id, ai, ad, hn, hc, nt, cr) in rows {
                let dec = |v: &String| -> Result<String, String> {
                    if v.is_empty() {
                        Ok(String::new())
                    } else {
                        crate::vault::decrypt_field(v, master_key)
                    }
                };
                let re = |v: &String| -> Result<String, String> {
                    let plain = dec(v)?;
                    if plain.is_empty() {
                        Ok(String::new())
                    } else {
                        crate::vault::encrypt_field(&plain, new_key)
                    }
                };
                conn.execute(
                    "UPDATE digital_assets SET account_identifier=?1, account_description=?2, heir_name=?3, heir_contact=?4, notes=?5, credential_enc=?6 WHERE id=?7",
                    params![re(&ai)?, re(&ad)?, re(&hn)?, re(&hc)?, re(&nt)?, re(&cr)?, id],
                ).map_err(|e| format!("重加密资产失败: {}", e))?;
            }

            if let Some((pid, vals)) = select_profile_sensitive(conn)? {
                let mut new_vals = Vec::with_capacity(vals.len());
                for v in &vals {
                    // 历史明文（如从旧版/未加密实例导入的备份）直接以新密钥加密，
                    // 不能按密文解密——否则整条恢复链路会因一条明文记录而失败。
                    new_vals.push(crate::vault::rekey_value(v, master_key, new_key)?);
                }
                update_profile_sensitive(conn, pid, &new_vals)
                    .map_err(|e| format!("重加密档案失败: {}", e))?;
            }

            conn.execute(
                "UPDATE security_config SET kdf_salt=?1, verifier=?2, recovery_seal=?3 WHERE id=1",
                params![kdf_salt, verifier, recovery_seal],
            )
            .map_err(|e| format!("更新安全配置失败: {}", e))?;

            Ok(())
        })();

        match result {
            Ok(()) => conn
                .execute_batch("COMMIT")
                .map_err(|e| format!("提交事务失败: {}", e))?,
            Err(e) => {
                let _ = conn.execute_batch("ROLLBACK");
                return Err(e);
            }
        }
    }
    crate::vault::set_key(zeroize::Zeroizing::new(*new_key));
    Ok(())
}

/// 导出备份 JSON：密文原样 + 盐/验证器，可在同密码下恢复到其他设备
pub fn export_backup_json_db() -> Result<String, String> {
    use serde_json::{json, Value};

    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();

    let mut assets: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, platform_id, account_identifier, account_description, category, estimated_value, value_currency, disposal_method, heir_name, heir_contact, notes, credential_enc, is_confirmed, created_at, updated_at FROM digital_assets",
        ).map_err(|e| format!("查询资产失败: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "platform_id": row.get::<_, i64>(1)?,
                    "account_identifier": row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    "account_description": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    "category": row.get::<_, String>(4)?,
                    "estimated_value": row.get::<_, Option<f64>>(5)?.unwrap_or(0.0),
                    "value_currency": row.get::<_, Option<String>>(6)?.unwrap_or_else(|| "CNY".to_string()),
                    "disposal_method": row.get::<_, String>(7)?,
                    "heir_name": row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    "heir_contact": row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    "notes": row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                    "credential_enc": row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    "is_confirmed": row.get::<_, i64>(12)?,
                    "created_at": row.get::<_, String>(13)?,
                    "updated_at": row.get::<_, String>(14)?,
                }))
            })
            .map_err(|e| format!("读取资产失败: {}", e))?;
        for r in rows {
            assets.push(r.map_err(|e| format!("资产解析失败: {}", e))?);
        }
    }

    let mut profiles: Vec<Value> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email, check_in_interval_days, last_check_in, dms_enabled, created_at, updated_at FROM user_profile",
        ).map_err(|e| format!("查询档案失败: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "full_name": row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    "phone": row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    "email": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    "id_last4": row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    "emergency_contact_name": row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    "emergency_contact_phone": row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                    "emergency_contact_relation": row.get::<_, Option<String>>(7)?.unwrap_or_default(),
                    "emergency_contact_email": row.get::<_, Option<String>>(8)?.unwrap_or_default(),
                    "heir_primary_name": row.get::<_, Option<String>>(9)?.unwrap_or_default(),
                    "heir_primary_phone": row.get::<_, Option<String>>(10)?.unwrap_or_default(),
                    "heir_primary_relation": row.get::<_, Option<String>>(11)?.unwrap_or_default(),
                    "heir_primary_email": row.get::<_, Option<String>>(12)?.unwrap_or_default(),
                    "check_in_interval_days": row.get::<_, i64>(13)?,
                    "last_check_in": row.get::<_, Option<String>>(14)?,
                    "dms_enabled": row.get::<_, i64>(15)?,
                    "created_at": row.get::<_, String>(16)?,
                    "updated_at": row.get::<_, String>(17)?,
                }))
            })
            .map_err(|e| format!("读取档案失败: {}", e))?;
        for r in rows {
            profiles.push(r.map_err(|e| format!("档案解析失败: {}", e))?);
        }
    }

    let mut check_ins: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, checked_at, status, note FROM check_in_log ORDER BY id")
            .map_err(|e| format!("查询记录失败: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "checked_at": row.get::<_, String>(1)?,
                    "status": row.get::<_, String>(2)?,
                    "note": row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                }))
            })
            .map_err(|e| format!("读取记录失败: {}", e))?;
        for r in rows {
            check_ins.push(r.map_err(|e| format!("记录解析失败: {}", e))?);
        }
    }

    let dms: Option<Value> = conn
        .query_row(
            "SELECT id, enabled, check_in_interval_days, stage1_wait_days, stage2_wait_days, stage3_wait_days, last_check_in FROM dms_config WHERE id=1",
            [],
            |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "enabled": row.get::<_, i64>(1)?,
                    "check_in_interval_days": row.get::<_, i64>(2)?,
                    "stage1_wait_days": row.get::<_, i64>(3)?,
                    "stage2_wait_days": row.get::<_, i64>(4)?,
                    "stage3_wait_days": row.get::<_, i64>(5)?,
                    "last_check_in": row.get::<_, Option<String>>(6)?,
                }))
            },
        )
        .ok();

    let security: Option<Value> = conn
        .query_row(
            "SELECT kdf_salt, verifier, COALESCE(recovery_seal, '') FROM security_config WHERE id = 1",
            [],
            |row| {
                Ok(json!({
                    "kdf_salt": row.get::<_, String>(0)?,
                    "verifier": row.get::<_, String>(1)?,
                    // v3：带上恢复封印。旧版备份不含封印，换机恢复后即使记得恢复码
                    // 也无法再解出主密钥——恢复链路在「换电脑」这一最常见场景下断掉。
                    "recovery_seal": row.get::<_, String>(2)?,
                }))
            },
        )
        .ok();

    // 心跳配置与政策补丁也必须进备份：
    // 旧版只备份 5 张表，换机恢复后用户会发现「心跳令牌没了、自己导入的政策补丁也没了」，
    // 而这两项都是用户手工配置、无法从内置库重建的数据。
    let heartbeat: Option<Value> = conn
        .query_row(
            "SELECT enabled, COALESCE(token,''), COALESCE(endpoint,'') FROM heartbeat_config WHERE id=1",
            [],
            |row| {
                Ok(json!({
                    "enabled": row.get::<_, i64>(0)?,
                    "token": row.get::<_, String>(1)?,
                    "endpoint": row.get::<_, String>(2)?,
                }))
            },
        )
        .ok();

    let mut platform_overrides: Vec<Value> = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes FROM platform_overrides ORDER BY id")
            .map_err(|e| format!("查询政策补丁失败: {}", e))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(json!({
                    "id": row.get::<_, i64>(0)?,
                    "name": row.get::<_, String>(1)?,
                    "name_en": row.get::<_, String>(2)?,
                    "category": row.get::<_, String>(3)?,
                    "subtype": row.get::<_, String>(4)?,
                    "has_property": row.get::<_, i64>(5)? != 0,
                    "has_personality": row.get::<_, i64>(6)? != 0,
                    "inheritable": row.get::<_, i64>(7)? != 0,
                    "required_documents": row.get::<_, String>(8)?,
                    "process_description": row.get::<_, String>(9)?,
                    "estimated_difficulty": row.get::<_, String>(10)?,
                    "official_policy_url": row.get::<_, String>(11)?,
                    "estimated_time_days": row.get::<_, String>(12)?,
                    "notes": row.get::<_, String>(13)?,
                }))
            })
            .map_err(|e| format!("读取政策补丁失败: {}", e))?;
        for r in rows {
            platform_overrides.push(r.map_err(|e| format!("政策补丁解析失败: {}", e))?);
        }
    }

    let backup = json!({
        "app": "digilegacy",
        "version": 3,
        "created_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "tables": {
            "digital_assets": assets,
            "user_profile": profiles,
            "check_in_log": check_ins,
            "dms_config": dms,
            "security_config": security,
            "heartbeat_config": heartbeat,
            "platform_overrides": platform_overrides,
        }
    });

    serde_json::to_string_pretty(&backup).map_err(|e| format!("生成备份失败: {}", e))
}

/// 导入备份：覆盖用户数据与安全配置；导入后强制重新解锁
pub fn import_backup_json_db(content: &str) -> Result<i64, String> {
    use serde_json::Value;

    let v: Value = serde_json::from_str(content).map_err(|e| format!("备份文件解析失败: {}", e))?;
    if v.get("app").and_then(|x| x.as_str()) != Some("digilegacy") {
        return Err("不是有效的归零备份文件".to_string());
    }
    // v1 备份仍然可导入（向后兼容）：新表缺失时保持本地现状，
    // 旧备份里没有的 heartbeat/platform_overrides 按「未提供」处理。
    let version = v.get("version").and_then(|x| x.as_i64()).unwrap_or(0);
    if !(1..=3).contains(&version) {
        return Err("备份版本不受支持".to_string());
    }
    let tables = v.get("tables").ok_or("备份缺少数据表")?;

    // 加密边界：把「无 security_config 的备份」导进已启用的加密库，
    // 会把明文写进本应只有密文的列——这是不可接受的静默降级。
    // 锁定态直接拒绝；解锁态则用当前密钥把这些字段加密后再落库。
    let local_has_vault = get_security_config_db()?.is_some();
    let local_unlocked = crate::vault::is_unlocked();
    let backup_has_security = tables
        .get("security_config")
        .map(|x| x.is_object())
        .unwrap_or(false);
    let encrypt_plaintext_on_import = local_has_vault && !backup_has_security;
    if encrypt_plaintext_on_import && !local_unlocked {
        return Err(
            "该备份不含加密配置，而当前库已加密且处于锁定状态。请先解锁再导入，否则明文数据会绕过加密"
                .to_string(),
        );
    }
    let protect = |value: String| -> Result<String, String> {
        if !encrypt_plaintext_on_import || value.is_empty() {
            return Ok(value);
        }
        crate::vault::encrypt_if_needed(&value)
    };

    let sval = |val: &Value, k: &str| -> String {
        val.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };
    let ival = |val: &Value, k: &str| -> i64 { val.get(k).and_then(|x| x.as_i64()).unwrap_or(0) };
    let fval = |val: &Value, k: &str| -> f64 { val.get(k).and_then(|x| x.as_f64()).unwrap_or(0.0) };
    let oval = |val: &Value, k: &str| -> Option<String> {
        val.get(k).and_then(|x| x.as_str()).map(|s| s.to_string())
    };

    let assets = tables
        .get("digital_assets")
        .and_then(|x| x.as_array())
        .ok_or("备份缺少资产数据")?;
    let profiles: Vec<Value> = tables
        .get("user_profile")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();
    let check_ins: Vec<Value> = tables
        .get("check_in_log")
        .and_then(|x| x.as_array())
        .cloned()
        .unwrap_or_default();

    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("开启事务失败: {}", e))?;
    let result = (|| -> Result<i64, String> {
        conn.execute("DELETE FROM digital_assets", [])
            .map_err(|e| format!("清空资产失败: {}", e))?;
        conn.execute("DELETE FROM user_profile", [])
            .map_err(|e| format!("清空档案失败: {}", e))?;
        conn.execute("DELETE FROM check_in_log", [])
            .map_err(|e| format!("清空记录失败: {}", e))?;

        let mut count = 0i64;
        for (idx, a) in assets.iter().enumerate() {
            // 备份文件是外部输入：类别/处置/金额必须与命令层同样校验，
            // 否则一个手改过的 JSON 就能把非法枚举和负金额写进统计与报告。
            let category = sval(a, "category");
            validate_category(&category)
                .map_err(|e| format!("备份第 {} 条资产类别非法: {}", idx + 1, e))?;
            let disposal = sval(a, "disposal_method");
            validate_disposal(&disposal)
                .map_err(|e| format!("备份第 {} 条资产处置非法: {}", idx + 1, e))?;
            let value = fval(a, "estimated_value");
            if !value.is_finite() || value < 0.0 || value > 1e12 {
                return Err(format!(
                    "备份第 {} 条资产估值非法（要求 0..1e12 的有限数值，实际 {}）",
                    idx + 1,
                    value
                ));
            }
            conn.execute(
                "INSERT INTO digital_assets (id, platform_id, account_identifier, account_description, category, estimated_value, value_currency, disposal_method, heir_name, heir_contact, notes, credential_enc, is_confirmed, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    ival(a, "id"),
                    ival(a, "platform_id"),
                    protect(sval(a, "account_identifier"))?,
                    protect(sval(a, "account_description"))?,
                    category,
                    value,
                    { let c = sval(a, "value_currency"); if c.is_empty() { "CNY".to_string() } else { c } },
                    disposal,
                    protect(sval(a, "heir_name"))?,
                    protect(sval(a, "heir_contact"))?,
                    protect(sval(a, "notes"))?,
                    protect(sval(a, "credential_enc"))?,
                    ival(a, "is_confirmed"),
                    sval(a, "created_at"),
                    sval(a, "updated_at"),
                ],
            ).map_err(|e| format!("写入资产失败: {}", e))?;
            count += 1;
        }

        for p in &profiles {
            conn.execute(
                "INSERT INTO user_profile (id, full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email, check_in_interval_days, last_check_in, dms_enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    ival(p, "id"),
                    protect(sval(p, "full_name"))?,
                    protect(sval(p, "phone"))?,
                    protect(sval(p, "email"))?,
                    protect(sval(p, "id_last4"))?,
                    protect(sval(p, "emergency_contact_name"))?,
                    protect(sval(p, "emergency_contact_phone"))?,
                    protect(sval(p, "emergency_contact_relation"))?,
                    protect(sval(p, "emergency_contact_email"))?,
                    protect(sval(p, "heir_primary_name"))?,
                    protect(sval(p, "heir_primary_phone"))?,
                    protect(sval(p, "heir_primary_relation"))?,
                    protect(sval(p, "heir_primary_email"))?,
                    ival(p, "check_in_interval_days"),
                    oval(p, "last_check_in"),
                    ival(p, "dms_enabled"),
                    sval(p, "created_at"),
                    sval(p, "updated_at"),
                ],
            ).map_err(|e| format!("写入档案失败: {}", e))?;
        }

        for c in &check_ins {
            conn.execute(
                "INSERT INTO check_in_log (id, checked_at, status, note) VALUES (?1, ?2, ?3, ?4)",
                params![
                    ival(c, "id"),
                    sval(c, "checked_at"),
                    sval(c, "status"),
                    sval(c, "note")
                ],
            )
            .map_err(|e| format!("写入记录失败: {}", e))?;
        }

        if let Some(d) = tables.get("dms_config").filter(|x| x.is_object()) {
            conn.execute(
                "UPDATE dms_config SET enabled=?1, check_in_interval_days=?2, stage1_wait_days=?3, stage2_wait_days=?4, stage3_wait_days=?5, last_check_in=?6 WHERE id=1",
                params![
                    ival(d, "enabled"),
                    ival(d, "check_in_interval_days"),
                    ival(d, "stage1_wait_days"),
                    ival(d, "stage2_wait_days"),
                    ival(d, "stage3_wait_days"),
                    oval(d, "last_check_in"),
                ],
            ).map_err(|e| format!("写入DMS配置失败: {}", e))?;
        }

        if let Some(sec) = tables.get("security_config").filter(|x| x.is_object()) {
            // v3 备份带恢复封印：与新主密钥（备份里的 kdf_salt/verifier）成套，
            // 一起恢复才能保留「记得恢复码就能找回」的能力。
            // v1/v2 备份不含封印：必须显式清空，否则本地旧封印指向旧设备的主密钥，
            // 会让恢复码解出错误密钥，恢复链路静默断裂。
            let seal = sval(sec, "recovery_seal");
            conn.execute(
                "INSERT INTO security_config (id, kdf_salt, verifier, created_at, recovery_seal) VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET kdf_salt=?1, verifier=?2, recovery_seal=?4",
                params![
                    sval(sec, "kdf_salt"),
                    sval(sec, "verifier"),
                    Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    seal,
                ],
            ).map_err(|e| format!("写入安全配置失败: {}", e))?;
        }

        // v2：心跳配置与政策补丁。旧备份没有这两块 → 保持本地现状，
        // 不主动清空（清空会让「导入旧备份」顺手删掉用户的令牌与补丁）。
        if let Some(hb) = tables.get("heartbeat_config").filter(|x| x.is_object()) {
            conn.execute(
                "INSERT INTO heartbeat_config (id, enabled, token, endpoint) VALUES (1, ?1, ?2, ?3)
                 ON CONFLICT(id) DO UPDATE SET enabled=?1, token=?2, endpoint=?3",
                params![ival(hb, "enabled"), sval(hb, "token"), sval(hb, "endpoint")],
            )
            .map_err(|e| format!("写入心跳配置失败: {}", e))?;
        }

        if let Some(overrides) = tables.get("platform_overrides").and_then(|x| x.as_array()) {
            conn.execute("DELETE FROM platform_overrides", [])
                .map_err(|e| format!("清空政策补丁失败: {}", e))?;
            for o in overrides {
                conn.execute(
                    "INSERT INTO platform_overrides (id, name, name_en, category, subtype, has_property, has_personality, inheritable, required_documents, process_description, estimated_difficulty, official_policy_url, estimated_time_days, notes, updated_at)
                     VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",
                    params![
                        ival(o, "id"),
                        sval(o, "name"),
                        sval(o, "name_en"),
                        sval(o, "category"),
                        sval(o, "subtype"),
                        ival(o, "has_property"),
                        ival(o, "has_personality"),
                        ival(o, "inheritable"),
                        sval(o, "required_documents"),
                        sval(o, "process_description"),
                        sval(o, "estimated_difficulty"),
                        sval(o, "official_policy_url"),
                        sval(o, "estimated_time_days"),
                        sval(o, "notes"),
                        Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    ],
                ).map_err(|e| format!("写入政策补丁失败: {}", e))?;
            }
        }

        Ok(count)
    })();

    match result {
        Ok(n) => {
            conn.execute_batch("COMMIT")
                .map_err(|e| format!("提交事务失败: {}", e))?;
            crate::vault::clear_key();
            Ok(n)
        }
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// 将存量明文敏感字段加密（幂等；返回改动的资产行数）
pub fn encrypt_existing_data_db() -> Result<i64, String> {
    if !crate::vault::is_unlocked() {
        return Err("请先解锁".to_string());
    }
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let mut changed: i64 = 0;

    let mut stmt = conn.prepare(
        "SELECT id, account_identifier, account_description, heir_name, heir_contact, notes, credential_enc FROM digital_assets",
    ).map_err(|e| format!("查询资产失败: {}", e))?;
    let rows: Vec<(i64, String, String, String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            ))
        })
        .map_err(|e| format!("读取资产失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集资产失败: {}", e))?;

    for (id, ai, ad, hn, hc, nt, cr) in rows {
        let nai = crate::vault::encrypt_if_needed(&ai)?;
        let nad = crate::vault::encrypt_if_needed(&ad)?;
        let nhn = crate::vault::encrypt_if_needed(&hn)?;
        let nhc = crate::vault::encrypt_if_needed(&hc)?;
        let nnt = crate::vault::encrypt_if_needed(&nt)?;
        let ncr = crate::vault::encrypt_if_needed(&cr)?;
        if nai != ai || nad != ad || nhn != hn || nhc != hc || nnt != nt || ncr != cr {
            conn.execute(
                "UPDATE digital_assets SET account_identifier=?1, account_description=?2, heir_name=?3, heir_contact=?4, notes=?5, credential_enc=?6 WHERE id=?7",
                params![nai, nad, nhn, nhc, nnt, ncr, id],
            ).map_err(|e| format!("加密存量资产失败: {}", e))?;
            changed += 1;
        }
    }

    if let Some((pid, vals)) = select_profile_sensitive(conn)? {
        let mut new_vals = Vec::with_capacity(vals.len());
        let mut dirty = false;
        for v in &vals {
            let nv = crate::vault::encrypt_if_needed(v)?;
            if nv != *v {
                dirty = true;
            }
            new_vals.push(nv);
        }
        if dirty {
            update_profile_sensitive(conn, pid, &new_vals)
                .map_err(|e| format!("加密存量档案失败: {}", e))?;
            changed += 1;
        }
    }

    Ok(changed)
}

// ========== Check-in / Dead Man's Switch ==========

pub fn record_check_in_db() -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO check_in_log (checked_at, status, note) VALUES (?1, 'ok', '用户手动检查')",
        params![now],
    )
    .map_err(|e| format!("记录检查失败: {}", e))?;

    conn.execute(
        "UPDATE dms_config SET last_check_in=?1 WHERE id=1",
        params![now],
    )
    .map_err(|e| format!("更新DMS状态失败: {}", e))?;

    conn.execute(
        "UPDATE user_profile SET last_check_in=?1 WHERE id=(SELECT MAX(id) FROM user_profile)",
        params![now],
    )
    .map_err(|e| format!("更新用户签到时间失败: {}", e))?;

    Ok(())
}

pub fn get_check_in_status_db() -> Result<CheckInStatus, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let last_check_in: Option<String> = conn
        .query_row(
            "SELECT last_check_in FROM dms_config WHERE id=1",
            [],
            |row| row.get(0),
        )
        .ok();

    let interval: i64 = conn
        .query_row(
            "SELECT check_in_interval_days FROM dms_config WHERE id=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(30);

    let dms_enabled: bool = conn
        .query_row("SELECT enabled FROM dms_config WHERE id=1", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(|v| v != 0)
        .unwrap_or(false);

    let next_check_in_due = if let Some(ref last) = last_check_in {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S") {
            let next = dt + chrono::Duration::days(interval);
            Some(next.format("%Y-%m-%d").to_string())
        } else {
            None
        }
    } else {
        None
    };

    let days_until_due = if let Some(ref last) = last_check_in {
        if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(last, "%Y-%m-%d %H:%M:%S") {
            let next = dt + chrono::Duration::days(interval);
            let now = chrono::Local::now().naive_local();
            let duration = next.signed_duration_since(now);
            // 向上取整：`num_days()` 截断会把「还剩 20 小时」算成 0 天，
            // 界面据此判定逾期，用户会在到期前被误报（含系统通知）。
            Some((duration.num_seconds() as f64 / 86_400.0).ceil() as i64)
        } else {
            None
        }
    } else {
        None
    };

    Ok(CheckInStatus {
        last_check_in,
        interval_days: interval,
        dms_enabled,
        next_check_in_due,
        days_until_due,
        recent_check_ins: recent_check_ins_db(conn, 5).unwrap_or_default(),
    })
}

/// 最近若干次签到时间（新的在前）。读失败返回 Err，由调用方决定是否降级为空。
fn recent_check_ins_db(conn: &Connection, limit: usize) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT checked_at FROM check_in_log ORDER BY id DESC LIMIT ?1")
        .map_err(|e| format!("查询签到记录失败: {}", e))?;
    let rows = stmt
        .query_map(params![limit as i64], |r| r.get::<_, String>(0))
        .map_err(|e| format!("读取签到记录失败: {}", e))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("收集签到记录失败: {}", e))
}

pub fn set_dms_config_db(enabled: bool, interval_days: i64) -> Result<(), String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();

    conn.execute(
        "INSERT OR IGNORE INTO dms_config (id, enabled, check_in_interval_days, stage1_wait_days, stage2_wait_days, stage3_wait_days, last_check_in) VALUES (1, 0, 30, 3, 7, 14, NULL)",
        [],
    ).map_err(|e| format!("初始化DMS配置失败: {}", e))?;

    // 首次启用时写入起始锚点：没有锚点，逾期天数永远算不出来，
    // 用户会以为「已开启」而实际永远不会触发任何提醒（已由
    // enabled_dms_anchors_on_first_enable_instead_of_never_firing 锁定）。
    // 已有锚点则保持不变——改间隔不应重置倒计时。
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "UPDATE dms_config SET enabled=?1, check_in_interval_days=?2,          last_check_in = CASE WHEN ?1 = 1 AND last_check_in IS NULL THEN ?3 ELSE last_check_in END          WHERE id=1",
        params![if enabled { 1 } else { 0 }, interval_days, now],
    )
    .map_err(|e| format!("设置DMS失败: {}", e))?;

    Ok(())
}

pub fn get_dashboard_stats_db() -> Result<DashboardStats, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();

    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM digital_assets", [], |row| row.get(0))
        .map_err(|e| format!("统计失败: {}", e))?;

    let property_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets WHERE category='property'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let personality_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets WHERE category='personality'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let mixed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets WHERE category='mixed'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let confirmed_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets WHERE is_confirmed=1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    // 只累加非负有限值：数据可能来自手改备份或外部导入，
    // 负估值会把「总价值」算成负数（前端 totalValue 已有同样口径）。
    let total_value: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(CASE WHEN estimated_value > 0 AND estimated_value < 1e12 THEN estimated_value ELSE 0 END), 0.0) FROM digital_assets",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);
    let disposal_planned: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets WHERE disposal_method != 'no_action'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let last_check_in: Option<String> = conn
        .query_row("SELECT MAX(checked_at) FROM check_in_log", [], |row| {
            row.get(0)
        })
        .ok();
    let dms_enabled: bool = conn
        .query_row("SELECT enabled FROM dms_config WHERE id=1", [], |row| {
            row.get::<_, i64>(0)
        })
        .map(|v| v != 0)
        .unwrap_or(false);

    let completion = if total > 0 {
        disposal_planned * 100 / total
    } else {
        0
    };

    Ok(DashboardStats {
        total_assets: total,
        property_count,
        personality_count,
        mixed_count,
        confirmed_count,
        unconfirmed_count: total - confirmed_count,
        total_estimated_value: total_value,
        disposal_planned_count: disposal_planned,
        last_check_in,
        dms_enabled,
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
    /// 最近 5 次确认时间（新的在前）。签到日志此前只写不读，
    /// 用户无法判断「我真的点过吗、上次是什么时候」。
    pub recent_check_ins: Vec<String>,
}

// ========== 继承报告 ==========

pub fn get_inheritance_report_db() -> Result<InheritanceReport, String> {
    let user = get_user_profile_db()?.unwrap_or(UserProfile {
        id: None,
        full_name: "未设置".to_string(),
        phone: String::new(),
        email: String::new(),
        id_last4: String::new(),
        emergency_contact_name: String::new(),
        emergency_contact_phone: String::new(),
        emergency_contact_relation: String::new(),
        emergency_contact_email: String::new(),
        heir_primary_name: String::new(),
        heir_primary_phone: String::new(),
        heir_primary_relation: String::new(),
        heir_primary_email: String::new(),
        check_in_interval_days: 30,
        last_check_in: None,
        dms_enabled: false,
        created_at: String::new(),
        updated_at: String::new(),
    });

    let assets = get_all_assets_db()?;
    // 与仪表板同口径：只累加有限非负值，避免脏数据让报告金额为负。
    let total_value: f64 = assets
        .iter()
        .filter_map(|a| a.estimated_value)
        .filter(|v| v.is_finite() && *v > 0.0 && *v < 1e12)
        .sum();

    let property_count = assets.iter().filter(|a| a.category == "property").count() as i64;
    let personality_count = assets
        .iter()
        .filter(|a| a.category == "personality")
        .count() as i64;
    let mixed_count = assets.iter().filter(|a| a.category == "mixed").count() as i64;

    Ok(InheritanceReport {
        user,
        assets,
        total_estimated_value: total_value,
        property_assets_count: property_count,
        personality_assets_count: personality_count,
        mixed_assets_count: mixed_count,
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::AssetRequest;
    use crate::vault;

    /// 测试库路径。
    ///
    /// 注意：不要在 Drop 里删文件。数据库连接是进程级全局单例，测试并行执行时
    /// 一个用例的清理会关掉另一个用例正在用的连接（实测表现为 "数据库未初始化"
    /// 随机失败）。临时文件由 CI/本地定期清理，正确性优先。
    struct TempDb {
        path: PathBuf,
    }

    impl std::ops::Deref for TempDb {
        type Target = PathBuf;
        fn deref(&self) -> &PathBuf {
            &self.path
        }
    }

    impl AsRef<std::path::Path> for TempDb {
        fn as_ref(&self) -> &std::path::Path {
            self.path.as_path()
        }
    }

    fn fresh_db_path(tag: &str) -> TempDb {
        let mut p = std::env::temp_dir();
        p.push(format!("digilegacy_test_{}_{}.db", tag, std::process::id()));
        // 每个用例开头先清掉自己上次跑的残留（-wal/-shm 也必须拼在后缀上，
        // 不能用 with_extension，否则会把 .db 替换掉导致删不到）。
        let base = p.to_string_lossy().to_string();
        for suffix in ["", "-wal", "-shm", "-journal"] {
            let _ = std::fs::remove_file(format!("{}{}", base, suffix));
        }
        TempDb { path: p }
    }

    #[test]
    fn test_migration_encryption_and_credential_roundtrip() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("enc");
        init_database_at(path.clone()).unwrap();

        let req = AssetRequest {
            platform_id: 1,
            account_identifier: "13800000000".to_string(),
            account_description: "主账号".to_string(),
            category: "property".to_string(),
            estimated_value: Some(100.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("张三".to_string()),
            heir_contact: None,
            notes: "备注信息".to_string(),
        };
        let id = add_asset_db(&req).unwrap();

        let guard = get_conn().unwrap();
        let conn = guard.as_ref().unwrap();
        let raw_before: String = conn
            .query_row(
                "SELECT account_identifier FROM digital_assets WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(raw_before, "13800000000", "未启用主密码前应保持明文");
        drop(guard);

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("test-master-123", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db("dGVzdC1zYWx0", &verifier).unwrap();
        vault::set_key(key);

        let changed = encrypt_existing_data_db().unwrap();
        assert!(changed >= 1, "存量数据应被加密");

        let guard = get_conn().unwrap();
        let conn = guard.as_ref().unwrap();
        let raw_after: String = conn
            .query_row(
                "SELECT account_identifier FROM digital_assets WHERE id=?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(vault::is_encrypted(&raw_after), "库内应为 enc:v1: 密文");
        assert!(!raw_after.contains("13800000000"), "库内不得出现明文");
        drop(guard);

        let asset = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(asset.account_identifier, "13800000000");
        assert_eq!(asset.notes, "备注信息");
        assert_eq!(asset.heir_name.as_deref(), Some("张三"));

        set_asset_credential_db(id, "手机号+密码+密保").unwrap();
        assert_eq!(get_asset_credential_db(id).unwrap(), "手机号+密码+密保");
        assert!(get_asset_by_id_db(id).unwrap().unwrap().has_credential);

        set_asset_credential_db(id, "").unwrap();
        assert_eq!(get_asset_credential_db(id).unwrap(), "");
        assert!(!get_asset_by_id_db(id).unwrap().unwrap().has_credential);

        set_asset_credential_db(id, "再次保存").unwrap();
        vault::clear_key();
        assert!(
            get_asset_credential_db(id).is_err(),
            "锁定后不得返回明文凭据"
        );
        assert!(
            set_asset_credential_db(id, "锁定写入").is_err(),
            "锁定后不得写入新凭据"
        );
        let list = get_all_assets_db().unwrap();
        let mine = list.iter().find(|a| a.id == Some(id)).unwrap();
        assert!(
            mine.account_identifier.contains("加密数据"),
            "锁定后读取应为占位符"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 启用加密时若已有档案，`encrypt_existing_data_db` 必须完整加密全部 12 个档案字段。
    ///
    /// 历史缺陷：该函数的档案查询只 SELECT 了 11 个字段（漏 heir_primary_email），
    /// 取值循环又写成 1..11（10 个），随后 UPDATE 却按下标取到 new_vals[11] —
    /// 直接越界 panic。「先填档案、再设置主密码」是正常用户路径，会当场崩溃。
    #[test]
    fn encrypt_existing_data_covers_all_profile_fields() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("profile-encrypt");
        init_database_at(path.clone()).unwrap();

        save_user_profile_db(&UserProfile {
            full_name: "张三".to_string(),
            phone: "13800000000".to_string(),
            email: "zhangsan@example.com".to_string(),
            id_last4: "1234".to_string(),
            emergency_contact_name: "李四".to_string(),
            emergency_contact_phone: "13900000000".to_string(),
            emergency_contact_relation: "配偶".to_string(),
            emergency_contact_email: "lisi@example.com".to_string(),
            heir_primary_name: "王五".to_string(),
            heir_primary_phone: "13700000000".to_string(),
            heir_primary_relation: "子女".to_string(),
            heir_primary_email: "wangwu@example.com".to_string(),
            check_in_interval_days: 30,
            ..Default::default()
        })
        .unwrap();

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("master-pw-123", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db("dGVzdC1zYWx0", &verifier).unwrap();
        vault::set_key(key);

        let changed = encrypt_existing_data_db().expect("档案加密不得 panic 或报错");
        assert_eq!(changed, 1, "一份档案应计为一次改动");

        let raw_rows: Vec<String> = {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            let mut stmt = conn
                .prepare("SELECT full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email FROM user_profile")
                .unwrap();
            let rows = stmt
                .query_map([], |r| {
                    Ok((0..12)
                        .map(|i| {
                            r.get::<_, Option<String>>(i)
                                .ok()
                                .flatten()
                                .unwrap_or_default()
                        })
                        .collect::<Vec<String>>())
                })
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            rows.into_iter().next().unwrap()
        };
        for (idx, raw) in raw_rows.iter().enumerate() {
            assert!(
                vault::is_encrypted(raw),
                "第 {} 个档案字段应已加密，实际: {}",
                idx,
                raw
            );
        }

        let profile = get_user_profile_db().unwrap().unwrap();
        assert_eq!(profile.full_name, "张三");
        assert_eq!(profile.email, "zhangsan@example.com");
        assert_eq!(profile.emergency_contact_email, "lisi@example.com");
        assert_eq!(profile.heir_primary_email, "wangwu@example.com");
        assert_eq!(profile.heir_primary_relation, "子女");

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_change_password_and_backup_roundtrip() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine};
        vault::clear_key();
        let path = fresh_db_path("r2");
        init_database_at(path.clone()).unwrap();

        let req = AssetRequest {
            platform_id: 1,
            account_identifier: "acc-round2".to_string(),
            account_description: "描述".to_string(),
            category: "property".to_string(),
            estimated_value: Some(88.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("李四".to_string()),
            heir_contact: Some("13900000000".to_string()),
            notes: "备注R2".to_string(),
        };
        let id = add_asset_db(&req).unwrap();

        let salt1 = crate::crypto::generate_salt();
        let key1 = crate::crypto::derive_key_bytes("old-master-123", &salt1);
        let verifier1 = vault::make_verifier(&key1).unwrap();
        set_security_config_db(&b64.encode(salt1), &verifier1).unwrap();
        vault::set_key(key1);
        encrypt_existing_data_db().unwrap();
        set_asset_credential_db(id, "cred-round2").unwrap();

        let new_recovery_code =
            change_master_password_db("old-master-123", "new-master-456", "new-master-456")
                .unwrap();
        assert!(vault::is_unlocked(), "改密后应保持解锁状态");
        assert_eq!(get_asset_credential_db(id).unwrap(), "cred-round2");
        let asset = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(asset.account_identifier, "acc-round2");
        assert_eq!(asset.heir_contact.as_deref(), Some("13900000000"));

        let (salt2_b64, verifier2) = get_security_config_db().unwrap().unwrap();
        // 改密后安全配置应为 v2 盐（自描述前缀 + 60 万次迭代）
        assert!(
            salt2_b64.starts_with(crate::crypto::SALT_PREFIX_V2),
            "改密后盐应为 v2 格式"
        );
        let (salt2_bytes, salt2_iters) = crate::crypto::resolve_salt(&salt2_b64).unwrap();
        assert_eq!(salt2_iters, crate::crypto::PBKDF2_ITERATIONS_V2);
        let key2 = crate::crypto::derive_key_bytes_v("new-master-456", &salt2_bytes, salt2_iters);
        assert!(vault::verify_key(&verifier2, &key2), "新密码应可解锁");
        let key_old =
            crate::crypto::derive_key_bytes_v("old-master-123", &salt2_bytes, salt2_iters);
        assert!(!vault::verify_key(&verifier2, &key_old), "旧密码不得再解锁");

        // 新恢复密钥必须能解封"当前数据密钥"（改密轮换恢复码不得丢弃——历史缺陷回归）
        let seal_after = get_recovery_seal_db().unwrap();
        let unsealed =
            crate::recovery::unseal_with_recovery(&new_recovery_code, &seal_after).unwrap();
        assert_eq!(*unsealed, *key2, "新恢复码必须解出改密后的数据密钥");

        let backup = export_backup_json_db().unwrap();
        assert!(
            backup.contains("acc-round2") || backup.contains("enc:v1:"),
            "备份应含密文数据"
        );

        {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            conn.execute("DELETE FROM digital_assets", []).unwrap();
            conn.execute("DELETE FROM user_profile", []).unwrap();
        }
        assert!(
            get_asset_by_id_db(id).unwrap().is_none(),
            "清空后资产应不存在"
        );

        let restored = import_backup_json_db(&backup).unwrap();
        assert!(restored >= 1, "导入应恢复资产行数");
        assert!(
            get_asset_credential_db(id).is_err(),
            "导入后应处于锁定态，需重新解锁"
        );

        vault::set_key(crate::crypto::derive_key_bytes_v(
            "new-master-456",
            &salt2_bytes,
            salt2_iters,
        ));
        assert_eq!(
            get_asset_credential_db(id).unwrap(),
            "cred-round2",
            "重新解锁后凭据应可读"
        );

        vault::clear_key();
        let (_, verifier_after) = get_security_config_db().unwrap().unwrap();
        let key2b = crate::crypto::derive_key_bytes_v("new-master-456", &salt2_bytes, salt2_iters);
        assert!(
            vault::verify_key(&verifier_after, &key2b),
            "导入后新密码仍可解锁"
        );
        vault::set_key(key2b);
        let after = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(
            after.heir_name.as_deref(),
            Some("李四"),
            "档案字段应完整恢复"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 安全回归：v3 备份自带恢复密钥封印，导入后必须原样保留；
    /// 而 v1/v2 备份不含封印，导入时必须清空本地残留，否则恢复码会解出错误密钥。
    #[test]
    fn import_backup_preserves_v3_seal_and_clears_legacy_stale_seal() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine};
        vault::clear_key();
        let path = fresh_db_path("seal");
        init_database_at(path.clone()).unwrap();

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("seal-master-123", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        set_recovery_seal_db("RS2.本设备的恢复密钥封印").unwrap();
        vault::set_key(key);

        // v3：封印随备份往返
        let backup = export_backup_json_db().unwrap();
        import_backup_json_db(&backup).unwrap();
        assert_eq!(
            get_recovery_seal_db().unwrap(),
            "RS2.本设备的恢复密钥封印",
            "v3 备份必须保留恢复密钥封印"
        );

        // 旧 v2 备份（无 recovery_seal）：本地残留封印必须被清空
        let legacy: serde_json::Value = serde_json::from_str(&backup).unwrap();
        let mut legacy = legacy;
        legacy["version"] = serde_json::json!(2);
        legacy["tables"]["security_config"]
            .as_object_mut()
            .unwrap()
            .remove("recovery_seal");
        vault::clear_key();
        import_backup_json_db(&legacy.to_string()).unwrap();
        assert_eq!(
            get_recovery_seal_db().unwrap(),
            "",
            "v2 备份不含封印，导入后不得留下指向旧主密钥的封印"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 安全回归：已设置主密码但处于锁定状态时，敏感字段写入必须被拒绝
    /// （否则 vault::encrypt_if_needed 会静默明文落库）。尚未启用加密时放行。
    #[test]
    fn locked_vault_blocks_plaintext_writes() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine};
        vault::clear_key();
        let path = fresh_db_path("lock");
        init_database_at(path.clone()).unwrap();

        let req = AssetRequest {
            platform_id: 1,
            account_identifier: "13800000000".to_string(),
            account_description: "主账号".to_string(),
            category: "property".to_string(),
            estimated_value: Some(100.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("张三".to_string()),
            heir_contact: None,
            notes: "备注".to_string(),
        };

        // 未启用主密码：允许写入（存量明文模式）
        let id = add_asset_db(&req).unwrap();

        // 模拟"已设置主密码但处于锁定状态"
        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("lock-master-123", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        vault::clear_key();

        assert!(add_asset_db(&req).is_err(), "锁定态不得静默新增敏感字段");
        assert!(
            update_asset_db(id, &req).is_err(),
            "锁定态不得静默更新敏感字段"
        );
        assert!(
            save_user_profile_db(&UserProfile::default()).is_err(),
            "锁定态不得静默保存档案敏感字段"
        );

        // 解锁后恢复正常写入
        vault::set_key(key);
        assert!(add_asset_db(&req).is_ok(), "解锁后应可正常写入");

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 政策库补丁：导入后必须覆盖同 id 内置条目、并新增未知 id。
    #[test]
    fn platform_patch_overrides_builtin_entries() {
        let _g = vault::test_lock();
        let path = fresh_db_path("patch");
        init_database_at(path.clone()).unwrap();

        let builtin = get_all_platforms_db().unwrap();
        let target_id = builtin.iter().find(|p| p.name == "支付宝").unwrap().id;
        let before_len = builtin.len();

        let patch = serde_json::json!({
            "app": "digilegacy",
            "platforms": [
                {
                    "id": target_id,
                    "name": "支付宝（2026 政策更新）",
                    "category": "支付金融",
                    "required_documents": "死亡证明、继承公证书、继承人身份证",
                    "process_description": "更新后的办理路径",
                    "inheritable": true
                },
                {
                    "id": 9001,
                    "name": "某新平台",
                    "category": "社交通讯"
                }
            ]
        })
        .to_string();

        let (added, updated) = import_platform_overrides_db(&patch).unwrap();
        assert_eq!((added, updated), (1, 1), "一条新增、一条覆盖");

        let after = get_all_platforms_db().unwrap();
        assert_eq!(after.len(), before_len + 1, "补丁新增的条目应出现在列表里");

        let patched = after.iter().find(|p| p.id == target_id).unwrap();
        assert_eq!(
            patched.name, "支付宝（2026 政策更新）",
            "同 id 必须被补丁覆盖"
        );
        assert_eq!(patched.process_description, "更新后的办理路径");

        let newp = after.iter().find(|p| p.id == 9001).unwrap();
        assert_eq!(newp.name, "某新平台");
        assert_eq!(count_platform_overrides_db().unwrap(), 2);

        // 再次启动时的 sync_platforms 不得抹掉补丁
        init_database_at(path.clone()).unwrap();
        let after_restart = get_all_platforms_db().unwrap();
        assert_eq!(
            after_restart
                .iter()
                .find(|p| p.id == target_id)
                .unwrap()
                .name,
            "支付宝（2026 政策更新）",
            "重启后补丁仍须生效（内置同步不能覆盖用户补丁）"
        );

        let cleared = clear_platform_overrides_db().unwrap();
        assert_eq!(cleared, 2);
        let restored = get_all_platforms_db().unwrap();
        assert_eq!(
            restored.iter().find(|p| p.id == target_id).unwrap().name,
            "支付宝"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn platform_patch_rejects_invalid_payloads() {
        let _g = vault::test_lock();
        let path = fresh_db_path("patchbad");
        init_database_at(path.clone()).unwrap();

        assert!(import_platform_overrides_db("{not json")
            .unwrap_err()
            .contains("解析失败"));
        assert!(import_platform_overrides_db(
            r#"{"app":"other","platforms":[{"id":1,"name":"x","category":"y"}]}"#
        )
        .unwrap_err()
        .contains("不是归零政策库补丁"));
        assert!(
            import_platform_overrides_db(r#"{"app":"digilegacy","platforms":[]}"#)
                .unwrap_err()
                .contains("没有平台条目")
        );
        assert!(import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":0,"name":"x","category":"y"}]}"#
        )
        .unwrap_err()
        .contains("id 必须为正数"));
        assert!(import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":5,"name":"","category":"y"}]}"#
        )
        .unwrap_err()
        .contains("缺少名称"));

        assert_eq!(
            count_platform_overrides_db().unwrap(),
            0,
            "失败的导入不得留下部分数据"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn platform_patch_export_roundtrip() {
        let _g = vault::test_lock();
        let path = fresh_db_path("patchrt");
        init_database_at(path.clone()).unwrap();

        let payload = r#"{"app":"digilegacy","platforms":[{"id":9100,"name":"导出测试","category":"游戏","notes":"n"}]}"#;
        import_platform_overrides_db(payload).unwrap();

        let exported = export_platform_overrides_db().unwrap();
        let v: serde_json::Value = serde_json::from_str(&exported).unwrap();
        assert_eq!(v["app"], "digilegacy");
        assert_eq!(v["kind"], "platform-policy-patch");
        assert_eq!(v["platforms"].as_array().unwrap().len(), 1);
        assert_eq!(v["platforms"][0]["name"], "导出测试");

        // 导出的文件必须能被自己再导入（格式闭环）
        let (added, updated) = import_platform_overrides_db(&exported).unwrap();
        assert_eq!((added, updated), (0, 1), "重复导入应视为覆盖而非新增");

        let _ = std::fs::remove_file(&path);
    }

    /// 备份导出必须包含真实数据且可被自己再导入（闭环），而不是空壳 JSON。
    #[test]
    fn backup_export_contains_real_rows_and_reimports() {
        let _g = vault::test_lock();
        let path = fresh_db_path("backup");
        init_database_at(path.clone()).unwrap();

        let req = AssetRequest {
            platform_id: 1,
            account_identifier: "acc-backup".to_string(),
            account_description: "desc".to_string(),
            category: "property".to_string(),
            estimated_value: Some(321.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("继承人A".to_string()),
            heir_contact: Some("h@example.com".to_string()),
            notes: "backup-notes".to_string().to_string(),
        };
        let id = add_asset_db(&req).unwrap();

        let json = export_backup_json_db().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["app"], "digilegacy");
        let rows = v["tables"]["digital_assets"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "备份必须包含已登记资产");
        assert_eq!(rows[0]["account_identifier"], "acc-backup");
        assert_eq!(rows[0]["notes"], "backup-notes");

        // 清空后重新导入，数据必须回来
        delete_asset_db(id).unwrap();
        assert!(get_all_assets_db().unwrap().is_empty());

        import_backup_json_db(&json).unwrap();
        let restored = get_all_assets_db().unwrap();
        assert_eq!(restored.len(), 1);
        assert_eq!(restored[0].account_identifier, "acc-backup");
        assert_eq!(restored[0].heir_name.as_deref(), Some("继承人A"));

        let _ = std::fs::remove_file(&path);
    }

    /// 备份必须覆盖用户手工配置的两张表：心跳令牌与政策补丁。
    ///
    /// 历史缺陷：v1 备份只导出 5 张表，换机恢复后心跳令牌与导入的政策补丁全部丢失，
    /// 而这两项无法从内置库重建。同时导入时不得因为「旧备份没有这些字段」而清空本地数据。
    #[test]
    fn backup_v2_covers_heartbeat_and_platform_overrides() {
        let _g = vault::test_lock();
        let path = fresh_db_path("backupv2");
        init_database_at(path.clone()).unwrap();

        set_heartbeat_config_db(true, "hb-token-123", "https://hb.example.org").unwrap();
        import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":9101,"name":"备份补丁平台","category":"游戏"}]}"#,
        )
        .unwrap();

        let json = export_backup_json_db().unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["version"], 3, "新增表/封印后必须升版本号");
        assert_eq!(
            v["tables"]["heartbeat_config"]["token"], "hb-token-123",
            "心跳令牌必须进备份"
        );
        assert_eq!(
            v["tables"]["platform_overrides"].as_array().unwrap().len(),
            1,
            "政策补丁必须进备份"
        );

        // 清空后导入，两项都要回来
        set_heartbeat_config_db(false, "", "").unwrap();
        clear_platform_overrides_db().unwrap();
        import_backup_json_db(&json).unwrap();

        let (enabled, token, endpoint) = get_heartbeat_config_db().unwrap();
        assert!(enabled);
        assert_eq!(token, "hb-token-123");
        assert_eq!(endpoint, "https://hb.example.org");
        assert_eq!(count_platform_overrides_db().unwrap(), 1);

        let _ = std::fs::remove_file(&path);
    }

    /// 旧版 v1 备份仍可导入，且不会清空本地 v2 才有的配置。
    #[test]
    fn legacy_v1_backup_imports_without_wiping_v2_config() {
        let _g = vault::test_lock();
        let path = fresh_db_path("backupv1");
        init_database_at(path.clone()).unwrap();

        set_heartbeat_config_db(true, "keep-me", "https://hb.example.org").unwrap();
        import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":9102,"name":"保留补丁","category":"游戏"}]}"#,
        )
        .unwrap();

        let v1 = serde_json::json!({
            "app": "digilegacy",
            "version": 1,
            "tables": {
                "digital_assets": [],
                "user_profile": [],
                "check_in_log": [],
                "dms_config": null,
                "security_config": null
            }
        })
        .to_string();
        import_backup_json_db(&v1).unwrap();

        let (enabled, token, _) = get_heartbeat_config_db().unwrap();
        assert!(enabled, "导入 v1 备份不得关闭心跳");
        assert_eq!(token, "keep-me");
        assert_eq!(
            count_platform_overrides_db().unwrap(),
            1,
            "不得清空政策补丁"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 明文备份不得绕过加密：锁定态拒绝，解锁态必须逐字段加密后再入库。
    #[test]
    fn plaintext_backup_never_lands_as_plaintext_in_encrypted_vault() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine};
        vault::clear_key();
        let path = fresh_db_path("plainimport");
        init_database_at(path.clone()).unwrap();

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("plain-import-pw", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db(&b64.encode(salt), &verifier).unwrap();

        let plaintext_backup = serde_json::json!({
            "app": "digilegacy",
            "version": 1,
            "tables": {
                "digital_assets": [{
                    "id": 1,
                    "platform_id": 1,
                    "account_identifier": "明文账号",
                    "account_description": "明文描述",
                    "category": "property",
                    "estimated_value": 10.0,
                    "value_currency": "CNY",
                    "disposal_method": "inherit",
                    "heir_name": "明文继承人",
                    "heir_contact": "13800000000",
                    "notes": "明文备注",
                    "credential_enc": "明文凭据",
                    "is_confirmed": 1,
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }],
                "user_profile": [{
                    "id": 1,
                    "full_name": "明文姓名",
                    "phone": "13800000000",
                    "email": "plain@example.com",
                    "created_at": "2026-01-01 00:00:00",
                    "updated_at": "2026-01-01 00:00:00"
                }],
                "check_in_log": [],
                "dms_config": null,
                "security_config": null
            }
        })
        .to_string();

        // 锁定态：必须拒绝，且不能改动已有数据
        let err = import_backup_json_db(&plaintext_backup).unwrap_err();
        assert!(
            err.contains("请先解锁"),
            "锁定态导入明文备份必须 fail-closed，实际: {}",
            err
        );

        // 解锁态：允许导入，但每个敏感字段都必须已加密
        vault::set_key(key.clone());
        import_backup_json_db(&plaintext_backup).unwrap();

        let guard = get_conn().unwrap();
        let conn = guard.as_ref().unwrap();
        let raw_identifier: String = conn
            .query_row(
                "SELECT account_identifier FROM digital_assets WHERE id=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            crate::vault::is_encrypted(&raw_identifier),
            "明文账号必须被加密后落库"
        );
        let raw_name: String = conn
            .query_row("SELECT full_name FROM user_profile WHERE id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert!(
            crate::vault::is_encrypted(&raw_name),
            "档案字段必须被加密后落库"
        );
        drop(guard);

        // 导入成功后密钥被清空（强制重新解锁），重新解锁才能验证明文回读
        vault::set_key(crate::crypto::derive_key_bytes("plain-import-pw", &salt));
        let assets = get_all_assets_db().unwrap();
        assert_eq!(assets[0].account_identifier, "明文账号");
        assert_eq!(
            get_user_profile_db().unwrap().unwrap().full_name,
            "明文姓名"
        );

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 备份是外部输入：非法类别/处置/负金额必须在事务内被拒绝，且不留半截数据。
    /// 旧实现直接把 JSON 里的字符串写进资产表，手改过的备份能污染统计与报告。
    #[test]
    fn backup_import_rejects_invalid_asset_fields() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("backup_invalid");
        init_database_at(path.clone()).unwrap();

        let id = add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "keep-me".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(10.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        })
        .unwrap();
        assert_eq!(get_all_assets_db().unwrap().len(), 1);

        let base_asset = serde_json::json!({
            "id": 2,
            "platform_id": 1,
            "account_identifier": "bad",
            "account_description": "",
            "category": "property",
            "estimated_value": 1.0,
            "value_currency": "CNY",
            "disposal_method": "inherit",
            "heir_name": "",
            "heir_contact": "",
            "notes": "",
            "credential_enc": "",
            "is_confirmed": 1,
            "created_at": "2026-01-01 00:00:00",
            "updated_at": "2026-01-01 00:00:00"
        });
        let mk = |patch: serde_json::Value| {
            let mut asset = base_asset.clone();
            if let Some(obj) = patch.as_object() {
                for (k, v) in obj {
                    asset[k] = v.clone();
                }
            }
            serde_json::json!({
                "app": "digilegacy",
                "version": 3,
                "tables": {
                    "digital_assets": [asset],
                    "user_profile": [],
                    "check_in_log": [],
                    "dms_config": null,
                    "security_config": null
                }
            })
            .to_string()
        };

        for (label, patch) in [
            (
                "非法类别",
                serde_json::json!({"category": "not_a_category"}),
            ),
            (
                "非法处置",
                serde_json::json!({"disposal_method": "sell_everything"}),
            ),
            ("负金额", serde_json::json!({"estimated_value": -5.0})),
            ("超大金额", serde_json::json!({"estimated_value": 1e15})),
        ] {
            let err = import_backup_json_db(&mk(patch)).unwrap_err();
            assert!(!err.is_empty(), "{} 必须给出错误信息", label);
            assert_eq!(
                get_all_assets_db().unwrap().len(),
                1,
                "{} 被拒绝后原数据必须完好",
                label
            );
            assert_eq!(
                get_asset_by_id_db(id).unwrap().unwrap().account_identifier,
                "keep-me",
                "{} 拒绝后不得改动既有资产",
                label
            );
        }

        // 合法备份仍然可以导入
        import_backup_json_db(&mk(serde_json::json!({}))).unwrap();
        assert_eq!(
            get_all_assets_db().unwrap().len(),
            1,
            "合法备份应替换为 1 条"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 安全回归：资产详情页拿到占位符后点「保存修改」，必须被拒绝且原始密文不受影响。
    ///
    /// 历史缺陷：`row_to_asset` 用占位符降级，前端把整张表单原样提交，
    /// `update_asset_db` 就把占位符当成明文重新加密写回，真实账号被永久覆盖。
    #[test]
    fn asset_placeholder_cannot_be_written_back() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
        vault::clear_key();
        let path = fresh_db_path("asset_placeholder");
        init_database_at(path.clone()).unwrap();

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("asset-pw", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        vault::set_key(key.clone());

        let id = add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "real-account".to_string(),
            account_description: "真实描述".to_string(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("真实继承人".to_string()),
            heir_contact: Some("13800000000".to_string()),
            notes: "真实备注".to_string(),
        })
        .unwrap();

        let wrong = crate::crypto::derive_key_bytes("other-pw", &salt);
        vault::set_key(wrong);
        let loaded = get_asset_by_id_db(id).unwrap().unwrap();
        assert!(
            crate::vault::is_placeholder(&loaded.account_identifier),
            "错钥匙时必须降级为占位符"
        );

        // 前端资产详情点「保存修改」时会把这个占位符原样提交
        let err = update_asset_db(
            id,
            &AssetRequest {
                platform_id: loaded.platform_id,
                account_identifier: loaded.account_identifier.clone(),
                account_description: loaded.account_description.clone(),
                category: loaded.category.clone(),
                estimated_value: loaded.estimated_value,
                disposal_method: loaded.disposal_method.clone(),
                heir_name: loaded.heir_name.clone(),
                heir_contact: loaded.heir_contact.clone(),
                notes: loaded.notes.clone(),
            },
        )
        .unwrap_err();
        assert!(
            err.contains("解锁") && err.contains("未写入"),
            "必须明确拒绝并说明未写入，实际: {}",
            err
        );

        // 原始密文必须完好
        vault::set_key(key);
        let after = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(after.account_identifier, "real-account", "原始账号必须还在");
        assert_eq!(after.heir_name.as_deref(), Some("真实继承人"));
        assert_eq!(after.notes, "真实备注");

        // 正常编辑（非占位符）仍然可以保存
        update_asset_db(
            id,
            &AssetRequest {
                platform_id: 1,
                account_identifier: "edited-account".to_string(),
                account_description: String::new(),
                category: "property".to_string(),
                estimated_value: Some(2.0),
                disposal_method: "memorialize".to_string(),
                heir_name: None,
                heir_contact: None,
                notes: String::new(),
            },
        )
        .unwrap();
        assert_eq!(
            get_asset_by_id_db(id).unwrap().unwrap().account_identifier,
            "edited-account"
        );

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 安全回归：档案页拿到占位符后点「保存档案」，必须被拒绝且原始密文完好。
    #[test]
    fn profile_placeholder_cannot_be_written_back() {
        let _g = vault::test_lock();
        use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
        vault::clear_key();
        let path = fresh_db_path("placeholder_wb");
        init_database_at(path.clone()).unwrap();

        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("wb-pw", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        set_security_config_db(&b64.encode(salt), &verifier).unwrap();
        vault::set_key(key.clone());

        let mut profile = UserProfile {
            id: None,
            full_name: "真实姓名".to_string(),
            phone: "13800000000".to_string(),
            email: "real@example.com".to_string(),
            id_last4: "1234".to_string(),
            emergency_contact_name: "紧急".to_string(),
            emergency_contact_phone: "13900000000".to_string(),
            emergency_contact_relation: "子女".to_string(),
            emergency_contact_email: "e@example.com".to_string(),
            heir_primary_name: "继承人".to_string(),
            heir_primary_phone: "13700000000".to_string(),
            heir_primary_relation: "子女".to_string(),
            heir_primary_email: "h@example.com".to_string(),
            check_in_interval_days: 30,
            last_check_in: None,
            dms_enabled: false,
            created_at: String::new(),
            updated_at: String::new(),
        };
        save_user_profile_db(&profile).unwrap();

        // 换一把钥匙：库内密文用旧钥匙，内存里是另一把 → 读回全是占位符
        let wrong = crate::crypto::derive_key_bytes("other-pw", &salt);
        vault::set_key(wrong);
        let loaded = get_user_profile_db().unwrap().unwrap();
        assert!(
            crate::vault::is_placeholder(&loaded.full_name),
            "错钥匙/未解锁时必须降级为占位符"
        );

        // 用户在界面上点保存（占位符被原样提交）
        profile.id = loaded.id;
        profile.full_name = loaded.full_name.clone();
        profile.phone = loaded.phone.clone();
        let err = save_user_profile_db(&profile).unwrap_err();
        assert!(
            err.contains("解锁") && err.contains("未写入"),
            "必须明确拒绝，实际: {}",
            err
        );

        // 原始密文完好
        vault::set_key(key);
        let after = get_user_profile_db().unwrap().unwrap();
        assert_eq!(after.full_name, "真实姓名");
        assert_eq!(after.phone, "13800000000");
        assert_eq!(after.heir_primary_name, "继承人");

        // 正常保存仍然可用
        let mut fixed = after.clone();
        fixed.full_name = "改名后".to_string();
        save_user_profile_db(&fixed).unwrap();
        assert_eq!(get_user_profile_db().unwrap().unwrap().full_name, "改名后");

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 回归：政策补丁新增的平台必须能直接登记资产。
    ///
    /// 历史缺陷：补丁只写 platform_overrides，而 digital_assets.platform_id 的外键
    /// 指向 platforms 表，于是「界面能选、一保存就 FOREIGN KEY constraint failed」。
    #[test]
    fn patch_only_platform_can_hold_assets() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("patch_fk");
        init_database_at(path.clone()).unwrap();

        import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":9001,"name":"补丁专属平台","category":"游戏"}]}"#,
        )
        .unwrap();

        let listed = get_all_platforms_db().unwrap();
        assert!(listed.iter().any(|p| p.id == 9001), "补丁平台应出现在列表");

        let id = add_asset_db(&AssetRequest {
            platform_id: 9001,
            account_identifier: "acc".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        })
        .expect("补丁平台必须能登记资产（外键依赖已通过镜像解决）");

        let asset = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(asset.platform_name, "补丁专属平台", "平台名应来自补丁");

        // 补丁覆盖内置平台时也不得破坏既有资产的外键
        import_platform_overrides_db(
            r#"{"app":"digilegacy","platforms":[{"id":1,"name":"支付宝（改名）","category":"支付金融"}]}"#,
        )
        .unwrap();
        let builtin_id = add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "builtin".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        })
        .unwrap();
        assert_eq!(
            get_asset_by_id_db(builtin_id)
                .unwrap()
                .unwrap()
                .platform_name,
            "支付宝（改名）",
            "补丁覆盖后的名称必须用于展示"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 仪表板与报告的总价值口径：负值与超大值都必须被排除，
    /// 否则一条脏数据就能让「我的遗产总值」变成负数或荒谬数字。
    #[test]
    fn totals_ignore_negative_and_absurd_values() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("totals");
        init_database_at(path.clone()).unwrap();

        for (identifier, value) in [
            ("valid", 100.0),
            ("zero", 0.0),
            ("negative", -5.0),
            ("absurd", 1e15),
        ] {
            add_asset_db(&AssetRequest {
                platform_id: 1,
                account_identifier: identifier.to_string(),
                account_description: String::new(),
                category: "property".to_string(),
                estimated_value: Some(value),
                disposal_method: "inherit".to_string(),
                heir_name: None,
                heir_contact: None,
                notes: String::new(),
            })
            .unwrap();
        }

        let stats = get_dashboard_stats_db().unwrap();
        assert_eq!(stats.total_assets, 4, "条数统计不受金额影响");
        assert!(
            (stats.total_estimated_value - 100.0).abs() < 1e-6,
            "只应累加合法的 100，实际 {}",
            stats.total_estimated_value
        );

        let report = get_inheritance_report_db().unwrap();
        assert!(
            (report.total_estimated_value - 100.0).abs() < 1e-6,
            "报告口径必须与仪表板一致，实际 {}",
            report.total_estimated_value
        );

        let _ = std::fs::remove_file(&path);
    }

    /// schema 版本号必须写入，且新库与再次打开都应保持同一版本。
    #[test]
    fn schema_version_is_recorded() {
        let _g = vault::test_lock();
        let path = fresh_db_path("schemaver");
        init_database_at(path.clone()).unwrap();

        let guard = get_conn().unwrap();
        let v: i64 = guard
            .as_ref()
            .unwrap()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, SCHEMA_VERSION);
        drop(guard);

        // 同一路径再次初始化（模拟重启）不应回退版本
        init_database_at(path.clone()).unwrap();
        let guard = get_conn().unwrap();
        let v2: i64 = guard
            .as_ref()
            .unwrap()
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v2, SCHEMA_VERSION);

        let _ = std::fs::remove_file(&path);
    }

    /// 继承报告必须真的由库内数据生成，而不是固定模板。
    #[test]
    fn inheritance_report_reflects_actual_assets() {
        let _g = vault::test_lock();
        let path = fresh_db_path("report");
        init_database_at(path.clone()).unwrap();

        assert_eq!(get_inheritance_report_db().unwrap().assets.len(), 0);

        add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "r1".to_string(),
            account_description: "".to_string(),
            category: "property".to_string(),
            estimated_value: Some(1000.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("甲".to_string()),
            heir_contact: None,
            notes: String::new(),
        })
        .unwrap();
        add_asset_db(&AssetRequest {
            platform_id: 2,
            account_identifier: "r2".to_string(),
            account_description: "".to_string(),
            category: "personality".to_string(),
            estimated_value: Some(500.0),
            disposal_method: "memorialize".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        })
        .unwrap();

        let report = get_inheritance_report_db().unwrap();
        assert_eq!(report.assets.len(), 2, "报告必须包含全部资产");
        assert_eq!(report.property_assets_count, 1);
        assert_eq!(report.personality_assets_count, 1);
        assert_eq!(report.total_estimated_value, 1500.0);
        assert!(!report.generated_at.is_empty(), "报告必须有生成时间");

        let _ = std::fs::remove_file(&path);
    }

    /// 新增资产时每个敏感字段必须立即以密文落库。
    ///
    /// 此前的加密断言全部集中在 `encrypt_existing_data_db`（存量收编），
    /// 「已解锁状态下 add_asset 写入」这条主路径没有断言——
    /// 把 encrypt_if_needed 从 add_asset_db 摘掉，59 个测试仍全绿（变异实测确认）。
    #[test]
    fn add_asset_encrypts_every_sensitive_column_on_write() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("write_enc");
        init_database_at(path.clone()).unwrap();

        // 启用加密并解锁
        let salt = crate::crypto::generate_salt();
        let key = crate::crypto::derive_key_bytes("write-enc-master", &salt);
        let verifier = vault::make_verifier(&key).unwrap();
        let salt_b64 = {
            use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
            b64.encode(salt)
        };
        set_security_config_db(&salt_b64, &verifier).unwrap();
        vault::set_key(key);

        let id = add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "PLAINTEXT-ID".to_string(),
            account_description: "PLAINTEXT-DESC".to_string(),
            category: "property".to_string(),
            estimated_value: Some(42.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("PLAINTEXT-HEIR".to_string()),
            heir_contact: Some("PLAINTEXT-CONTACT".to_string()),
            notes: "PLAINTEXT-NOTES".to_string(),
        })
        .unwrap();

        // 直接读原始行，绕过读取层的自动解密
        let guard = get_conn().unwrap();
        let conn = guard.as_ref().unwrap();
        let row: (String, String, String, String, String) = conn
            .query_row(
                "SELECT account_identifier, account_description, COALESCE(heir_name,''), COALESCE(heir_contact,''), notes FROM digital_assets WHERE id=?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .unwrap();
        drop(guard);

        let (acc, desc, heir, contact, notes) = row;
        for (name, value, plain) in [
            ("account_identifier", &acc, "PLAINTEXT-ID"),
            ("account_description", &desc, "PLAINTEXT-DESC"),
            ("heir_name", &heir, "PLAINTEXT-HEIR"),
            ("heir_contact", &contact, "PLAINTEXT-CONTACT"),
            ("notes", &notes, "PLAINTEXT-NOTES"),
        ] {
            assert!(
                vault::is_encrypted(value),
                "{} 必须以 enc:v1 密文落库，实际: {}",
                name,
                value
            );
            assert!(!value.contains(plain), "{} 不得以明文出现在库里", name);
        }

        // 读取层仍应解密还原
        let asset = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(asset.account_identifier, "PLAINTEXT-ID");
        assert_eq!(asset.notes, "PLAINTEXT-NOTES");
        assert_eq!(asset.heir_contact.as_deref(), Some("PLAINTEXT-CONTACT"));

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }

    /// 签到必须复位 DMS 计时。
    ///
    /// 删掉 `UPDATE dms_config SET last_check_in` 这一句，60 个测试仍全绿
    /// （变异实测确认）。而真实后果是：用户点了「我还安好」，三级升级链
    /// 却仍按旧时间继续升级，持续误报——这是会直接伤害用户的高危缺口。
    #[test]
    fn check_in_resets_dms_countdown() {
        let _g = vault::test_lock();
        let path = fresh_db_path("checkin");
        init_database_at(path.clone()).unwrap();

        // 造一个「已逾期」的初始状态：上次签到在 100 天前，间隔 30 天
        let old = (chrono::Local::now() - chrono::Duration::days(100))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            conn.execute("UPDATE dms_config SET enabled=1, check_in_interval_days=30, last_check_in=?1 WHERE id=1", params![old])
                .unwrap();
        }
        let before = get_check_in_status_db().unwrap();
        assert!(before.days_until_due.unwrap_or(0) < 0, "前置：应已逾期");

        record_check_in_db().unwrap();

        let after = get_check_in_status_db().unwrap();
        assert!(
            after.days_until_due.unwrap_or(-1) > 0,
            "签到后剩余天数必须重新变正，实际 {:?}",
            after.days_until_due
        );
        assert_ne!(
            after.last_check_in.as_deref(),
            Some(old.as_str()),
            "dms_config.last_check_in 必须被更新"
        );

        // 签到日志也必须有记录
        let logs: i64 = {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            conn.query_row("SELECT COUNT(*) FROM check_in_log", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(logs, 1, "签到必须写日志");

        let _ = std::fs::remove_file(&path);
    }

    /// 报告的类别统计必须准确，尤其 mixed 不能被算进 property。
    #[test]
    fn inheritance_report_counts_mixed_separately() {
        let _g = vault::test_lock();
        let path = fresh_db_path("report_mixed");
        init_database_at(path.clone()).unwrap();

        for (cat, val) in [
            ("property", 100.0),
            ("personality", 50.0),
            ("mixed", 25.0),
            ("mixed", 25.0),
        ] {
            add_asset_db(&AssetRequest {
                platform_id: 1,
                account_identifier: format!("acc-{}", cat),
                account_description: String::new(),
                category: cat.to_string(),
                estimated_value: Some(val),
                disposal_method: "inherit".to_string(),
                heir_name: None,
                heir_contact: None,
                notes: String::new(),
            })
            .unwrap();
        }

        let report = get_inheritance_report_db().unwrap();
        assert_eq!(
            report.property_assets_count, 1,
            "property 只应统计 property"
        );
        assert_eq!(
            report.personality_assets_count, 1,
            "personality 只应统计 personality"
        );
        assert_eq!(report.mixed_assets_count, 2, "mixed 必须单独统计");
        assert_eq!(report.assets.len(), 4);
        assert_eq!(report.total_estimated_value, 200.0);

        let _ = std::fs::remove_file(&path);
    }

    /// 备份里含签到记录时必须能完整导入。
    ///
    /// 此前 `backup_export_contains_real_rows_and_reimports` 没先签到，
    /// 所以永远造不出「check_in_log 有行」的场景；删掉导入里的
    /// `DELETE FROM check_in_log` 后 62 个测试仍全绿，而真实后果是：
    /// 任何用过签到的用户，备份在灾难恢复时都会因主键冲突整体失败。
    #[test]
    fn backup_with_checkin_log_reimports_cleanly() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("backup_checkin");
        init_database_at(path.clone()).unwrap();

        add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "acc-ci".to_string(),
            account_description: String::new(),
            category: "property".to_string(),
            estimated_value: Some(1.0),
            disposal_method: "inherit".to_string(),
            heir_name: None,
            heir_contact: None,
            notes: String::new(),
        })
        .unwrap();
        // 产生至少一条签到记录（这是旧测试缺失的前置）
        record_check_in_db().unwrap();
        record_check_in_db().unwrap();

        let logs_before: i64 = {
            let guard = get_conn().unwrap();
            guard
                .as_ref()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM check_in_log", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(logs_before, 2);

        let json = export_backup_json_db().unwrap();
        assert!(json.contains("check_in_log"), "备份必须包含签到表");

        // 再签到一次，让本地记录比备份多，验证导入是整体替换而非追加
        record_check_in_db().unwrap();

        import_backup_json_db(&json).expect("含签到记录的备份必须可导入");

        let logs_after: i64 = {
            let guard = get_conn().unwrap();
            guard
                .as_ref()
                .unwrap()
                .query_row("SELECT COUNT(*) FROM check_in_log", [], |r| r.get(0))
                .unwrap()
        };
        assert_eq!(
            logs_after, 2,
            "导入后签到记录应与备份一致（不追加、不冲突）"
        );

        let assets = get_all_assets_db().unwrap();
        assert_eq!(assets.len(), 1, "资产也要一并恢复");

        let _ = std::fs::remove_file(&path);
    }

    /// 按类别查询：只返回该类别，且按估值降序。
    #[test]
    fn assets_by_category_filters_and_sorts() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("bycat");
        init_database_at(path.clone()).unwrap();

        for (cat, val) in [
            ("property", 100.0),
            ("property", 900.0),
            ("personality", 500.0),
        ] {
            add_asset_db(&AssetRequest {
                platform_id: 1,
                account_identifier: format!("acc-{}-{}", cat, val),
                account_description: String::new(),
                category: cat.to_string(),
                estimated_value: Some(val),
                disposal_method: "inherit".to_string(),
                heir_name: None,
                heir_contact: None,
                notes: String::new(),
            })
            .unwrap();
        }

        let prop = get_assets_by_category_db("property").unwrap();
        assert_eq!(prop.len(), 2, "只应返回 property");
        assert!(prop.iter().all(|a| a.category == "property"));
        assert_eq!(prop[0].estimated_value, Some(900.0), "必须按估值降序");
        assert_eq!(prop[1].estimated_value, Some(100.0));

        let per = get_assets_by_category_db("personality").unwrap();
        assert_eq!(per.len(), 1);

        let none = get_assets_by_category_db("mixed").unwrap();
        assert!(none.is_empty());

        let _ = std::fs::remove_file(&path);
    }

    /// DMS 配置读写：enabled 与 interval 必须持久化并回读一致。
    #[test]
    fn dms_config_roundtrip() {
        let _g = vault::test_lock();
        let path = fresh_db_path("dmscfg");
        init_database_at(path.clone()).unwrap();

        // 默认值
        let st = get_check_in_status_db().unwrap();
        assert_eq!(st.interval_days, 30);
        assert!(!st.dms_enabled);

        set_dms_config_db(true, 14).unwrap();
        let st = get_check_in_status_db().unwrap();
        assert!(st.dms_enabled);
        assert_eq!(st.interval_days, 14);

        set_dms_config_db(false, 7).unwrap();
        let st = get_check_in_status_db().unwrap();
        assert!(!st.dms_enabled);
        assert_eq!(st.interval_days, 7);

        let _ = std::fs::remove_file(&path);
    }

    /// 剩余天数必须向上取整：到期前不到 24 小时应显示为 1 天而不是 0 天。
    /// 旧实现用 `num_days()` 截断，界面把「还剩 20 小时」判成逾期，会误发提醒。
    #[test]
    fn days_until_due_rounds_up_instead_of_truncating() {
        let _g = vault::test_lock();
        let path = fresh_db_path("dueceil");
        init_database_at(path.clone()).unwrap();

        // 上次签到在 29 天 4 小时前，间隔 30 天 → 还剩 20 小时
        let last = (chrono::Local::now() - chrono::Duration::hours(29 * 24 + 4))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            conn.execute(
                "UPDATE dms_config SET enabled=1, check_in_interval_days=30, last_check_in=?1 WHERE id=1",
                params![last],
            )
            .unwrap();
        }

        let st = get_check_in_status_db().unwrap();
        assert_eq!(
            st.days_until_due,
            Some(1),
            "到期前不足一天必须向上取整为 1，不能是 0（会被判成逾期）"
        );

        let _ = std::fs::remove_file(&path);
    }

    /// 签到历史：新的在前、只取指定条数，并在 status 里回传。
    #[test]
    fn check_in_history_is_newest_first_and_capped() {
        let _g = vault::test_lock();
        let path = fresh_db_path("checkin_log");
        init_database_at(path.clone()).unwrap();

        for _ in 0..7 {
            record_check_in_db().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(1100));
        }

        let st = get_check_in_status_db().unwrap();
        assert_eq!(st.recent_check_ins.len(), 5, "只回传最近 5 条");
        assert_eq!(
            Some(st.recent_check_ins[0].clone()),
            st.last_check_in,
            "第一条必须是最新签到"
        );
        let mut sorted = st.recent_check_ins.clone();
        sorted.sort();
        sorted.reverse();
        assert_eq!(sorted, st.recent_check_ins, "必须按时间倒序");

        let _ = std::fs::remove_file(&path);
    }

    /// 仪表板统计：各类计数与完成度必须与库内真实数据一致。
    #[test]
    fn dashboard_stats_reflect_real_data() {
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("stats");
        init_database_at(path.clone()).unwrap();

        let empty = get_dashboard_stats_db().unwrap();
        assert_eq!(empty.total_assets, 0);
        assert_eq!(empty.completion_percentage, 0, "空库完成度应为 0，不能除零");

        // 两条已设处置（inherit）、一条未设（no_action）
        for (method, cat) in [
            ("inherit", "property"),
            ("inherit", "personality"),
            ("no_action", "mixed"),
        ] {
            add_asset_db(&AssetRequest {
                platform_id: 1,
                account_identifier: format!("a-{}-{}", method, cat),
                account_description: String::new(),
                category: cat.to_string(),
                estimated_value: Some(10.0),
                disposal_method: method.to_string(),
                heir_name: None,
                heir_contact: None,
                notes: String::new(),
            })
            .unwrap();
        }

        let s = get_dashboard_stats_db().unwrap();
        assert_eq!(s.total_assets, 3);
        assert_eq!(s.property_count, 1);
        assert_eq!(s.personality_count, 1);
        assert_eq!(s.mixed_count, 1);
        assert_eq!(s.disposal_planned_count, 2, "no_action 不计入已规划");
        assert_eq!(s.total_estimated_value, 30.0);
        assert_eq!(s.completion_percentage, 66, "2/3 应四舍五入为 66");

        let _ = std::fs::remove_file(&path);
    }

    /// 恢复流程的重加密：用旧主密钥解密 → 新主密钥加密，且新密码可正常解锁。
    ///
    /// 这是「忘记主密码后用恢复码重设」的最后一公里：若这里漏掉某张表或某个字段，
    /// 用户重设成功后会发现部分数据永久无法解密。
    #[test]
    fn reset_security_reencrypts_assets_and_profile() {
        use base64::{engine::general_purpose::STANDARD as b64, Engine as _};
        let _g = vault::test_lock();
        vault::clear_key();
        let path = fresh_db_path("reset");
        init_database_at(path.clone()).unwrap();

        // 旧密钥体系
        let salt_old = crate::crypto::generate_salt();
        let key_old = crate::crypto::derive_key_bytes("old-master-111", &salt_old);
        let verifier_old = vault::make_verifier(&key_old).unwrap();
        set_security_config_db(&b64.encode(salt_old), &verifier_old).unwrap();
        vault::set_key(key_old.clone());

        let id = add_asset_db(&AssetRequest {
            platform_id: 1,
            account_identifier: "reset-acc".to_string(),
            account_description: "desc-reset".to_string(),
            category: "property".to_string(),
            estimated_value: Some(77.0),
            disposal_method: "inherit".to_string(),
            heir_name: Some("继承人X".to_string()),
            heir_contact: Some("x@example.com".to_string()),
            notes: "notes-reset".to_string(),
        })
        .unwrap();
        set_asset_credential_db(id, "cred-reset").unwrap();
        save_user_profile_db(&UserProfile {
            full_name: "档案姓名".to_string(),
            phone: "13800000000".to_string(),
            ..Default::default()
        })
        .unwrap();

        // 关键前置：确认换密钥前，档案与资产确实是以旧密钥加密存储的
        {
            let guard = get_conn().unwrap();
            let conn = guard.as_ref().unwrap();
            let raw: String = conn
                .query_row(
                    "SELECT full_name FROM user_profile ORDER BY id DESC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            assert!(vault::is_encrypted(&raw), "前置：档案字段必须是密文");
        }

        // 新密钥体系（salt 与密钥必须配对）
        let salt_new = crate::crypto::generate_salt();
        let key_new = crate::crypto::derive_key_bytes("new-master-222", &salt_new);
        let verifier_new = vault::make_verifier(&key_new).unwrap();
        let salt_v2 = crate::crypto::generate_salt_v2();

        reset_security_config_db(&salt_v2, &verifier_new, "", &key_old, &key_new)
            .expect("重加密必须成功");

        // 切到新密钥后，所有数据必须可读且值不变
        vault::set_key(key_new.clone());
        let asset = get_asset_by_id_db(id).unwrap().unwrap();
        assert_eq!(asset.account_identifier, "reset-acc");
        assert_eq!(asset.account_description, "desc-reset");
        assert_eq!(asset.heir_name.as_deref(), Some("继承人X"));
        assert_eq!(asset.heir_contact.as_deref(), Some("x@example.com"));
        assert_eq!(asset.notes, "notes-reset");
        assert_eq!(get_asset_credential_db(id).unwrap(), "cred-reset");

        // 档案表是独立的一条 UPDATE，漏掉它时资产仍能读，所以必须单独断言
        let profile = get_user_profile_db().unwrap().expect("档案必须存在");
        assert_eq!(profile.full_name, "档案姓名", "档案必须用新密钥重新加密");
        assert_eq!(profile.phone, "13800000000");

        // 旧密钥不再能解开任何数据（证明密钥真的换了）
        vault::set_key(key_old);
        let stale = get_asset_by_id_db(id).unwrap().unwrap();
        assert!(
            stale.account_identifier.contains("加密数据"),
            "旧密钥必须解不开新密文，实际拿到: {}",
            stale.account_identifier
        );
        let stale_profile = get_user_profile_db().unwrap().unwrap();
        assert!(
            stale_profile.full_name.contains("加密数据"),
            "旧密钥必须解不开档案，实际拿到: {}",
            stale_profile.full_name
        );

        vault::clear_key();
        let _ = std::fs::remove_file(&path);
    }
}
