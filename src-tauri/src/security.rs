//! Secret 安全存储（文档 §11）
//!
//! Windows 使用 DPAPI（CryptProtectData / CryptUnprotectData，CURRENT_USER scope）：
//! - 明文不落 SQLite / 项目文件
//! - 密文以 base64 保存于 env_secret_refs 表
//!
//! 非 Windows 平台（开发环境）使用 base64 明文兜底，便于跨平台调试。

use base64::Engine;

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

// 非 Windows 平台兜底（开发调试用，不做真加密）
#[cfg(not(windows))]
fn protect(plain: &[u8]) -> Result<Vec<u8>, String> {
    Ok(plain.to_vec())
}

#[cfg(not(windows))]
fn unprotect(blob: &[u8]) -> Result<Vec<u8>, String> {
    Ok(blob.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip() {
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
        // DPAPI 每次加密产生不同密文（随机 salt）
        let a = encrypt("hello").unwrap();
        let b = encrypt("hello").unwrap();
        assert_ne!(a, b);
        assert_eq!(decrypt(&a).unwrap(), "hello");
        assert_eq!(decrypt(&b).unwrap(), "hello");
    }
}
