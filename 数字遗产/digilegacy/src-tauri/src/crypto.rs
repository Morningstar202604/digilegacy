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

const PBKDF2_ITERATIONS: u32 = 100_000;

/// 生成随机12字节nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// 从密码派生密钥（PBKDF2-SHA256，10万次迭代）
fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    *Key::<Aes256Gcm>::from_slice(&derive_key_bytes(password, salt))
}

/// 从密码与盐派生 32 字节原始密钥（密钥式接口与验证器复用）
pub fn derive_key_bytes(password: &str, salt: &[u8]) -> [u8; 32] {
    let mut key_bytes = [0u8; 32];
    pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key_bytes);
    key_bytes
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

/// AES-256-GCM 加密
pub fn encrypt(plaintext: &str, password: &str) -> Result<String, String> {
    let salt = {
        let mut s = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut s);
        s
    };
    
    let key = derive_key(password, &salt);
    let cipher = Aes256Gcm::new(&key);
    
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("加密失败: {:?}", e))?;
    
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    
    Ok(BASE64.encode(&result))
}

/// AES-256-GCM 解密
pub fn decrypt(encrypted: &str, password: &str) -> Result<String, String> {
    let data = BASE64.decode(encrypted).map_err(|_| "数据格式无效".to_string())?;
    
    if data.len() < 28 {
        return Err("数据长度不合法".to_string());
    }
    
    let salt = &data[0..16];
    let nonce_bytes = &data[16..28];
    let ciphertext = &data[28..];
    
    let key = derive_key(password, salt);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "解密失败，请检查密码".to_string())?;
    
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
    fn test_encrypt_decrypt() {
        let password = "test_password_123";
        let message = "这是一段测试用的敏感数字遗产数据";
        
        let encrypted = encrypt(message, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();
        
        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let encrypted = encrypt("test", "correct_password").unwrap();
        assert!(decrypt(&encrypted, "wrong_password").is_err());
    }

    #[test]
    fn test_deterministic_key_derivation() {
        let password = "my_secure_password";
        let salt = [1u8; 16];
        let key1 = derive_key(password, &salt);
        let key2 = derive_key(password, &salt);
        assert_eq!(key1.as_slice(), key2.as_slice(), "相同密码和盐应产生相同密钥");
    }

    #[test]
    fn test_different_salts_produce_different_keys() {
        let password = "my_secure_password";
        let salt1 = [1u8; 16];
        let salt2 = [2u8; 16];
        let key1 = derive_key(password, &salt1);
        let key2 = derive_key(password, &salt2);
        assert_ne!(key1.as_slice(), key2.as_slice(), "不同盐应产生不同密钥");
    }

    #[test]
    fn test_deterministic_encryption_uses_random_salt() {
        let password = "test_password";
        let message = "same message";
        let encrypted1 = encrypt(message, password).unwrap();
        let encrypted2 = encrypt(message, password).unwrap();
        assert_ne!(encrypted1, encrypted2, "相同明文两次加密应产生不同密文（随机盐）");
    }

    #[test]
    fn test_unicode_password() {
        let password = "密码测试123!@#";
        let message = "敏感数据";
        let encrypted = encrypt(message, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();
        assert_eq!(message, decrypted);
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