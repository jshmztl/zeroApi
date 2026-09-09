//! Secret 安全存储（文档 §11）
//!
//! Windows 使用 DPAPI（CryptProtectData / CryptUnprotectData，CURRENT_USER scope）：
//! - 明文不落 SQLite / 项目文件
//! - 密文以 base64 保存于 env_secret_refs 表
//!
//! 非 Windows 平台使用 XChaCha20-Poly1305 对称加密，密钥存储于本地文件。

use rand::Rng;
use base64::Engine;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static KEY_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 初始化加密模块（在应用启动时调用）
pub fn init(key_dir: &Path) {
    let _ = KEY_DIR.set(key_dir.to_path_buf());
}

/// 加密并返回 base64 字符串
pub fn encrypt(plain: &str) -> Result<String, String> {
    let blob = protect(plain.as_bytes())?;
    Ok(base64::engine::general_purpose::STANDARD.encode(blob))
}

/// 解密 base64 字符串
pub fn decrypt(encoded: &str) -> Result<String, String> {
    let blob = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| format!("base64 解码失败: {}", e))?;
    let plain = unprotect(&blob)?;
    String::from_utf8(plain).map_err(|e| format!("UTF-8 解码失败: {}", e))
}

#[cfg(windows)]
fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: plain.len() as u32,
        pbData: plain.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptProtectData(
            &mut in_blob,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
    };
    if ok == 0 {
        return Err("CryptProtectData 失败（DPAPI 不可用？）".to_string());
    }
    let data = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) }.to_vec();
    unsafe { LocalFree(out_blob.pbData as _) };
    Ok(data)
}

#[cfg(windows)]
fn unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptUnprotectData, CRYPTPROTECT_UI_FORBIDDEN, CRYPT_INTEGER_BLOB,
    };

    let mut in_blob = CRYPT_INTEGER_BLOB {
        cbData: blob.len() as u32,
        pbData: blob.as_ptr() as *mut u8,
    };
    let mut out_blob = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        CryptUnprotectData(
            &mut in_blob,
            std::ptr::null_mut(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            CRYPTPROTECT_UI_FORBIDDEN,
            &mut out_blob,
        )
    };
    if ok == 0 {
        return Err("CryptUnprotectData 失败（密文无效或非本机用户加密）".to_string());
    }
    let data = unsafe { std::slice::from_raw_parts(out_blob.pbData, out_blob.cbData as usize) }.to_vec();
    unsafe { LocalFree(out_blob.pbData as _) };
    Ok(data)
}

// 非 Windows 平台：XChaCha20-Poly1305 对称加密
#[cfg(not(windows))]
fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit, OsRng},
        AeadCore, XChaCha20Poly1305,
    };

    let key = get_or_create_key()?;
    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| format!("初始化加密器失败: {}", e))?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plain)
        .map_err(|e| format!("加密失败: {}", e))?;

    // 格式: nonce (24 bytes) || ciphertext
    let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
    result.extend_from_slice(&nonce);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

#[cfg(not(windows))]
fn unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    use chacha20poly1305::{
        aead::{Aead, KeyInit},
        XChaCha20Poly1305,
    };

    const NONCE_LEN: usize = 24;
    if blob.len() < NONCE_LEN {
        return Err("密文太短，缺少 nonce".to_string());
    }
    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let nonce = chacha20poly1305::XNonce::from_slice(nonce_bytes);

    let key = get_or_create_key()?;
    let cipher = XChaCha20Poly1305::new_from_slice(&key)
        .map_err(|e| format!("初始化解密器失败: {}", e))?;
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("解密失败（密钥不匹配或密文损坏）: {}", e))
}

#[cfg(not(windows))]
fn get_or_create_key() -> Result<[u8; 32], String> {
    let key_file = KEY_DIR
        .get()
        .ok_or("加密模块未初始化（缺少 key_dir），请确保启动时调用 security::init()")?
        .join(".zeroapi_key");

    if key_file.exists() {
        let b64 = std::fs::read_to_string(&key_file)
            .map_err(|e| format!("读取密钥文件失败: {}", e))?;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| format!("密钥文件格式错误: {}", e))?;
        if bytes.len() != 32 {
            return Err(format!("密钥长度错误: 期望 32，实际 {}", bytes.len()));
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(key)
    } else {
        let mut key = [0u8; 32];
        rand::thread_rng().fill(&mut key[..]);
        let b64 = base64::engine::general_purpose::STANDARD.encode(key);
        std::fs::write(&key_file, b64)
            .map_err(|e| format!("写入密钥文件失败: {}", e))?;
        // 限制文件权限（仅所有者可读）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = std::fs::metadata(&key_file)
                .map_err(|e| format!("获取密钥文件元数据失败: {}", e))?
                .permissions();
            perm.set_mode(0o600);
            std::fs::set_permissions(&key_file, perm)
                .map_err(|e| format!("设置密钥文件权限失败: {}", e))?;
        }
        Ok(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn ensure_init() {
        INIT.call_once(|| {
            let dir = std::env::temp_dir().join(format!("zeroapi-security-test-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&dir);
            init(&dir);
        });
    }

    #[test]
    fn test_roundtrip() {
        ensure_init();
        let secret = "sk-test-1234567890-abcdef";
        let encrypted = encrypt(secret).unwrap();
        assert_ne!(encrypted, secret);
        // 密文不应包含明文
        assert!(!encrypted.contains(secret));
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_different_encryptions() {
        ensure_init();
        // XChaCha20-Poly1305 每次使用随机 nonce，密文不同
        let a = encrypt("hello").unwrap();
        let b = encrypt("hello").unwrap();
        assert_ne!(a, b);
        assert_eq!(decrypt(&a).unwrap(), "hello");
        assert_eq!(decrypt(&b).unwrap(), "hello");
    }

    #[test]
    fn test_non_windows_not_plaintext() {
        ensure_init();
        let secret = "my-secret";
        let encrypted = encrypt(secret).unwrap();
        // base64 解码后的原始字节不应包含明文
        let raw = base64::engine::general_purpose::STANDARD.decode(&encrypted).unwrap();
        assert!(!raw.windows(secret.len()).any(|w| w == secret.as_bytes()));
    }
}
