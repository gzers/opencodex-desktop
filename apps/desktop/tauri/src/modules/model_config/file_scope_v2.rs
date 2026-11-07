//! Last explicit LocalFile selection, NOT a prepared plan or capture authority.
//! D/H come from the host. No endpoint, capture, secret owner or business write.
//! The fixed file lock serializes reads/saves; last completed explicit save wins.
//! UI response ordering belongs to its read generation, not a source/runtime CAS.
use super::{
    objects_overlay_v2::{SelectionV2, UsageV2},
    saved_objects_capture_v2::{self, CaptureRequestV2},
    schema::{invalid, ModelError, ModelResult},
    schema_v2,
    sync_service_v2::ScopeRequestV2,
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        locking::{lock_path_for, TargetFileLock},
    },
    modules::{
        backup::safety,
        extensions::{discovery, CLIENT_IDS},
        sync::portable_assets,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

pub const RELATIVE_PATH: &str = "manager-state/file-transfer-v2-scope.json";
/// The complete stored JSON must be strictly smaller than 512 KiB.
pub const BYTE_LIMIT: usize = 512 * 1024;
const FORMAT: &str = "OCXD_LOCAL_FILE_SCOPE_V2";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    format: String,
    schema_version: u32,
    root_home_digest: String,
    initialized: bool,
    request: ScopeRequestV2,
}
struct Context {
    root: PathBuf,
    home: PathBuf,
    binding: String,
    target: PathBuf,
}
fn err(code: &str) -> ModelError {
    invalid(
        code,
        "本机文件选区偏好未通过格式、范围或根绑定核对；未采用默认选区",
    )
}
fn canonical<T: Serialize>(value: &T) -> ModelResult<Vec<u8>> {
    schema_v2::canonical_bytes(value).map_err(|_| err("file_scope_format"))
}
impl Context {
    fn new(root: &Path, home: &Path) -> ModelResult<Self> {
        for p in [root, home] {
            if !p.is_absolute()
                || p.components()
                    .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
                || !safety::inspect(p)
                    .map_err(|_| err("file_scope_root"))?
                    .is_dir()
            {
                return Err(err("file_scope_root"));
            }
        }
        let root = root.canonicalize().map_err(|_| err("file_scope_root"))?;
        let home = home.canonicalize().map_err(|_| err("file_scope_root"))?;
        let binding = schema_v2::digest(&canonical(&(FORMAT, 2_u32, &root, &home))?);
        let target = root.join(RELATIVE_PATH);
        let context = Self {
            root,
            home,
            binding,
            target,
        };
        context.check_paths()?;
        Ok(context)
    }
    fn check_paths(&self) -> ModelResult<()> {
        for p in [&self.target, &lock_path_for(&self.target)] {
            safety::check_path(&self.root, p, true).map_err(|_| err("file_scope_path"))?;
            match std::fs::symlink_metadata(p) {
                Ok(meta) => {
                    if !meta.is_file() {
                        return Err(err("file_scope_path"));
                    }
                    if p == &self.target {
                        let file =
                            safety::open_file(p, false).map_err(|_| err("file_scope_path"))?;
                        safety::require_single_link(&file).map_err(|_| err("file_scope_path"))?;
                    }
                    // Do not read a Windows byte-range lock through another
                    // handle. TargetFileLock's Windows private opener checks
                    // type/reparse/link ownership before locking the file.
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::MetadataExt;
                        if meta.nlink() != 1 {
                            return Err(err("file_scope_path"));
                        }
                    }
                    require_private(p)?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err(err("file_scope_io")),
            }
        }
        Ok(())
    }
    fn lock(&self) -> ModelResult<TargetFileLock> {
        self.check_paths()?;
        let guard = TargetFileLock::lock(&self.target).map_err(|_| err("file_scope_lock"))?;
        self.check_paths()?;
        Ok(guard)
    }
    fn recheck_roots(&self) -> ModelResult<()> {
        let current = Self::new(&self.root, &self.home)?;
        if current.binding != self.binding {
            return Err(err("file_scope_binding"));
        }
        Ok(())
    }
    fn load(&self) -> ModelResult<Option<ScopeRequestV2>> {
        self.check_paths()?;
        let metadata = match std::fs::symlink_metadata(&self.target) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err(err("file_scope_io")),
        };
        if metadata.len() >= BYTE_LIMIT as u64 {
            return Err(err("file_scope_limit"));
        }
        let bytes = safety::read_file(&self.target, (BYTE_LIMIT - 1) as u64)
            .map_err(|_| err("file_scope_io"))?;
        let value =
            schema_v2::parse_strict_json_v2(&bytes).map_err(|_| err("file_scope_format"))?;
        // CaptureRequestV2's explicit-selection decoder prevents serde defaults
        // from turning absent nested fields into an apparently explicit choice.
        let explicit: CaptureRequestV2 = serde_json::from_value(
            value
                .get("request")
                .cloned()
                .ok_or_else(|| err("file_scope_format"))?,
        )
        .map_err(|_| err("file_scope_format"))?;
        let record: Record = serde_json::from_value(value).map_err(|_| err("file_scope_format"))?;
        if record.format != FORMAT || record.schema_version != 2 || !record.initialized {
            return Err(err("file_scope_format"));
        }
        if !schema_v2::valid_digest(&record.root_home_digest)
            || record.root_home_digest != self.binding
        {
            return Err(err("file_scope_binding"));
        }
        // Both decoders use the same public shape; never recover an implicit
        // field, migrate old formats, match names to IDs, or inspect live files.
        validate(&ScopeRequestV2 {
            selection: explicit.selection,
            mcp_clients: explicit.mcp_clients,
            assets_reviewed: explicit.assets_reviewed,
        })?;
        Ok(Some(record.request))
    }
}

/// A missing preference is None; damaged, future, unsafe or foreign data is Err.
/// A first missing read does not initialize directories or default selections.
pub fn read(root: &Path, home: &Path) -> ModelResult<Option<ScopeRequestV2>> {
    let context = Context::new(root, home)?;
    let parent = context
        .target
        .parent()
        .ok_or_else(|| err("file_scope_path"))?;
    match std::fs::symlink_metadata(parent) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(err("file_scope_io")),
        Ok(_) => {}
    }
    let _guard = context.lock()?;
    let scope = context.load()?;
    context.recheck_roots()?;
    Ok(scope)
}

/// Persist choices only. Actual source existence/credentials/digests must be
/// checked again by file-transfer prepare; saving this record grants no writes.
pub fn save(root: &Path, home: &Path, request: ScopeRequestV2) -> ModelResult<()> {
    validate(&request)?;
    let context = Context::new(root, home)?;
    let bytes = canonical(&Record {
        format: FORMAT.into(),
        schema_version: 2,
        root_home_digest: context.binding.clone(),
        initialized: true,
        request,
    })?;
    if bytes.len() >= BYTE_LIMIT {
        return Err(err("file_scope_limit"));
    }
    let _guard = context.lock()?;
    // Never replace a corrupt/future/foreign preference with a silent reset.
    context.load()?;
    context.recheck_roots()?;
    atomic_write(&context.target, &bytes, 0o600).map_err(|_| err("file_scope_io"))?;
    safety::sync_directories(
        &context.root,
        context
            .target
            .parent()
            .ok_or_else(|| err("file_scope_path"))?,
    )
    .map_err(|_| err("file_scope_io"))?;
    context.check_paths()?;
    if safety::read_file(&context.target, (BYTE_LIMIT - 1) as u64)
        .map_err(|_| err("file_scope_io"))?
        != bytes
    {
        return Err(err("file_scope_io"));
    }
    context.recheck_roots()
}

fn require_private(path: &Path) -> ModelResult<()> {
    #[cfg(unix)]
    {
        if crate::infrastructure::platform::mode_of(path).map_err(|_| err("file_scope_path"))?
            != 0o600
        {
            return Err(err("file_scope_path"));
        }
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
fn id(value: &str) -> ModelResult<()> {
    schema_v2::check_id(value).map_err(|_| err("file_scope_selection"))
}
fn ids(values: &[String], max: usize) -> ModelResult<()> {
    if values.len() > max || values.iter().collect::<BTreeSet<_>>().len() != values.len() {
        return Err(err("file_scope_selection"));
    }
    values.iter().try_for_each(|v| id(v))
}
fn asset_names(values: &[String], max: usize) -> ModelResult<()> {
    let mut seen = BTreeSet::new();
    if values.len() > max {
        return Err(err("file_scope_selection"));
    }
    for v in values {
        if !discovery::is_safe_skill_name(v)
            || v.len() > 256
            || v.chars().any(|c| c.is_control() || "<>:\"|?*".contains(c))
            || v.ends_with([' ', '.'])
            || !seen.insert(v.to_lowercase())
        {
            return Err(err("file_scope_selection"));
        }
    }
    Ok(())
}
fn endpoint_ids(values: &[String]) -> ModelResult<()> {
    if values.len() > 256
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
        || values.iter().any(|v| {
            v.is_empty()
                || v.len() > 256
                || !v
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
                || v == "."
                || v == ".."
        })
    {
        return Err(err("file_scope_selection"));
    }
    Ok(())
}
fn validate(request: &ScopeRequestV2) -> ModelResult<()> {
    let s: &SelectionV2 = &request.selection;
    if s.usage != UsageV2::LocalFile {
        return Err(err("file_scope_local_required"));
    }
    // This pure validator performs no capture, credential or filesystem reads.
    saved_objects_capture_v2::validate_request(&CaptureRequestV2 {
        selection: s.clone(),
        mcp_clients: request.mcp_clients.clone(),
        assets_reviewed: request.assets_reviewed,
    })
    .map_err(|_| err("file_scope_selection"))?;
    for c in &s.channels {
        if let Some(values) = &c.family_ids {
            ids(values, 4096)?;
        }
        if let Some(values) = &c.model_ids {
            if values
                .iter()
                .any(|v| v.len() > 2048 || v.trim() != v || v.chars().any(char::is_control))
            {
                return Err(err("file_scope_selection"));
            }
        }
    }
    for refs in [&s.template_refs, &s.adopt_template_defaults] {
        for r in refs {
            id(&r.template_id)?;
            id(&r.revision_id)?;
            if !schema_v2::valid_digest(&r.digest) {
                return Err(err("file_scope_selection"));
            }
        }
    }
    ids(&s.manager.account_ids, 1000)?;
    endpoint_ids(&s.manager.webdav_ids)?;
    endpoint_ids(&s.manager.webdav_password_ids)?;
    asset_names(&s.skill_names, 500)?;
    asset_names(
        &s.mcp_sources
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<_>>(),
        1000,
    )?;
    asset_names(&s.extension_credentials.skill_names, 500)?;
    asset_names(&s.extension_credentials.mcp_names, 1000)?;
    for source in request.mcp_clients.values() {
        if source != "managed"
            && !CLIENT_IDS
                .iter()
                .any(|id| portable_assets::client_key(*id) == *source)
        {
            return Err(err("file_scope_selection"));
        }
    }
    Ok(())
}
#[cfg(test)]
#[path = "file_scope_v2_tests.rs"]
mod tests;
