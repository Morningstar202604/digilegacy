#![allow(dead_code)]
//! AES-256-GCM 加密基础设施
//! 
//! 当前 MVP 阶段数据库采用本地文件级安全控制（仅当前用户可读），
//! 后续版本将升级为字段级 AES-256-GCM 加密，此时直接启用本模块。

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;

/// 生成随机12字节nonce
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce
}

/// 从密码派生密钥
fn derive_key(password: &str, salt: &[u8]) -> Key<Aes256Gcm> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut key_material = Vec::new();
    key_material.extend_from_slice(password.as_bytes());
    key_material.extend_from_slice(salt);
    
    let mut hasher = DefaultHasher::new();
    key_material.hash(&mut hasher);
    let hash1 = hasher.finish();
    
    key_material.push(1u8);
    let mut hasher2 = DefaultHasher::new();
    key_material.hash(&mut hasher2);
    let hash2 = hasher2.finish();
    
    let mut key_bytes = Vec::with_capacity(32);
    key_bytes.extend_from_slice(&hash1.to_le_bytes());
    key_bytes.extend_from_slice(&hash2.to_le_bytes());
    key_bytes.extend_from_slice(&hash1.to_be_bytes());
    key_bytes.extend_from_slice(&hash2.to_be_bytes());
    key_bytes.truncate(32);
    
    *Key::<Aes256Gcm>::from_slice(&key_bytes)
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
}
