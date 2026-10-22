//! 凭据身份与账户位（跨平台）。
//!
//! 本模块只负责**身份**：service 名、purpose、`account_key` 形状与 `ref_id` 生成。
//! 值的读写已迁到 `infrastructure::credentials`（本机文件后端，见 WP-37）：
//! 发布包是 ad-hoc 签名，系统钥匙串的 ACL 认不出每次构建，会反复弹授权窗口。
//! 剩余的渠道 Key / 扩展密钥 / 授权保护仍走系统钥匙串，各自用途不同，不能混改。
use uuid::Uuid;

use crate::errors::AppError;

pub const KEYCHAIN_SERVICE_NAME: &str = "OpenCodex Desktop";
/// 沙箱（开发/测试）专用的钥匙串服务名：与日常使用身份分离，错误身份不回退到日常服务。
pub const KEYCHAIN_SANDBOX_SERVICE_NAME: &str = "OpenCodex Desktop Sandbox";

/// 运行时钥匙串服务名：经沙箱开关决定，构建期与日常保持兼容。
pub fn keychain_service_name() -> &'static str {
    if crate::modules::test_sandbox::enabled() {
        KEYCHAIN_SANDBOX_SERVICE_NAME
    } else {
        KEYCHAIN_SERVICE_NAME
    }
}
pub const WEBDAV_CREDENTIAL_PURPOSE: &str = "webdav_credential";
pub const ENCRYPTION_PASSWORD_PURPOSE: &str = "encryption_password";

pub fn account_key(purpose: &str, ref_id: &str) -> Result<String, AppError> {
    if !matches!(
        purpose,
        WEBDAV_CREDENTIAL_PURPOSE | ENCRYPTION_PASSWORD_PURPOSE
    ) {
        return Err(AppError::NotConfigured);
    }
    if !ref_id.starts_with("cred_")
        || ref_id.len() != 5 + 36
        || !ref_id[5..]
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        return Err(AppError::NotConfigured);
    }
    let prefix = if purpose == WEBDAV_CREDENTIAL_PURPOSE {
        "ocx.dav"
    } else {
        "ocx.sync"
    };
    Ok(format!("{prefix}.{ref_id}"))
}

pub fn new_ref_id() -> String {
    format!("cred_{}", Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_key_matches_frozen_shape() {
        let ref_id = "cred_12345678-1234-1234-1234-123456789abc";
        assert_eq!(
            account_key(WEBDAV_CREDENTIAL_PURPOSE, ref_id).unwrap(),
            "ocx.dav.cred_12345678-1234-1234-1234-123456789abc"
        );
        assert_eq!(
            account_key(ENCRYPTION_PASSWORD_PURPOSE, ref_id).unwrap(),
            "ocx.sync.cred_12345678-1234-1234-1234-123456789abc"
        );
        assert!(account_key("other", ref_id).is_err());
        assert!(account_key(WEBDAV_CREDENTIAL_PURPOSE, "bad").is_err());
    }

    #[test]
    fn new_ref_id_is_valid() {
        let ref_id = new_ref_id();
        assert!(account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).is_ok());
    }

    #[test]
    fn webdav_and_encryption_purposes_use_different_accounts() {
        let ref_id = new_ref_id();
        assert_ne!(
            account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).unwrap(),
            account_key(ENCRYPTION_PASSWORD_PURPOSE, &ref_id).unwrap(),
            "两个口令必须落在不同账户位，否则后写会覆盖先写"
        );
    }
}
