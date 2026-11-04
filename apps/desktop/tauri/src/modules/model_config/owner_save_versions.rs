//! Write-ahead content receipts for owners with an existing transaction lock.
//! Lock order: owner -> receipt -> versions. Replay only reads bounded bytes;
//! no closure here may acquire an owner/model lock or manufacture a clock.
use super::super::{schema::*, store::sha256};
use super::{record_at_validated, SaveState};
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[derive(Clone, Copy)]
pub enum Owner {
    Preferences,
    Templates,
    Extensions,
}
impl Owner {
    fn name(self) -> &'static str {
        match self {
            Self::Preferences => "preferences",
            Self::Templates => "templates",
            Self::Extensions => "extensions",
        }
    }
    fn file(self) -> &'static str {
        match self {
            Self::Preferences => "manager-state/preferences.json",
            Self::Templates => super::super::schema::LIBRARY_PATH,
            Self::Extensions => crate::modules::extensions::projection::CONFIG_RELATIVE_PATH,
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Receipt {
    #[serde(default)]
    id: String,
    // Semantic recovery is permitted only after the original full byte image
    // was read back. A merely staged intent cannot certify a later rewrite.
    #[serde(default)]
    readback_verified: bool,
    before: SaveState,
    after: SaveState,
    before_file: Option<String>,
    after_file: String,
    saved_at: String,
}
fn path(root: &Path, owner: Owner) -> ModelResult<PathBuf> {
    let path = root
        .join("manager-state/content-save-events")
        .join(format!("{}.json", owner.name()));
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    Ok(path)
}
fn digest(root: &Path, owner: Owner) -> ModelResult<Option<String>> {
    let path = root.join(owner.file());
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(None);
    }
    Ok(Some(sha256(&safety::read_file(&path, 16 * 1024 * 1024)?)))
}
pub struct Prepared {
    root: PathBuf,
    owner: Owner,
    path: PathBuf,
    _guard: TargetFileLock,
}
pub fn begin(
    root: &Path,
    owner: Owner,
    before: &SaveState,
    after: &SaveState,
    bytes: &[u8],
) -> ModelResult<Option<Prepared>> {
    recover(root, owner)?;
    if before == after {
        return Ok(None);
    }
    let path = path(root, owner)?;
    let guard = TargetFileLock::lock(&path)?;
    let receipt = Receipt {
        id: uuid::Uuid::new_v4().to_string(),
        readback_verified: false,
        before: before.clone(),
        after: after.clone(),
        before_file: digest(root, owner)?,
        after_file: sha256(bytes),
        saved_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
    };
    atomic_write(&path, &serde_json::to_vec(&receipt)?, 0o600)?;
    // Keep the receipt guard through the business write. A concurrent reader
    // cannot discard a staged intent while the owner is still writing it.
    Ok(Some(Prepared {
        root: root.into(),
        owner,
        path,
        _guard: guard,
    }))
}
pub fn commit(prepared: Option<Prepared>) -> ModelResult<()> {
    if let Some(prepared) = prepared {
        replay(&prepared.root, prepared.owner, &prepared.path)?;
    }
    Ok(())
}
/// Called after the owner's exact readback, or at its next read/save entry.
/// A pre-write failure/compensation or later external content discards the
/// obsolete intent, while exact applied bytes replay their original receipt.
pub fn recover(root: &Path, owner: Owner) -> ModelResult<()> {
    let path = path(root, owner)?;
    if !path.exists() {
        return Ok(());
    }
    let _guard = TargetFileLock::lock(&path)?;
    if !path.exists() {
        return Ok(());
    }
    replay(root, owner, &path)
}
fn matches_receipt(root: &Path, owner: Owner, receipt: &Receipt) -> ModelResult<bool> {
    if digest(root, owner)?.as_deref() == Some(receipt.after_file.as_str()) {
        return Ok(true);
    }
    if !matches!(owner, Owner::Preferences) || !receipt.readback_verified {
        return Ok(false);
    }
    // Agent settings and portable preferences share a file but have different
    // ownership. Parse bounded bytes only; never call PreferencesStore or take
    // an owner lock from the version-lock CAS.
    let target = root.join(owner.file());
    safety::check_path(root, &target, true)?;
    if !target.exists() {
        return Ok(false);
    }
    let document: serde_json::Value =
        serde_json::from_slice(&safety::read_file(&target, 16 * 1024 * 1024)?)?;
    let preferences =
        crate::modules::preferences::preferences_from_document(&document).map_err(|_| {
            invalid(
                "content_version_preferences",
                "偏好保存回执无法核验当前内容",
            )
        })?;
    Ok(super::super::file_objects::preferences_save_state(Some(&preferences))? == receipt.after)
}
fn replay(root: &Path, owner: Owner, path: &Path) -> ModelResult<()> {
    let mut receipt: Receipt = serde_json::from_slice(&safety::read_file(path, 8 * 1024 * 1024)?)?;
    if !matches_receipt(root, owner, &receipt)? {
        std::fs::remove_file(path)?;
        return Ok(());
    }
    let mut updated = false;
    if receipt.id.is_empty() {
        // Migrate a pre-event-ID pending receipt once under its existing lock;
        // preserve its original time and frozen digests.
        receipt.id = uuid::Uuid::new_v4().to_string();
        updated = true;
    }
    require_uuid(&receipt.id)?;
    if !receipt.readback_verified {
        // matches_receipt above could only accept the exact original byte image.
        receipt.readback_verified = true;
        updated = true;
    }
    if updated {
        atomic_write(path, &serde_json::to_vec(&receipt)?, 0o600)?;
    }
    record_at_validated(
        root,
        &receipt.before,
        &receipt.after,
        &receipt.saved_at,
        &receipt.id,
        || {
            if !matches_receipt(root, owner, &receipt)? {
                return Err(invalid(
                    "content_version_changed",
                    "原保存内容在版本提交前改变",
                ));
            }
            Ok(())
        },
    )?;
    if !matches_receipt(root, owner, &receipt)? {
        return Err(invalid(
            "content_version_changed",
            "原保存内容在回执清理前改变",
        ));
    }
    std::fs::remove_file(path)?;
    Ok(())
}
