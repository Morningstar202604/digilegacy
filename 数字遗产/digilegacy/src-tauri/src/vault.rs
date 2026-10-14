//! 主密码与字段级加密保管
//!
//! 主密码经 PBKDF2 派生 32 字节密钥驻留内存（锁定即清除）；
//! 用户敏感字段以 `enc:v1:` + base64(nonce||ciphertext) 形式写入数据库。

use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::crypto;

pub const ENC_PREFIX: &str = "enc:v1:";
const VERIFIER_PLAINTEXT: &str = "digilegacy-verifier-v1";

static MASTER_KEY: Lazy<Mutex<Option<[u8; 32]>>> = Lazy::new(|| Mutex::new(None));

/// 测试串行锁：跨模块共享，避免并行测试互相污染全局状态
#[cfg(test)]
pub(crate) static TEST_SERIAL: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[cfg(test)]
pub(crate) fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

/// 解锁后驻留内存的主密钥
pub fn set_key(k: [u8; 32]) {
    let mut guard = MASTER_KEY.lock().unwrap();
    *guard = Some(k);
}

/// 锁定：先清零再清除内存中的主密钥
pub fn clear_key() {
    let mut guard = MASTER_KEY.lock().unwrap();
    if let Some(k) = guard.as_mut() {
        for b in k.iter_mut() {
            *b = 0;
        }
    }
    *guard = None;
}

pub fn get_key() -> Option<[u8; 32]> {
    *MASTER_KEY.lock().unwrap()
}

pub fn is_unlocked() -> bool {
    MASTER_KEY.lock().unwrap().is_some()
}

pub fn is_encrypted(s: &str) -> bool {
    s.starts_with(ENC_PREFIX)
}

/// 用指定密钥加密字段，输出带 `enc:v1:` 前缀的密文
pub fn encrypt_field(plain: &str, key: &[u8; 32]) -> Result<String, String> {
    Ok(format!("{}{}", ENC_PREFIX, crypto::encrypt_with_key(plain, key)?))
}

/// 解密带 `enc:v1:` 前缀的字段
pub fn decrypt_field(enc: &str, key: &[u8; 32]) -> Result<String, String> {
    let body = enc.strip_prefix(ENC_PREFIX).ok_or("非加密字段")?;
    crypto::decrypt_with_key(body, key)
}

/// 写入前加密：空值/已加密/未解锁时原样返回
pub fn encrypt_if_needed(plain: &str) -> Result<String, String> {
    if plain.is_empty() || is_encrypted(plain) {
        return Ok(plain.to_string());
    }
    match get_key() {
        Some(k) => encrypt_field(plain, &k),
        None => Ok(plain.to_string()),
    }
}

/// 读取时解密：空值/明文原样返回；密文在未解锁时报错
pub fn decrypt_if_needed(value: &str) -> Result<String, String> {
    if value.is_empty() || !is_encrypted(value) {
        return Ok(value.to_string());
    }
    let key = get_key().ok_or("数据已加密，请先解锁")?;
    decrypt_field(value, &key)
}

/// 读取容错版：解密失败返回占位符，避免单行异常导致整表加载失败
pub fn decrypt_or_placeholder(value: &str) -> String {
    if value.is_empty() || !is_encrypted(value) {
        return value.to_string();
    }
    match decrypt_if_needed(value) {
        Ok(v) => v,
        Err(_) => "【加密数据 · 需解锁】".to_string(),
    }
}

/// 生成主密码验证器（用于校验解锁口令是否正确）
pub fn make_verifier(key: &[u8; 32]) -> Result<String, String> {
    encrypt_field(VERIFIER_PLAINTEXT, key)
}

/// 重加密单值：密文用旧钥解开后以新钥加密；历史明文直接以新钥加密；空值原样
pub fn rekey_value(raw: &str, old_key: &[u8; 32], new_key: &[u8; 32]) -> Result<String, String> {
    if raw.is_empty() {
        return Ok(String::new());
    }
    let plain = if is_encrypted(raw) {
        decrypt_field(raw, old_key)?
    } else {
        raw.to_string()
    };
    encrypt_field(&plain, new_key)
}

pub fn verify_key(verifier: &str, key: &[u8; 32]) -> bool {
    decrypt_field(verifier, key)
        .map(|s| s == VERIFIER_PLAINTEXT)
        .unwrap_or(false)
}

/// 校验外部链接：仅允许 http/https 且包含主机名
pub fn validate_external_url(url: &str) -> Result<String, String> {
    let u = url.trim();
    if u.is_empty() {
        return Err("链接为空".to_string());
    }
    let lower = u.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err("仅允许打开 http/https 链接".to_string());
    }
    let rest = &u[u.find("://").map(|i| i + 3).unwrap_or(u.len())..];
    if rest.is_empty() || rest.starts_with('/') {
        return Err("链接缺少主机名".to_string());
    }
    Ok(u.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unlocked_key() -> [u8; 32] {
        crypto::derive_key_bytes("master-pw-123", &[9u8; 16])
    }

    #[test]
    fn test_field_encrypt_prefix_and_roundtrip() {
        let _g = test_lock();
        set_key(unlocked_key());
        let enc = encrypt_if_needed("my-account@example.com").unwrap();
        assert!(is_encrypted(&enc), "加密结果应带 enc:v1: 前缀");
        assert!(enc.starts_with(ENC_PREFIX));
        assert_eq!(decrypt_if_needed(&enc).unwrap(), "my-account@example.com");
        clear_key();
    }

    #[test]
    fn test_plaintext_passthrough() {
        let _g = test_lock();
        set_key(unlocked_key());
        assert_eq!(decrypt_if_needed("旧的明文数据").unwrap(), "旧的明文数据");
        assert_eq!(decrypt_if_needed("").unwrap(), "");
        clear_key();
    }

    #[test]
    fn test_lock_clears_key_and_blocks_decrypt() {
        let _g = test_lock();
        set_key(unlocked_key());
        let enc = encrypt_if_needed("secret").unwrap();
        clear_key();
        assert!(!is_unlocked());
        assert!(get_key().is_none());
        assert!(decrypt_if_needed(&enc).is_err(), "锁定状态不得解密返回明文");
    }

    #[test]
    fn test_no_key_encrypt_passthrough() {
        let _g = test_lock();
        clear_key();
        assert_eq!(encrypt_if_needed("未解锁状态").unwrap(), "未解锁状态");
        assert_eq!(encrypt_if_needed("").unwrap(), "");
    }

    #[test]
    fn test_verifier_accepts_correct_rejects_wrong() {
        let _g = test_lock();
        let k1 = unlocked_key();
        let v = make_verifier(&k1).unwrap();
        assert!(verify_key(&v, &k1));
        let k2 = crypto::derive_key_bytes("wrong-password", &[9u8; 16]);
        assert!(!verify_key(&v, &k2));
        clear_key();
    }

    #[test]
    fn test_url_whitelist() {
        assert!(validate_external_url("https://www.bilibili.com").is_ok());
        assert!(validate_external_url("http://example.com/path").is_ok());
        assert!(validate_external_url("file:///etc/passwd").is_err());
        assert!(validate_external_url("javascript:alert(1)").is_err());
        assert!(validate_external_url("ftp://example.com").is_err());
        assert!(validate_external_url("").is_err());
        assert!(validate_external_url("https://").is_err());
    }

    #[test]
    fn test_rekey_value() {
        let _g = test_lock();
        let old = crypto::derive_key_bytes("old-pw", &[1u8; 16]);
        let new = crypto::derive_key_bytes("new-pw", &[2u8; 16]);

        let enc_old = encrypt_field("敏感内容", &old).unwrap();
        let enc_new = rekey_value(&enc_old, &old, &new).unwrap();
        assert!(is_encrypted(&enc_new));
        assert_eq!(decrypt_field(&enc_new, &new).unwrap(), "敏感内容");
        assert!(decrypt_field(&enc_new, &old).is_err(), "旧密钥不得解开新密文");

        let upgraded = rekey_value("历史明文", &old, &new).unwrap();
        assert!(is_encrypted(&upgraded), "明文在重加密时应被顺带加密");
        assert_eq!(decrypt_field(&upgraded, &new).unwrap(), "历史明文");

        assert_eq!(rekey_value("", &old, &new).unwrap(), "");
    }
}
