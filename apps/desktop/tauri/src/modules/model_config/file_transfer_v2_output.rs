//! Native folder output for a frozen v6 export. Mounted as a child of
//! file_transfer_v2; no v1 package/parser, IPC path authority or source capture.
use super::{
    codec, err, hash, schema_v2, ExportPlanV2, FileTransferOwnerV2, FileTransferReceiptV2,
    FrozenOperation, ModelResult,
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        locking::{lock_path_for, TargetFileLock},
        platform,
    },
    modules::backup::safety,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    io::Write,
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const ALLOCATIONS: &str = "manager-state/file-export-outputs-v2";
const RECORD_LIMIT: u64 = 64 * 1024;
const MANIFEST: &str = "manifest.json";

/// Output-only IPC DTO. A custom Path enters save_export_at exclusively from
/// the host's explicit native folder dialog, never from deserialized UI input.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOutputV2 {
    pub operation_id: String,
    pub receipt: FileTransferReceiptV2,
    pub path: String,
    pub directory: String,
    pub filename: String,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Allocation {
    version: u32,
    id: String,
    root_binding: String,
    file_digest: String,
    length: usize,
    base: PathBuf,
    date: String,
    clock: String,
    path: PathBuf,
    complete: bool,
    receipt: Option<FileTransferReceiptV2>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedAllocation {
    checksum: String,
    authentication: String,
    allocation: Allocation,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OutputManifest<'a> {
    format: &'static str,
    version: u32,
    operation_id: &'a str,
    schema: &'a super::PayloadSchemaRef,
    filename: String,
    digest: &'a str,
    length: usize,
    encrypted: bool,
    counts: Counts,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Counts {
    templates: usize,
    channels: usize,
    accounts: usize,
    skills: usize,
    mcps: usize,
    preferences: bool,
    extensions: bool,
    webdav_candidates: usize,
    channel_keys: usize,
    channel_authorizations: usize,
    webdav_passwords: usize,
    extension_credentials: usize,
    content_versions: usize,
}
fn allocation_path(d: &Path, id: &str) -> ModelResult<PathBuf> {
    if !uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id && u.get_version_num() == 4) {
        return Err(err("file_output_identity"));
    }
    let p = d.join(ALLOCATIONS).join(format!("{id}.json"));
    safety::check_path(d, &p, true)?;
    safety::check_path(d, &lock_path_for(&p), true)?;
    Ok(p)
}
fn stamp(a: &Allocation) -> String {
    format!("opencodex-backup-{}-{}", a.clock, a.id.replace('-', ""))
}
fn calculated_path(a: &Allocation) -> PathBuf {
    let name = stamp(a);
    a.base
        .join(&a.date)
        .join(&name)
        .join(format!("{name}.ocxdconf"))
}
fn utf8(p: &Path) -> ModelResult<String> {
    p.to_str()
        .map(str::to_owned)
        .ok_or_else(|| err("file_output_path_encoding"))
}
fn present(p: &Path) -> ModelResult<bool> {
    match std::fs::symlink_metadata(p) {
        Ok(_) => {
            safety::inspect(p)?;
            Ok(true)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err(err("file_output_io")),
    }
}
fn record_bytes(d: &Path, id: &str) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    let p = allocation_path(d, id)?;
    if !present(&p)? {
        return Ok(None);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(&p)?.permissions().mode() & 0o777 != 0o600 {
            return Err(err("file_output_record_permissions"));
        }
    }
    let file = safety::open_file(&p, false)?;
    safety::require_single_link(&file)?;
    drop(file);
    Ok(Some(Zeroizing::new(safety::read_file(&p, RECORD_LIMIT)?)))
}
fn receipt_matches(r: &FileTransferReceiptV2, plan: &ExportPlanV2) -> bool {
    r.operation_id == plan.id
        && r.plan_digest == plan.digest
        && r.file_digest == plan.digest
        && r.length == plan.length
}
impl FileTransferOwnerV2 {
    /// Read-only allocation lookup. Even an unfinished allocation suppresses a
    /// new folder picker; changed/malformed records are errors, not "missing".
    pub fn has_export_destination(&self, id: &str) -> ModelResult<bool> {
        let Some(a) = self.read_output_allocation(id)? else {
            return Ok(false);
        };
        let (plan, _, _) = self.output_original(id)?;
        self.validate_allocation(&a, &plan)?;
        Ok(true)
    }
    /// Trusted host only. None means default on FIRST allocation; on retry it
    /// means keep the frozen base. Some must be an existing native-picked dir.
    pub fn save_export_at(
        &mut self,
        id: &str,
        custom: Option<&Path>,
    ) -> ModelResult<ExportOutputV2> {
        let state = allocation_path(&self.d, id)?;
        let _guard = TargetFileLock::lock(&state)?;
        self.reopen_export(id)?;
        let (plan, original, _) = self.output_original(id)?;
        let mut allocation = if let Some(a) = self.read_output_allocation(id)? {
            self.validate_allocation(&a, &plan)?;
            if let Some(path) = custom {
                if chosen_base(path)? != a.base {
                    return Err(err("file_output_destination_frozen"));
                }
            }
            a
        } else {
            let base = match custom {
                Some(path) => chosen_base(path)?,
                None => {
                    let p = self.d.join("exports");
                    safety::check_path(&self.d, &p, true)?;
                    platform::create_private_dir_all(&p, 0o700)?;
                    safety::check_path(&self.d, &p, false)?;
                    p.canonicalize()?
                }
            };
            let now = chrono::Local::now();
            let mut a = Allocation {
                version: 2,
                id: id.into(),
                root_binding: hash(&("OCXD_FILE_OUTPUT_ROOT_V2", &self.d, &self.h))?,
                file_digest: plan.digest.clone(),
                length: plan.length,
                base,
                date: now.format("%Y-%m-%d").to_string(),
                clock: now.format("%H%M%S").to_string(),
                path: PathBuf::new(),
                complete: false,
                receipt: None,
            };
            a.path = calculated_path(&a);
            utf8(&a.path)?;
            // Freeze attempted source BEFORE allocating output. A failure to
            // publish the allocation cannot make this operation cancellable.
            self.bytes(id)?;
            self.persist_allocation(&a)?;
            a
        };
        verify_base(&allocation)?;
        let manifest = manifest_bytes(&allocation, &plan)?;
        if allocation.complete {
            return self.verify_output(&allocation, &plan, &original);
        }
        // Even when finish_export already succeeded before an interruption,
        // this false-complete allocation must verify both files before return.
        self.bytes(id)?;
        let directory = allocation
            .path
            .parent()
            .ok_or_else(|| err("file_output_identity"))?;
        safety::check_path(&allocation.base, directory, true)?;
        if present(directory)? {
            verify_entries(directory, &allocation)?;
        }
        check_existing(&allocation.path, &original, false)?;
        check_existing(&directory.join(MANIFEST), &manifest, false)?;
        platform::create_private_dir_all(directory, 0o700)?;
        safety::check_path(&allocation.base, directory, false)?;
        verify_entries(directory, &allocation)?;
        publish_exact(&allocation.base, &allocation.path, &original)?;
        publish_exact(&allocation.base, &directory.join(MANIFEST), &manifest)?;
        // A crash may have left both complete names but not their directory
        // fsync. Existing-file retries must protect that publication too.
        safety::sync_directories(&allocation.base, directory)?;
        // Read actual native output bytes; never pass planned digest as proof.
        let actual = Zeroizing::new(safety::read_file(
            &allocation.path,
            codec::FILE_LIMIT as u64,
        )?);
        if actual.as_slice() != original.as_slice() {
            return Err(err("file_output_readback"));
        }
        check_existing(&directory.join(MANIFEST), &manifest, true)?;
        verify_entries(directory, &allocation)?;
        let receipt = self.finish_export(id, &schema_v2::digest(&actual))?;
        allocation.receipt = Some(receipt);
        allocation.complete = true;
        self.persist_allocation(&allocation)?;
        self.verify_output(&allocation, &plan, &original)
    }
    /// Read-only result. Incomplete allocation returns None; complete output is
    /// checked against the original sealed bytes and original owner receipt.
    pub fn export_output(&self, id: &str) -> ModelResult<Option<ExportOutputV2>> {
        let Some(a) = self.read_output_allocation(id)? else {
            return Ok(None);
        };
        let (plan, bytes, _) = self.output_original(id)?;
        self.validate_allocation(&a, &plan)?;
        if !a.complete {
            return Ok(None);
        }
        self.verify_output(&a, &plan, &bytes).map(Some)
    }
    /// Host-only directory authority for shell_open; IPC accepts operation ID.
    pub fn export_directory(&self, id: &str) -> ModelResult<PathBuf> {
        self.export_output(id)?
            .map(|o| PathBuf::from(o.directory))
            .ok_or_else(|| err("file_output_incomplete"))
    }
    fn output_original(
        &self,
        id: &str,
    ) -> ModelResult<(
        ExportPlanV2,
        Zeroizing<Vec<u8>>,
        Option<FileTransferReceiptV2>,
    )> {
        let (frozen, _) = self.load_recovery(id)?;
        let FrozenOperation::Export {
            plan,
            bytes,
            receipt,
            ..
        } = frozen
        else {
            return Err(err("file_operation_kind_mismatch"));
        };
        if plan.id != id
            || plan.schema != codec::supported_schema()
            || plan.digest != schema_v2::digest(&bytes.0)
            || plan.length != bytes.0.len()
            || receipt.as_ref().is_some_and(|r| !receipt_matches(r, &plan))
        {
            return Err(err("file_recovery_invalid"));
        }
        Ok((plan, bytes.0, receipt))
    }
    fn validate_allocation(&self, a: &Allocation, plan: &ExportPlanV2) -> ModelResult<()> {
        if a.version != 2
            || a.id != plan.id
            || a.root_binding != hash(&("OCXD_FILE_OUTPUT_ROOT_V2", &self.d, &self.h))?
            || a.file_digest != plan.digest
            || a.length != plan.length
            || !a.base.is_absolute()
            || chrono::NaiveDate::parse_from_str(&a.date, "%Y-%m-%d").is_err()
            || a.date.len() != 10
            || chrono::NaiveTime::parse_from_str(&a.clock, "%H%M%S").is_err()
            || a.clock.len() != 6
            || a.path != calculated_path(a)
            || a.complete != a.receipt.is_some()
            || a.receipt
                .as_ref()
                .is_some_and(|r| !receipt_matches(r, plan))
        {
            return Err(err("file_output_identity"));
        }
        utf8(&a.path)?;
        Ok(())
    }
    fn allocation_auth(&self, a: &Allocation) -> ModelResult<Vec<u8>> {
        let key = self.recovery_keys.key(&self.d, &self.h, false)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_ref())
            .map_err(|_| err("file_output_record_invalid"))?;
        mac.update(b"OCXD_FILE_OUTPUT_ALLOCATION_V2\0");
        mac.update(&schema_v2::canonical_bytes(a).map_err(super::model)?);
        Ok(mac.finalize().into_bytes().to_vec())
    }
    fn read_output_allocation(&self, id: &str) -> ModelResult<Option<Allocation>> {
        use base64::Engine;
        let Some(bytes) = record_bytes(&self.d, id)? else {
            return Ok(None);
        };
        let checked: CheckedAllocation =
            serde_json::from_slice(&bytes).map_err(|_| err("file_output_record_invalid"))?;
        // Reject duplicate fields/noncanonical records as well as corruption.
        if serde_json::to_vec(&checked)? != bytes.as_slice()
            || checked.allocation.id != id
            || checked.checksum != hash(&checked.allocation)?
        {
            return Err(err("file_output_record_invalid"));
        }
        let signature = base64::engine::general_purpose::STANDARD
            .decode(&checked.authentication)
            .map_err(|_| err("file_output_record_invalid"))?;
        let key = self.recovery_keys.key(&self.d, &self.h, false)?;
        let mut mac = Hmac::<Sha256>::new_from_slice(key.as_ref())
            .map_err(|_| err("file_output_record_invalid"))?;
        mac.update(b"OCXD_FILE_OUTPUT_ALLOCATION_V2\0");
        mac.update(&schema_v2::canonical_bytes(&checked.allocation).map_err(super::model)?);
        mac.verify_slice(&signature)
            .map_err(|_| err("file_output_record_invalid"))?;
        Ok(Some(checked.allocation))
    }
    fn persist_allocation(&self, a: &Allocation) -> ModelResult<()> {
        use base64::Engine;
        let path = allocation_path(&self.d, &a.id)?;
        let parent = path.parent().ok_or_else(|| err("file_output_identity"))?;
        safety::check_path(&self.d, parent, true)?;
        platform::create_private_dir_all(parent, 0o700)?;
        safety::check_path(&self.d, parent, false)?;
        let bytes = serde_json::to_vec(&CheckedAllocation {
            checksum: hash(a)?,
            authentication: base64::engine::general_purpose::STANDARD
                .encode(self.allocation_auth(a)?),
            allocation: Allocation {
                version: a.version,
                id: a.id.clone(),
                root_binding: a.root_binding.clone(),
                file_digest: a.file_digest.clone(),
                length: a.length,
                base: a.base.clone(),
                date: a.date.clone(),
                clock: a.clock.clone(),
                path: a.path.clone(),
                complete: a.complete,
                receipt: a.receipt.clone(),
            },
        })?;
        if bytes.len() as u64 > RECORD_LIMIT {
            return Err(err("file_output_record_invalid"));
        }
        atomic_write(&path, &bytes, 0o600)?;
        safety::sync_directories(&self.d, parent)?;
        Ok(())
    }
    fn verify_output(
        &self,
        a: &Allocation,
        plan: &ExportPlanV2,
        original: &[u8],
    ) -> ModelResult<ExportOutputV2> {
        verify_base(a)?;
        let directory = a.path.parent().ok_or_else(|| err("file_output_identity"))?;
        safety::check_path(&a.base, &a.path, false)?;
        verify_entries(directory, a)?;
        check_existing(&a.path, original, true)?;
        check_existing(&directory.join(MANIFEST), &manifest_bytes(a, plan)?, true)?;
        let (_, _, receipt) = self.output_original(&a.id)?;
        let receipt = receipt
            .filter(|r| Some(r) == a.receipt.as_ref())
            .ok_or_else(|| err("file_output_receipt_missing"))?;
        Ok(ExportOutputV2 {
            operation_id: a.id.clone(),
            receipt,
            path: utf8(&a.path)?,
            directory: utf8(directory)?,
            filename: a
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or_else(|| err("file_output_path_encoding"))?
                .into(),
        })
    }
}
fn chosen_base(path: &Path) -> ModelResult<PathBuf> {
    if !path.is_absolute() || !safety::inspect(path)?.is_dir() {
        return Err(err("file_output_destination_invalid"));
    }
    let base = path.canonicalize()?;
    safety::check_path(&base, &base, false)?;
    Ok(base)
}
fn verify_base(a: &Allocation) -> ModelResult<()> {
    if chosen_base(&a.base)? != a.base {
        return Err(err("file_output_destination_changed"));
    }
    safety::check_path(&a.base, &a.path, true)?;
    Ok(())
}
fn manifest_bytes(a: &Allocation, p: &ExportPlanV2) -> ModelResult<Vec<u8>> {
    let s = &p.summary;
    Ok(serde_json::to_vec_pretty(&OutputManifest {
        format: "OCXD_CONFIG_EXPORT_OUTPUT_V2",
        version: 2,
        operation_id: &a.id,
        schema: &p.schema,
        filename: a
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| err("file_output_path_encoding"))?
            .into(),
        digest: &p.digest,
        length: p.length,
        encrypted: p.encrypted,
        counts: Counts {
            templates: s.template_ids.len(),
            channels: s.channel_ids.len(),
            accounts: s.account_count,
            skills: s.skill_count,
            mcps: s.mcp_count,
            preferences: s.has_preferences,
            extensions: s.has_extensions,
            webdav_candidates: s.webdav_count,
            channel_keys: s.channel_key_count,
            channel_authorizations: s.channel_authorization_count,
            webdav_passwords: s.webdav_password_count,
            extension_credentials: s.extension_secret_count,
            content_versions: s.content_version_count,
        },
    })?)
}
fn temp_path(path: &Path) -> ModelResult<PathBuf> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| err("file_output_path_encoding"))?;
    Ok(path.with_file_name(format!(".{name}.ocxd-v2-pending")))
}
fn verify_entries(directory: &Path, a: &Allocation) -> ModelResult<()> {
    if !safety::inspect(directory)?.is_dir() {
        return Err(err("file_output_target_changed"));
    }
    let file = &a.path;
    let manifest = directory.join(MANIFEST);
    let allowed = [
        file.clone(),
        manifest.clone(),
        temp_path(file)?,
        temp_path(&manifest)?,
    ];
    for (n, entry) in std::fs::read_dir(directory)?.enumerate() {
        if n >= 4 {
            return Err(err("file_output_target_changed"));
        }
        let p = entry?.path();
        if !allowed.contains(&p) || !safety::inspect(&p)?.is_file() {
            return Err(err("file_output_target_changed"));
        }
    }
    Ok(())
}
fn check_existing(path: &Path, bytes: &[u8], required: bool) -> ModelResult<bool> {
    if !present(path)? {
        if required {
            return Err(err("file_output_target_changed"));
        }
        return Ok(false);
    }
    let actual = Zeroizing::new(safety::read_file(path, bytes.len() as u64)?);
    if actual.as_slice() != bytes {
        return Err(err("file_output_target_changed"));
    }
    Ok(true)
}
/// Exclusive publication: close the private write handle before hard-linking
/// and removing its same-directory temp (Windows sharing semantics). hard_link
/// fails if the destination exists; rename/atomic_write could replace it.
fn publish_exact(anchor: &Path, target: &Path, bytes: &[u8]) -> ModelResult<()> {
    safety::check_path(anchor, target, true)?;
    let temp = temp_path(target)?;
    safety::check_path(anchor, &temp, true)?;
    if check_existing(target, bytes, false)? {
        cleanup_temp(&temp)?;
        return Ok(());
    }
    // Only these exact private temp names belong to the frozen allocation.
    cleanup_temp(&temp)?;
    let written = (|| -> ModelResult<()> {
        let mut file = platform::create_new_private_file(&temp, 0o600)?;
        file.write_all(bytes)?;
        file.flush()?;
        file.sync_all()?;
        drop(file);
        Ok(())
    })();
    if let Err(e) = written {
        let _ = cleanup_temp(&temp);
        return Err(e);
    }
    let result = std::fs::hard_link(&temp, target);
    // All private file handles have closed before cleanup or publication.
    if let Err(e) = result {
        cleanup_temp(&temp)?;
        if e.kind() == std::io::ErrorKind::AlreadyExists {
            check_existing(target, bytes, true)?;
            return Ok(());
        }
        return Err(err("file_output_publish_failed"));
    }
    std::fs::remove_file(&temp)?;
    let parent = target.parent().ok_or_else(|| err("file_output_identity"))?;
    safety::sync_directories(anchor, parent)?;
    check_existing(target, bytes, true)?;
    Ok(())
}
fn cleanup_temp(path: &Path) -> ModelResult<()> {
    if !present(path)? {
        return Ok(());
    }
    let file = safety::open_file(path, false)?;
    // A crash after exclusive hard-link publication can leave two names for
    // the same inode. Removing the TEMP name never modifies its payload.
    drop(file);
    std::fs::remove_file(path)?;
    Ok(())
}
#[cfg(test)]
#[path = "file_transfer_v2_output_tests.rs"]
mod tests;
