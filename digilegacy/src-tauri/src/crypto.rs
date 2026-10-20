#![allow(dead_code)]
/// AES-256-GCM 加密基础设施
///
/// 提供两套接口：
/// 1. 密码式 encrypt/decrypt（每次调用派生密钥，适合低频、单值场景）；
/// 2. 密钥式 encrypt_with_key/decrypt_with_key（密钥派生一次、批量复用，
///    供 vault 模块做主密码字段级加密，避免逐字段 PBKDF2 开销）。

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use zeroize::Zeroizing;

/// v1 迭代数（历史格式兼容读取；新写入一律 v2）
pub const PBKDF2_ITERATIONS_V1: u32 = 100_000;
/// v2 迭代数（OWASP 2023 对 PBKDF2-HMAC-SHA256 的建议：600k）
pub const PBKDF2_ITERATIONS_V2: u32 = 600_000;
/// v2 盐的自描述前缀（'.' 不在 base64 字母表内，与 v1 裸 base64 盐无碰撞可能）
pub const SALT_PREFIX_V2: &str = "v2.";

/// 生成随机12字节nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// v1 兼容入口：按旧迭代数派生（历史调用点/测试使用）
pub fn derive_key_bytes(password: &str, salt: &[u8]) -> Zeroizing<[u8; 32]> {
    derive_key_bytes_v(password, salt, PBKDF2_ITERATIONS_V1)
}

/// 按显式迭代数派生 32 字节密钥；Zeroizing 包装保证离开作用域即清零
pub fn derive_key_bytes_v(password: &str, salt: &[u8], iterations: u32) -> Zeroizing<[u8; 32]> {
    let mut key_bytes = Zeroizing::new([0u8; 32]);
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, iterations, key_bytes.as_mut());
    key_bytes
}

/// 生成 v2 随机盐（自描述前缀 + base64），存库为 TEXT
pub fn generate_salt_v2() -> String {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    format!("{}{}", SALT_PREFIX_V2, BASE64.encode(salt))
}

/// 按盐前缀解析出真实盐字节与应使用的迭代数（无前缀 = v1 裸 base64 盐）
pub fn resolve_salt(salt_str: &str) -> Result<(Vec<u8>, u32), String> {
    if let Some(rest) = salt_str.strip_prefix(SALT_PREFIX_V2) {
        let bytes = BASE64.decode(rest).map_err(|_| "盐格式无效".to_string())?;
        Ok((bytes, PBKDF2_ITERATIONS_V2))
    } else {
        let bytes = BASE64.decode(salt_str).map_err(|_| "盐格式无效".to_string())?;
        Ok((bytes, PBKDF2_ITERATIONS_V1))
    }
}

/// 用已派生密钥加密，输出 base64(nonce(12) || ciphertext)
pub fn encrypt_with_key(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("加密失败: {:?}", e))?;

    let mut out = Vec::with_capacity(12 + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    Ok(BASE64.encode(&out))
}

/// 用已派生密钥解密 encrypt_with_key 的输出
pub fn decrypt_with_key(encrypted: &str, key: &[u8; 32]) -> Result<String, String> {
    let data = BASE64.decode(encrypted).map_err(|_| "数据格式无效".to_string())?;

    if data.len() < 13 {
        return Err("数据长度不合法".to_string());
    }

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&data[0..12]);

    let plaintext = cipher
        .decrypt(nonce, &data[12..])
        .map_err(|_| "解密失败，请检查主密码".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "解密后数据编码错误".to_string())
}

/// 生成随机盐
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_prefix_selects_iterations() {
        // v1 裸盐与 v2 前缀盐分别解析出不同迭代数
        let raw = [1u8; 16];
        let (bytes1, iters1) = resolve_salt(&BASE64.encode(raw)).unwrap();
        assert_eq!(iters1, PBKDF2_ITERATIONS_V1);
        assert_eq!(bytes1, raw.to_vec());

        let salt_v2 = generate_salt_v2();
        assert!(salt_v2.starts_with(SALT_PREFIX_V2));
        let (bytes2, iters2) = resolve_salt(&salt_v2).unwrap();
        assert_eq!(iters2, PBKDF2_ITERATIONS_V2);
        assert_eq!(bytes2.len(), 16);
    }

    #[test]
    fn test_v2_derivation_matches_v1_semantics() {
        // 相同密码与盐字节：显式传 v1 迭代数的结果必须与兼容入口一致
        let a = derive_key_bytes("pw", &[9u8; 16]);
        let b = derive_key_bytes_v("pw", &[9u8; 16], PBKDF2_ITERATIONS_V1);
        assert_eq!(*a, *b);
        // v2 迭代数不同 → 密钥不同
        let c = derive_key_bytes_v("pw", &[9u8; 16], PBKDF2_ITERATIONS_V2);
        assert_ne!(*a, *c);
    }

    #[test]
    fn test_wrong_password_fails() {
        let key = derive_key_bytes("correct_password", &[3u8; 16]);
        let encrypted = encrypt_with_key("test", &key).unwrap();
        let wrong = derive_key_bytes("wrong_password", &[3u8; 16]);
        assert!(decrypt_with_key(&encrypted, &wrong).is_err());
    }

    #[test]
    fn test_deterministic_key_derivation() {
        let password = "my_secure_password";
        let salt = [1u8; 16];
        let key1 = derive_key_bytes(password, &salt);
        let key2 = derive_key_bytes(password, &salt);
        assert_eq!(key1, key2, "相同密码和盐应产生相同密钥");
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "my_secure_password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        let key1 = derive_key_bytes(password, &salt1);
        let key2 = derive_key_bytes(password, &salt2);
        assert_ne!(key1, key2, "不同盐应产生不同密钥");
    }

    #[test]
    fn test_unicode_password() {
        let key = derive_key_bytes("密码测试123!@#", &[5u8; 16]);
        let encrypted = encrypt_with_key("敏感数据", &key).unwrap();
        assert_eq!(decrypt_with_key(&encrypted, &key).unwrap(), "敏感数据");
    }

    #[test]
    fn test_key_based_roundtrip() {
        let key = derive_key_bytes("master-pw", &[7u8; 16]);
        let ct = encrypt_with_key("敏感内容-账号密码", &key).unwrap();
        assert_ne!(ct, "敏感内容-账号密码");
        assert_eq!(decrypt_with_key(&ct, &key).unwrap(), "敏感内容-账号密码");
    }

    #[test]
    fn test_key_based_tamper_fails() {
        let key = derive_key_bytes("master-pw", &[7u8; 16]);
        let ct = encrypt_with_key("data", &key).unwrap();
        let mut bytes = BASE64.decode(&ct).unwrap();
        let n = bytes.len();
        bytes[n - 1] ^= 0x01;
        let tampered = BASE64.encode(&bytes);
        assert!(decrypt_with_key(&tampered, &key).is_err());
    }

    #[test]
    fn test_key_based_empty_roundtrip() {
        let key = derive_key_bytes("master-pw", &[7u8; 16]);
        let ct = encrypt_with_key("", &key).unwrap();
        assert_eq!(decrypt_with_key(&ct, &key).unwrap(), "");
    }
}