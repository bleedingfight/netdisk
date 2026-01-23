//! Token 加密模块
//!
//! 使用 AES-256-GCM 对 access token 进行加密存储，
//! 密钥从环境变量 NETDISK_ENCRYPTION_KEY 获取。
//!
//! # 使用方法
//!
//! ```bash
//! # 设置加密密钥（至少 32 字符）
//! export NETDISK_ENCRYPTION_KEY="your-secret-key-at-least-32-chars"
//! ```

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use std::env;

/// 环境变量名称
pub const ENCRYPTION_KEY_ENV: &str = "NETDISK_ENCRYPTION_KEY";

/// 加密错误类型
#[derive(Debug)]
pub enum CryptoError {
    /// 缺少加密密钥环境变量
    MissingKey,
    /// 密钥长度不足
    KeyTooShort,
    /// 加密失败
    EncryptionFailed(String),
    /// 解密失败
    DecryptionFailed(String),
    /// 数据格式错误
    InvalidFormat(String),
}

impl std::fmt::Display for CryptoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CryptoError::MissingKey => {
                write!(f, "缺少加密密钥，请设置环境变量 {}", ENCRYPTION_KEY_ENV)
            }
            CryptoError::KeyTooShort => write!(f, "加密密钥长度不足，需要至少 16 个字符"),
            CryptoError::EncryptionFailed(msg) => write!(f, "加密失败: {}", msg),
            CryptoError::DecryptionFailed(msg) => write!(f, "解密失败: {}", msg),
            CryptoError::InvalidFormat(msg) => write!(f, "数据格式错误: {}", msg),
        }
    }
}

impl std::error::Error for CryptoError {}

/// Token 加密器
///
/// 使用 AES-256-GCM 加密算法保护敏感数据
pub struct TokenCrypto {
    cipher: Aes256Gcm,
}

impl TokenCrypto {
    /// 从环境变量创建加密器
    ///
    /// 读取 NETDISK_ENCRYPTION_KEY 环境变量作为密钥
    pub fn from_env() -> Result<Self, CryptoError> {
        let key_str = env::var(ENCRYPTION_KEY_ENV).map_err(|_| CryptoError::MissingKey)?;
        Self::new(&key_str)
    }

    /// 使用指定密钥创建加密器
    ///
    /// 密钥会通过 SHA-256 哈希处理，确保长度为 32 字节
    pub fn new(key: &str) -> Result<Self, CryptoError> {
        if key.len() < 16 {
            return Err(CryptoError::KeyTooShort);
        }

        // 使用 SHA-256 将任意长度密钥转换为 32 字节
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key_bytes = hasher.finalize();

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(Self { cipher })
    }

    /// 加密数据
    ///
    /// 返回格式: nonce(12字节) + ciphertext + tag
    /// 编码为 hex 字符串
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        // 使用 AeadCore 的 generate_nonce 生成随机 nonce
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        // 组合 nonce + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        // 返回 hex 编码
        Ok(hex::encode(result))
    }

    /// 解密数据
    ///
    /// 输入格式: hex 编码的 nonce(12字节) + ciphertext + tag
    pub fn decrypt(&self, encrypted_hex: &str) -> Result<String, CryptoError> {
        // 解码 hex
        let data =
            hex::decode(encrypted_hex).map_err(|e| CryptoError::InvalidFormat(e.to_string()))?;

        if data.len() < 12 {
            return Err(CryptoError::InvalidFormat(
                "加密数据太短，无法提取 nonce".to_string(),
            ));
        }

        // 分离 nonce 和 ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // 解密
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

        String::from_utf8(plaintext).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }

    /// 检查加密密钥是否可用
    pub fn is_available() -> bool {
        env::var(ENCRYPTION_KEY_ENV).is_ok()
    }
}

/// 便捷函数：加密 token
///
/// 如果环境变量未设置，返回原始 token（不加密）
pub fn encrypt_token(token: &str) -> String {
    match TokenCrypto::from_env() {
        Ok(crypto) => crypto.encrypt(token).unwrap_or_else(|_| token.to_string()),
        Err(_) => {
            log::warn!("未设置 {} 环境变量，token 将以明文存储", ENCRYPTION_KEY_ENV);
            token.to_string()
        }
    }
}

/// 便捷函数：解密 token
///
/// 如果数据不是加密格式或环境变量未设置，返回原始数据
pub fn decrypt_token(data: &str) -> String {
    // 尝试解密，如果失败则假设是明文
    match TokenCrypto::from_env() {
        Ok(crypto) => crypto.decrypt(data).unwrap_or_else(|_| data.to_string()),
        Err(_) => data.to_string(),
    }
}

/// 检查数据是否已加密
///
/// 加密数据是 hex 编码，且长度至少为 24 (12字节nonce * 2)
pub fn is_encrypted(data: &str) -> bool {
    // 加密数据特征：全是 hex 字符，且长度足够
    data.len() >= 24 && data.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let crypto = TokenCrypto::new("test-key-for-encryption-32chars").unwrap();

        let original = "my_secret_access_token_12345";
        let encrypted = crypto.encrypt(original).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }

    #[test]
    fn test_different_encryptions_produce_different_output() {
        let crypto = TokenCrypto::new("test-key-for-encryption-32chars").unwrap();

        let original = "my_secret_token";
        let encrypted1 = crypto.encrypt(original).unwrap();
        let encrypted2 = crypto.encrypt(original).unwrap();

        // 由于 nonce 是随机的，两次加密结果应该不同
        assert_ne!(encrypted1, encrypted2);

        // 但解密结果应该相同
        assert_eq!(crypto.decrypt(&encrypted1).unwrap(), original);
        assert_eq!(crypto.decrypt(&encrypted2).unwrap(), original);
    }

    #[test]
    fn test_key_too_short() {
        let result = TokenCrypto::new("short");
        assert!(matches!(result, Err(CryptoError::KeyTooShort)));
    }

    #[test]
    fn test_invalid_encrypted_data() {
        let crypto = TokenCrypto::new("test-key-for-encryption-32chars").unwrap();

        // 无效的 hex 数据
        let result = crypto.decrypt("not-valid-hex!");
        assert!(matches!(result, Err(CryptoError::InvalidFormat(_))));

        // 数据太短
        let result = crypto.decrypt("abcd");
        assert!(matches!(result, Err(CryptoError::InvalidFormat(_))));
    }

    #[test]
    fn test_is_encrypted() {
        assert!(is_encrypted("abcdef1234567890abcdef1234567890"));
        assert!(!is_encrypted("short"));
        assert!(!is_encrypted("contains-non-hex-chars!"));
        assert!(!is_encrypted("my_plain_text_token"));
    }

    #[test]
    fn test_unicode_content() {
        let crypto = TokenCrypto::new("test-key-for-encryption-32chars").unwrap();

        let original = "包含中文的token_测试123";
        let encrypted = crypto.encrypt(original).unwrap();
        let decrypted = crypto.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }
}
