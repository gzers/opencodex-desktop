//! Fixed-resource recovery source authority. The signed application's compiled
//! descriptor pin authenticates version/source digest before any tgz is used.
//! This module never installs, extracts executable code, queries or stops services.
use std::path::PathBuf;

use serde::Deserialize;

use super::runtime_recovery_v2::{
    RecoveryContext, RecoverySourceKind, RecoverySourcePin, ResolvedRecoverySource,
    RuntimeRecoveryPrepareRequest, SourceResolver,
};
use crate::{
    errors::{AppError, AppResult},
    modules::{backup::safety, runtime::offline_drop::SharedOfflineDrop},
};

const DESCRIPTOR_NAME: &str = "runtime-bundle.manifest.json";
const ARCHIVE_NAME: &str = "maintained-runtime.tgz";
const MAX_DESCRIPTOR: u64 = 64 * 1024 * 1024;

fn error(code: &str, detail: impl Into<String>) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: detail.into(),
    }
}
fn io(operation: &str, value: AppError) -> AppError {
    match value {
        AppError::FileSystem { detail, .. } => AppError::FileSystem {
            operation: operation.into(),
            detail,
        },
        other => other,
    }
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthenticatedDescriptor {
    schema_version: u32,
    platform: String,
    runtime_version: String,
    runtime_archive_sha256: String,
}
fn current_platform() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "darwin-arm64"
    } else if cfg!(all(windows, target_arch = "x86_64")) {
        "win32-x64"
    } else {
        "unsupported"
    }
}

pub struct FixedResourceSourceResolver {
    resource_directory: PathBuf,
    offline: SharedOfflineDrop,
    descriptor_sha256: Option<String>,
    runtime_source_sha256: Option<String>,
    runtime_source_version: Option<String>,
}
impl FixedResourceSourceResolver {
    /// Paths/pins are startup Rust inputs, not IPC. Missing developer resources
    /// are reported on explicit use, so registration need not disable bundled.
    pub fn new(
        resource_directory: PathBuf,
        offline: SharedOfflineDrop,
        descriptor_sha256: Option<String>,
        runtime_source_sha256: Option<String>,
        runtime_source_version: Option<String>,
    ) -> AppResult<Self> {
        if !resource_directory.is_absolute()
            || resource_directory.components().any(|p| {
                matches!(
                    p,
                    std::path::Component::ParentDir | std::path::Component::CurDir
                )
            })
        {
            return Err(error(
                "recovery_resource_directory_invalid",
                "恢复资源必须来自启动时固定的绝对资源目录",
            ));
        }
        if descriptor_sha256
            .iter()
            .chain(runtime_source_sha256.iter())
            .any(|pin| !valid_digest(pin))
        {
            return Err(error(
                "recovery_resource_pin_invalid",
                "编译固化的恢复资源 pin 无效",
            ));
        }
        if runtime_source_version
            .as_ref()
            .is_some_and(|v| !crate::modules::update::handoff::valid_version(v))
        {
            return Err(error(
                "recovery_resource_pin_invalid",
                "编译固化的恢复源版本无效",
            ));
        }
        Ok(Self {
            resource_directory,
            offline,
            descriptor_sha256,
            runtime_source_sha256,
            runtime_source_version,
        })
    }
    /// build.rs supplies all three pins from release-maintained resources.
    pub fn from_compiled(
        resource_directory: PathBuf,
        offline: SharedOfflineDrop,
    ) -> AppResult<Self> {
        Self::new(
            resource_directory,
            offline,
            option_env!("OCXD_BUNDLED_RUNTIME_DESCRIPTOR_SHA256").map(str::to_owned),
            option_env!("OCXD_BUNDLED_RUNTIME_SOURCE_SHA256").map(str::to_owned),
            option_env!("OCXD_BUNDLED_RUNTIME_SOURCE_VERSION").map(str::to_owned),
        )
    }
    fn authenticated_source(&self) -> AppResult<(PathBuf, RecoverySourcePin)> {
        let expected = self.descriptor_sha256.as_deref().ok_or_else(|| {
            error(
                "recovery_resource_pin_missing",
                "当前构建未带签名应用固化的恢复资源描述 pin；请使用附带维护恢复包的 Desktop 构建",
            )
        })?;
        let descriptor_path = self.resource_directory.join(DESCRIPTOR_NAME);
        safety::check_path(&self.resource_directory, &descriptor_path, false)
            .map_err(|e| io("inspect recovery resource descriptor", e))?;
        let raw = safety::read_file(&descriptor_path, MAX_DESCRIPTOR)
            .map_err(|e| io("read recovery resource descriptor", e))?;
        if crate::infrastructure::hash::sha256_hex(&raw) != expected.to_ascii_lowercase() {
            return Err(error(
                "recovery_descriptor_pin_mismatch",
                "恢复资源描述与应用编译 pin 不一致，未信任相邻资源",
            ));
        }
        // Parse only after authenticating all descriptor bytes. Additional bundle
        // fields remain covered by its digest and are handled by bundled core.
        let descriptor: AuthenticatedDescriptor = serde_json::from_slice(&raw)
            .map_err(|_| error("recovery_descriptor_invalid", "已认证恢复资源描述格式无效"))?;
        if descriptor.schema_version != 1
            || descriptor.platform != current_platform()
            || current_platform() == "unsupported"
        {
            return Err(error(
                "recovery_descriptor_platform_mismatch",
                "恢复描述的 schema 或平台与当前 Desktop 不一致",
            ));
        }
        let source_sha256 = self.runtime_source_sha256.as_deref().ok_or_else(|| {
            error(
                "recovery_source_pin_missing",
                "当前构建未固化维护恢复包摘要",
            )
        })?;
        let source_version = self.runtime_source_version.as_deref().ok_or_else(|| {
            error(
                "recovery_source_pin_missing",
                "当前构建未固化维护恢复包版本",
            )
        })?;
        if !source_sha256.eq_ignore_ascii_case(&descriptor.runtime_archive_sha256) {
            return Err(error(
                "recovery_source_pin_mismatch",
                "恢复源摘要与独立编译 pin 不一致",
            ));
        }
        if source_version != descriptor.runtime_version {
            return Err(error(
                "recovery_source_version_pin_mismatch",
                "恢复源版本与独立编译 pin 不一致",
            ));
        }
        let pin =
            RecoverySourcePin::from_trusted_metadata(source_version.into(), source_sha256.into())?;
        let path = self.resource_directory.join(ARCHIVE_NAME);
        Ok((path, pin))
    }
}
impl SourceResolver for FixedResourceSourceResolver {
    fn resolve(
        &self,
        request: &RuntimeRecoveryPrepareRequest,
        _: &RecoveryContext,
    ) -> AppResult<ResolvedRecoverySource> {
        request.validate()?;
        let result = (|| {
            let (path, pin) = self.authenticated_source()?;
            match request.source {
                RecoverySourceKind::Bundled => {
                    safety::check_path(&self.resource_directory, &path, false)
                        .map_err(|e| io("inspect bundled recovery archive", e))?;
                    ResolvedRecoverySource::copy_pinned(&path, &pin)
                }
                RecoverySourceKind::Offline => ResolvedRecoverySource::from_staged_offline(
                    &self.offline,
                    request
                        .staged_archive_id
                        .as_deref()
                        .ok_or_else(|| error("bad_recovery_source", "缺少 stagedArchiveId"))?,
                    &pin,
                ),
            }
        })();
        if result.is_err() {
            if let (RecoverySourceKind::Offline, Some(id)) =
                (request.source, request.staged_archive_id.as_deref())
            {
                if let Ok(mut receiver) = self.offline.0.lock() {
                    receiver.clear(id);
                }
            }
        }
        result
    }
}
