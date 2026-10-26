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

        -- SMTP 配置：此前只能通过环境变量设置，普通用户根本配不起来，
        -- 结果「安心传承」的通知永远只落本地发件箱，家人一封邮件也收不到。
        -- 密码走 vault 字段级加密（与凭据同一条路径），不明文落库。
        -- 保管位置：备份文件放在哪、恢复码抄在哪。
        -- 这两个信息决定了「家人能不能真正拿到凭据」，此前完全没有字段可填。
        -- 属于敏感元数据（泄露即等于指出攻击目标），因此同样走字段级加密。
        CREATE TABLE IF NOT EXISTS custody_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            backup_location_enc TEXT DEFAULT '',
            recovery_code_location_enc TEXT DEFAULT '',
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS smtp_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            host TEXT DEFAULT '',
            port INTEGER DEFAULT 587,
            from_addr TEXT DEFAULT '',
            username TEXT DEFAULT '',
            password_enc TEXT DEFAULT '',
            allow_insecure INTEGER DEFAULT 0,
            updated_at TEXT NOT NULL
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
pub(super) fn sync_platforms(conn: &Connection) -> Result<(), String> {
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

    platforms::apply_platform_overrides(conn, &mut platforms)?;

    Ok(platforms)
}

/// 报告用：把资产实际引用到的平台政策收成一份索引。
///
/// 只收 `assets` 用到的平台，避免报告里塞进 152 条无关政策。
/// 扩展字段（变现路径/风险提示/标签）来自内置表；政策补丁覆盖过的平台
/// 以 `platforms` 表为准（补丁镜像后的结果），保证导出与界面读同一份数据。
pub fn get_policy_briefs_db(platform_ids: &[i64]) -> Result<Vec<PlatformPolicyBrief>, String> {
    use std::collections::HashSet;
    let wanted: HashSet<i64> = platform_ids.iter().copied().collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    let platforms = get_all_platforms_db()?;
    let meta_by_id: std::collections::HashMap<i64, crate::platform_data::PlatformEntry> =
        crate::platform_data::get_all_platforms()
            .into_iter()
            .map(|m| (m.id, m))
            .collect();
    let mut out: Vec<PlatformPolicyBrief> = platforms
        .into_iter()
        .filter(|p| wanted.contains(&p.id))
        .map(|p| {
            let meta = meta_by_id.get(&p.id);
            PlatformPolicyBrief {
                platform_id: p.id,
                name: p.name,
                required_documents: p.required_documents,
                process_description: p.process_description,
                estimated_difficulty: p.estimated_difficulty,
                estimated_time_days: p.estimated_time_days,
                official_policy_url: p.official_policy_url,
                monetization_path: meta
                    .map(|m| m.monetization_path.clone())
                    .unwrap_or_default(),
                risk_warnings: meta.map(|m| m.risk_warnings.clone()).unwrap_or_default(),
                tags: meta.map(|m| m.tags.clone()).unwrap_or_default(),
                has_source: meta
                    .map(|m| !m.data_source.trim().is_empty() && !m.data_source.contains("未核实"))
                    .unwrap_or(false),
                last_reviewed: meta.map(|m| m.last_reviewed.clone()).unwrap_or_default(),
                data_source: meta.map(|m| m.data_source.clone()).unwrap_or_default(),
            }
        })
        .collect();
    out.sort_by_key(|p| p.platform_id);
    Ok(out)
}

// ========== 数字资产 CRUD ==========

/// 写敏感字段前的前置校验：已设置主密码但处于锁定状态时，禁止写入。
/// 此时 vault::encrypt_if_needed 会静默明文直通，用户以为已加密实则明文落库——
/// 这是历史缺陷，必须在此处拦截。尚未启用加密（无 security_config）时放行：
/// 存量明文模式由 encrypt_existing_data_db 在启用加密后统一收编。
/// 供其他 db 子模块复用的写入门禁（锁定态一律拒绝）。
pub(crate) fn ensure_vault_writable_pub(conn: &Connection) -> Result<(), String> {
    ensure_vault_writable(conn)
}

fn ensure_vault_writable(conn: &Connection) -> Result<(), String> {
    let configured: i64 = conn
        .query_row("SELECT COUNT(*) FROM security_config WHERE id=1", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);
    if configured == 0 {
        // 尚未启用主密码：明文模式，放行
        return Ok(());
    }
    if !crate::vault::is_unlocked() {
        return Err("主密码已设置但处于锁定状态，请先解锁后再保存".to_string());
    }
    // 内存里有密钥 ≠ 密钥正确。恢复流程用旧封印解出的密钥可能与当前验证器不匹配，
    // 那种状态下读取全是占位符、写入会把占位符当明文写回（数据丢失）。
    // 因此写路径必须验证密钥与库里验证器一致，而不只看「有没有 key」。
    if !vault_key_matches_config(conn) {
        return Err("当前会话密钥与主密码不匹配，请重新锁定后再解锁".to_string());
    }
    Ok(())
}

/// 内存中的密钥是否能通过库内验证器校验。
pub fn vault_key_matches_config(conn: &Connection) -> bool {
    let Ok(Some((_salt, verifier))) = read_security_config(conn) else {
        return false;
    };
    match crate::vault::get_key() {
        Some(key) => crate::vault::verify_key(&verifier, &key),
        None => false,
    }
}

/// 读取安全配置（不依赖全局连接，供内部守卫复用）。
fn read_security_config(conn: &Connection) -> Result<Option<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT kdf_salt, verifier FROM security_config WHERE id = 1")
        .map_err(|e| format!("查询安全配置失败: {}", e))?;
    let mut rows = stmt.query([]).map_err(|e| format!("查询执行失败: {}", e))?;
    if let Some(row) = rows.next().map_err(|e| format!("读取失败: {}", e))? {
        Ok(Some((
            row.get(0).map_err(|e| format!("{}", e))?,
            row.get(1).map_err(|e| format!("{}", e))?,
        )))
    } else {
        Ok(None)
    }
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
    validate_disposal_heir(&req.disposal_method, req.heir_name.as_deref())?;
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
    validate_disposal_heir(&req.disposal_method, req.heir_name.as_deref())?;
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

/// SMTP 配置（不含密码明文）。
///
/// `has_password` 只表示库里有密文，不返回明文——界面只需要知道「已保存」。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: i64,
    pub from_addr: String,
    pub username: String,
    pub has_password: bool,
    pub allow_insecure: bool,
}

pub fn get_smtp_config_db() -> Result<SmtpConfig, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let row = conn.query_row(
        "SELECT COALESCE(host,''), port, COALESCE(from_addr,''), COALESCE(username,''), COALESCE(password_enc,''), allow_insecure FROM smtp_config WHERE id=1",
        [],
        |r| {
            Ok(SmtpConfig {
                host: r.get::<_, String>(0)?,
                port: r.get::<_, i64>(1)?,
                from_addr: r.get::<_, String>(2)?,
                username: r.get::<_, String>(3)?,
                has_password: !r.get::<_, String>(4)?.is_empty(),
                allow_insecure: r.get::<_, i64>(5)? != 0,
            })
        },
    );
    match row {
        Ok(v) => Ok(v),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(SmtpConfig {
            port: 587,
            ..Default::default()
        }),
        Err(e) => Err(format!("读取 SMTP 配置失败: {}", e)),
    }
}

/// 保存 SMTP 配置。`password` 为 `None` 表示「保持现有密码不变」——
/// 界面只回显 `has_password`，用户不重填时不能把已保存的密码清空。
pub fn set_smtp_config_db(
    host: &str,
    port: i64,
    from_addr: &str,
    username: &str,
    password: Option<&str>,
    allow_insecure: bool,
) -> Result<(), String> {
    if !(1..=65535).contains(&port) {
        return Err("SMTP 端口必须在 1 到 65535 之间".to_string());
    }
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    ensure_vault_writable(conn)?;
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let password_enc = match password {
        Some(p) if !p.is_empty() => crate::vault::encrypt_if_needed(p)?,
        Some(_) => String::new(),
        None => conn
            .query_row(
                "SELECT COALESCE(password_enc,'') FROM smtp_config WHERE id=1",
                [],
                |r| r.get::<_, String>(0),
            )
            .unwrap_or_default(),
    };
    conn.execute(
        "INSERT INTO smtp_config (id, host, port, from_addr, username, password_enc, allow_insecure, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(id) DO UPDATE SET host=?1, port=?2, from_addr=?3, username=?4, password_enc=?5, allow_insecure=?6, updated_at=?7",
        params![host, port, from_addr, username, password_enc, allow_insecure as i64, now],
    )
    .map_err(|e| format!("保存 SMTP 配置失败: {}", e))?;
    Ok(())
}

/// 取出解密后的 SMTP 明文密码（仅供投递使用，绝不回传前端）。
pub fn get_smtp_password_plain() -> Result<Option<String>, String> {
    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();
    let enc: String = conn
        .query_row(
            "SELECT COALESCE(password_enc,'') FROM smtp_config WHERE id=1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_default();
    if enc.is_empty() {
        return Ok(None);
    }
    if crate::vault::is_encrypted(&enc) && !crate::vault::is_unlocked() {
        return Err("SMTP 密码已加密，请先解锁后重试".to_string());
    }
    crate::vault::decrypt_if_needed(&enc).map(Some)
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

/// 备份导入模式。
///
/// 覆盖是历史行为，会先清空资产/档案/签到记录；导入前没有快照时，
/// 一次误操作就永久丢数据。默认改为合并，覆盖必须显式选择。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportMode {
    /// 保留现有数据，只追加备份里的资产（id 冲突时分配新 id）。
    Merge,
    /// 清空资产/档案/签到后写入备份内容（原行为）。
    Replace,
}

impl ImportMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "replace" => Self::Replace,
            _ => Self::Merge,
        }
    }
}

/// 导入备份：覆盖模式（历史行为，保留给既有调用方与回归测试）。
///
/// 新代码应使用 `import_backup_json_db_with_mode`，由命令层按用户选择决定模式。
/// 生产路径已全部切到带模式的版本，这里只被测试与场景矩阵调用。
#[allow(dead_code)]
pub fn import_backup_json_db(content: &str) -> Result<i64, String> {
    import_backup_json_db_with_mode(content, ImportMode::Replace)
}

/// 导入备份：按 `mode` 合并或覆盖用户数据与安全配置；导入后强制重新解锁。
///
/// 调用方（命令层）负责在覆盖模式下先写一份当前数据快照。
pub fn import_backup_json_db_with_mode(content: &str, mode: ImportMode) -> Result<i64, String> {
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
    // 缺失 = 旧格式，允许为空；**存在但不是数组 = 损坏，必须拒绝**。
    // 旧写法用 `and_then(as_array).unwrap_or_default()`，一个手改过的
    // `"user_profile": "x"` 会被静默当成空数组，导入"成功"却把档案清空了。
    let array_or_missing = |key: &str| -> Result<Vec<Value>, String> {
        match tables.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(items)) => Ok(items.clone()),
            Some(_) => Err(format!("备份字段 {} 必须是数组", key)),
        }
    };
    let profiles = array_or_missing("user_profile")?;
    let check_ins = array_or_missing("check_in_log")?;

    let guard = get_conn()?;
    let conn = guard.as_ref().unwrap();

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("开启事务失败: {}", e))?;
    let result = (|| -> Result<i64, String> {
        if mode == ImportMode::Replace {
            conn.execute("DELETE FROM digital_assets", [])
                .map_err(|e| format!("清空资产失败: {}", e))?;
            conn.execute("DELETE FROM user_profile", [])
                .map_err(|e| format!("清空档案失败: {}", e))?;
            conn.execute("DELETE FROM check_in_log", [])
                .map_err(|e| format!("清空记录失败: {}", e))?;
        }

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
            // 合并模式不能复用备份里的 id：本地可能已有同 id 资产，
            // 直接 INSERT 会撞主键并把整个导入回滚。让 SQLite 分配新 id。
            let backup_id = ival(a, "id");
            let target_id = if mode == ImportMode::Replace {
                backup_id
            } else {
                let taken: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM digital_assets WHERE id=?1",
                        params![backup_id],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if taken == 0 && backup_id > 0 {
                    backup_id
                } else {
                    0
                }
            };
            conn.execute(
                "INSERT INTO digital_assets (id, platform_id, account_identifier, account_description, category, estimated_value, value_currency, disposal_method, heir_name, heir_contact, notes, credential_enc, is_confirmed, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    if target_id > 0 { Some(target_id) } else { None },
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

        // 合并模式不清空档案/签到：只补「本地还没有」的档案行，
        // 已有档案保持现状（否则一次合并导入会把当前联系人信息换掉）。
        let profile_id = |p: &Value| ival(p, "id");
        for p in &profiles {
            let pid = profile_id(p);
            if mode == ImportMode::Merge {
                let taken: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM user_profile WHERE id=?1",
                        params![pid],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if taken > 0 {
                    continue;
                }
            }
            conn.execute(
                "INSERT INTO user_profile (id, full_name, phone, email, id_last4, emergency_contact_name, emergency_contact_phone, emergency_contact_relation, emergency_contact_email, heir_primary_name, heir_primary_phone, heir_primary_relation, heir_primary_email, check_in_interval_days, last_check_in, dms_enabled, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
                params![
                    pid,
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

        // 合并模式只追加备份里比本地更新的确认记录（id 冲突跳过），
        // 覆盖模式沿用原行为整表写入。
        for c in &check_ins {
            let cid = ival(c, "id");
            if mode == ImportMode::Merge {
                let taken: i64 = conn
                    .query_row(
                        "SELECT COUNT(*) FROM check_in_log WHERE id=?1",
                        params![cid],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if taken > 0 {
                    continue;
                }
            }
            conn.execute(
                "INSERT INTO check_in_log (id, checked_at, status, note) VALUES (?1, ?2, ?3, ?4)",
                params![
                    cid,
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

/// SMTP 是否已配置到「可以真正发送」的程度（host 与 from 都非空）。
/// 直接查库：调用方可能已持有连接锁，不能再走 `get_conn()`。
fn smtp_configured_in(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT COALESCE(host,''), COALESCE(from_addr,'') FROM smtp_config WHERE id=1",
        [],
        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
    )
    .map(|(host, from)| !host.trim().is_empty() && !from.trim().is_empty())
    .unwrap_or(false)
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
    // 注意：这里不再统计 `is_confirmed`。该列在插入/更新时恒写 1，
    // 从未有真实的「未确认」状态，`unconfirmed_count` 永远为 0——
    // 一个永远显示「全部已确认」的指标比没有指标更糟，故整列从统计中移除
    // （列本身保留，避免破坏既有备份与迁移路径）。
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
    // 继承人：只有「选继承却没填名字」才是不合格，其他处置方式不涉及继承人。
    let heir_assigned: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets
             WHERE disposal_method != 'inherit'
                OR (heir_name IS NOT NULL AND TRIM(heir_name) != '')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    // 账号标识：占位符表示未解密，不算已填——否则锁定态下指标会虚高。
    let identifier_ready: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM digital_assets
             WHERE account_identifier IS NOT NULL
               AND TRIM(account_identifier) != ''
               AND account_identifier NOT LIKE '%需解锁%'",
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

    // 触发链可用性：只认真正能送达的通道，不看「开关有没有打开」。
    // 旧界面把「DMS 已启用」等同于「家人会收到通知」，实际没配 SMTP 时
    // 通知只进本地发件箱，家人什么都收不到。
    //
    // 注意：这里必须直接查库，不能调用 `notify::resolve_smtp_settings()`——
    // 那个函数内部会 `get_conn()`，而本函数已持有连接锁，二次加锁会死锁
    // （std Mutex 不可重入，实测整个测试套挂死）。
    let smtp_ready = smtp_configured_in(conn);
    let heartbeat_ready = conn
        .query_row(
            "SELECT enabled, COALESCE(endpoint,'') FROM heartbeat_config WHERE id=1",
            [],
            |r| Ok((r.get::<_, i64>(0)? != 0, r.get::<_, String>(1)?)),
        )
        .map(|(enabled, endpoint)| enabled && !endpoint.trim().is_empty())
        .unwrap_or(false);
    let trigger_channel = if smtp_ready {
        "smtp".to_string()
    } else if heartbeat_ready {
        "heartbeat".to_string()
    } else {
        "none".to_string()
    };
    let trigger_ready = dms_enabled && trigger_channel != "none";

    // 四项各 25%。任何一项为 0，整体就不会虚高到「完成」。
    let ratio = |n: i64| -> i64 {
        if total > 0 {
            n * 100 / total
        } else {
            0
        }
    };
    let trigger_score = if trigger_ready { 100 } else { 0 };
    let completion = if total > 0 {
        (ratio(disposal_planned) + ratio(heir_assigned) + ratio(identifier_ready) + trigger_score)
            / 4
    } else {
        0
    };

    Ok(DashboardStats {
        total_assets: total,
        property_count,
        personality_count,
        mixed_count,
        disposal_planned_count: disposal_planned,
        heir_assigned_count: heir_assigned,
        identifier_ready_count: identifier_ready,
        trigger_ready,
        trigger_channel,
        total_estimated_value: total_value,
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

    let platform_ids: Vec<i64> = assets.iter().map(|a| a.platform_id).collect();
    let policies = get_policy_briefs_db(&platform_ids)?;

    Ok(InheritanceReport {
        user,
        assets,
        policies,
        total_estimated_value: total_value,
        property_assets_count: property_count,
        personality_assets_count: personality_count,
        mixed_assets_count: mixed_count,
        generated_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

#[cfg(test)]
#[path = "db_tests.rs"]
mod tests;

#[path = "db_platforms.rs"]
mod platforms;
pub use platforms::*;

#[path = "db_custody.rs"]
mod custody;
pub use custody::*;
