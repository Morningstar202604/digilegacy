//! 恢复密钥：一次性随机码，加密保存主密钥本体
//!
//! 机制：设置主密码时生成 24 位恢复码（如 XXXX-XXXX-XXXX-XXXX-XXXX-XXXX），
//! 用"恢复码 → 派生密钥"加密"主密钥原文"后入库。
//! 忘记主密码时：输入恢复码 → 解出主密钥 → 用主密钥解开全部数据 → 用新密码重设。
//! 重设主密码时轮换恢复码，旧的立即失效。

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use zeroize::Zeroizing;

use crate::crypto;

const SPLIT_DASH_EVERY: usize = 4;
/// v2 封印前缀：恢复码派生使用 PBKDF2 600k；无前缀 = v1 旧封印（100k，兼容读取）
const SEAL_PREFIX_V2: &str = "RS2.";

/// 生成 24 字符恢复码（16 字节随机 → base32 风格字母表），4 位一组用 - 连接
pub fn generate_recovery_code() -> String {
    const ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // 去除易混 I/O/0/1（含 L，共 32 字符满足 5 位索引）
    let mut bytes = [0u8; 15];
    rand::thread_rng().fill_bytes(&mut bytes);
    let mut out = String::with_capacity(29);
    let mut bits: u32 = 0;
    let mut acc: u32 = 0;
    for &b in bytes.iter() {
        acc = (acc << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((acc >> bits) & 0x1f) as usize;
            out.push(ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((acc << (5 - bits)) & 0x1f) as usize;
        out.push(ALPHABET[idx] as char);
    }
    // 24 个字符，4 个一组
    let mut grouped = String::new();
    for (i, ch) in out.chars().take(24).enumerate() {
        if i > 0 && i % SPLIT_DASH_EVERY == 0 {
            grouped.push('-');
        }
        grouped.push(ch);
    }
    grouped
}

/// 规范化用户输入的恢复码（去空格/连字符，转大写）
pub fn normalize_recovery_code(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

/// 设置流程：以恢复码派生密钥（v2 迭代数）封装主密钥，返回 (恢复码, 带版本前缀的密文)
pub fn create_recovery_seal(master_key: &[u8; 32]) -> Result<(String, String), String> {
    let code = generate_recovery_code();
    let salt = crypto::generate_salt();
    let rec_key = crypto::derive_key_bytes_v(
        &normalize_recovery_code(&code),
        &salt,
        crypto::PBKDF2_ITERATIONS_V2,
    );
    let sealed = crypto::encrypt_with_key(&BASE64.encode(master_key), &rec_key)?;
    // 密文格式: RS2. + base64(salt(16) || sealed)
    let mut combined = salt.to_vec();
    combined.extend_from_slice(
        &BASE64
            .decode(&sealed)
            .map_err(|e| format!("封装修理失败: {}", e))?,
    );
    Ok((
        code,
        format!("{}{}", SEAL_PREFIX_V2, BASE64.encode(&combined)),
    ))
}

/// 恢复流程：以恢复码解开主密钥；按封印前缀选择迭代数（v1 旧封印继续可解）
pub fn unseal_with_recovery(code: &str, sealed_b64: &str) -> Result<Zeroizing<[u8; 32]>, String> {
    let (iterations, body) = if let Some(rest) = sealed_b64.strip_prefix(SEAL_PREFIX_V2) {
        (crypto::PBKDF2_ITERATIONS_V2, rest)
    } else {
        (crypto::PBKDF2_ITERATIONS_V1, sealed_b64)
    };
    let combined = BASE64
        .decode(body)
        .map_err(|_| "恢复密钥数据损坏".to_string())?;
    if combined.len() < 16 {
        return Err("恢复密钥数据长度不合法".to_string());
    }
    let salt = &combined[0..16];
    let sealed = BASE64.encode(&combined[16..]);
    let rec_key = crypto::derive_key_bytes_v(&normalize_recovery_code(code), salt, iterations);
    let master_b64 = crypto::decrypt_with_key(&sealed, &rec_key)?;
    let master = BASE64
        .decode(master_b64)
        .map_err(|_| "主密钥数据损坏".to_string())?;
    if master.len() != 32 {
        return Err("主密钥长度不合法".to_string());
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&master);
    Ok(Zeroizing::new(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_code_shape() {
        let code = generate_recovery_code();
        assert_eq!(code.len(), 29, "24 字符 + 5 个连字符");
        assert_eq!(code.matches('-').count(), 5);
        let bad: Vec<char> = code
            .chars()
            .filter(|c| ['I', 'O', '0', '1'].contains(c))
            .collect();
        assert!(bad.is_empty(), "出现易混字符: {:?} in {}", bad, code);
    }

    #[test]
    fn seal_unseal_roundtrip() {
        let master = [7u8; 32];
        let (code, sealed) = create_recovery_seal(&master).unwrap();
        let unsealed = unseal_with_recovery(&code, &sealed).unwrap();
        assert_eq!(*unsealed, master);
    }

    #[test]
    fn wrong_code_rejected() {
        let master = [7u8; 32];
        let (_code, sealed) = create_recovery_seal(&master).unwrap();
        assert!(unseal_with_recovery("AAAA-BBBB-CCCC-DDDD-EEEE-FFFF", &sealed).is_err());
    }

    #[test]
    fn normalization_strips_dashes_and_case() {
        assert_eq!(normalize_recovery_code("abcd-efgh"), "ABCDEFGH");
        assert_eq!(normalize_recovery_code(" ab cd "), "ABCD");
    }

    #[test]
    fn codes_are_unique() {
        let a = generate_recovery_code();
        let b = generate_recovery_code();
        assert_ne!(a, b);
    }
}
