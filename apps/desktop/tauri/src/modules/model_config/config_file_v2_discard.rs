//! Explicit discard of one exact server draft. Metadata is invalidated before
//! bounded cleanup; no actual config/library/Policy/version or rematch write.
use super::*;
const DISCARDS: &str = "manager-state/model-draft-discards-v2";
pub fn discard(root: &Path, home: &Path, expected: ReadBaseV2) -> Result<FileSnapshotV2> {
    discard_with_store(root, home, expected, &SystemDraftCredentialsV2)
}
fn discard_with_store(
    root: &Path,
    home: &Path,
    expected: ReadBaseV2,
    vault: &dyn DraftCredentialStoreV2,
) -> Result<FileSnapshotV2> {
    history::with_state(root, home, Some(&expected), |locked, state, raw| {
        let draft = state
            .draft
            .as_ref()
            .ok_or_else(|| error("model_discard_stale"))?;
        // Construct actual saved facts BEFORE invalidation, without allocating
        // provider identities or using draft policy as source authority.
        let bindings = draft
            .observed_baseline
            .as_ref()
            .ok_or_else(|| error("model_discard_stale"))?
            .channels
            .iter()
            .map(|c| ProviderBindingV2 {
                channel_id: c.id.clone(),
                provider_id: c.connection.provider_id.clone(),
            })
            .collect::<Vec<_>>();
        let observed = provider_import::read_policy_v2(
            &raw,
            &expected.source_digest,
            &state.policy,
            state.ledger.as_ref(),
            &state.mappings,
            &bindings,
        )?;
        let result = FileSnapshotV2 {
            dto_version: 2,
            library: state.library.clone(),
            library_digest: state.digests.library.clone(),
            saved_policy: observed.clone(),
            draft: None,
            source_digest: expected.source_digest.clone(),
            root_revision: expected.root_revision.clone(),
            effective: effective(&state.library, &observed)?,
        };
        let draft_path = root.join(store_v2::DRAFT_PATH);
        safety::check_path(root, &draft_path, false).map_err(io)?;
        let bytes = safety::read_file(&draft_path, 32 * 1024 * 1024).map_err(io)?;
        if digest(&bytes) != state.digests.files[store_v2::DRAFT_PATH]
            || locked.snapshot_unlocked()?.digests != state.digests
            || source_digest(&read_raw(home)?.1) != expected.source_digest
        {
            return Err(error("model_discard_stale"));
        }
        let directory = root.join(DISCARDS);
        safety::check_path(root, &directory, true).map_err(io)?;
        std::fs::create_dir_all(&directory).map_err(io)?;
        crate::infrastructure::platform::set_mode(&directory, 0o700).map_err(io)?;
        let tombstone = directory.join(format!("{}.json", uuid::Uuid::new_v4()));
        safety::check_path(root, &tombstone, true).map_err(io)?;
        // Same-root rename atomically makes this generation unavailable. The
        // exact private original survives interruption or a failed cleanup.
        if let Err(error) = invalidate(&draft_path, &tombstone) {
            if !draft_path.exists() && tombstone.exists() {
                let _ = std::fs::rename(&tombstone, &draft_path);
            }
            return Err(error);
        }
        let current = (|| -> Result<bool> {
            crate::infrastructure::platform::set_mode(&tombstone, 0o600).map_err(io)?;
            let saved = locked.snapshot_saved_unlocked()?;
            Ok(source_digest(&read_raw(home)?.1) == expected.source_digest
                && saved.digests.library == state.digests.library
                && saved
                    .digests
                    .files
                    .iter()
                    .filter(|(p, _)| p.as_str() != store_v2::DRAFT_PATH)
                    .all(|(p, v)| state.digests.files.get(p) == Some(v)))
        })();
        if !matches!(current, Ok(true)) {
            if !draft_path.exists() {
                std::fs::rename(&tombstone, &draft_path).map_err(io)?;
            }
            return Err(error("model_discard_stale"));
        }
        // Refuse to clean anything if an independent writer ignored the Store
        // lock and installed another generation during invalidation.
        if std::fs::symlink_metadata(&draft_path).is_ok() {
            return Err(error("model_discard_generation_changed"));
        }
        if cleanup_exact(root, draft, &state.policy, &raw, vault) {
            let _ = std::fs::remove_file(&tombstone);
        }
        Ok(result)
    })
}
// Do not delete private vault material before invalidation is durable. A
// failed flush retains/rolls back the exact draft and never enters cleanup.
fn invalidate(source: &Path, target: &Path) -> Result<()> {
    if std::fs::symlink_metadata(target).is_ok() {
        return Err(error("model_discard_collision"));
    }
    #[cfg(unix)]
    {
        std::fs::rename(source, target).map_err(io)?;
        for directory in [source.parent(), target.parent()].into_iter().flatten() {
            std::fs::File::open(directory)
                .and_then(|f| f.sync_all())
                .map_err(io)?;
        }
        Ok(())
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{MoveFileExW, MOVEFILE_WRITE_THROUGH};
        let source = source
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        let target = target
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();
        // SAFETY: fixed owned paths; both UTF-16 buffers are NUL terminated.
        if unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), MOVEFILE_WRITE_THROUGH) } == 0 {
            return Err(error("model_storage"));
        }
        Ok(())
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = (source, target);
        Err(error("model_discard_platform_unsupported"))
    }
}
fn mentions(value: &Value, reference: &str) -> bool {
    match value {
        Value::String(s) => s.contains(reference),
        Value::Array(a) => a.iter().any(|v| mentions(v, reference)),
        Value::Object(o) => o.values().any(|v| mentions(v, reference)),
        _ => false,
    }
}
fn cleanup_exact(
    root: &Path,
    draft: &DraftV2,
    saved: &PolicyV2,
    raw: &Value,
    vault: &dyn DraftCredentialStoreV2,
) -> bool {
    let mut complete = true;
    let references = draft
        .credential_edits
        .values()
        .filter_map(|e| match e {
            DraftCredentialEditV2::Replace { credential_ref } => Some(credential_ref),
            DraftCredentialEditV2::Clear => None,
        })
        .collect::<BTreeSet<_>>();
    for reference in references {
        if saved
            .channels
            .iter()
            .any(|c| c.connection.credential_ref.as_ref() == Some(reference))
            || mentions(raw, reference)
        {
            continue;
        }
        // Only the captured draft's typed Replace authority supplies this UUID.
        // Never enumerate a vault or derive actual-provider service accounts.
        if check_id(reference).is_err() || vault.remove(reference).is_err() {
            complete = false;
        }
    }
    for channel in &draft.policy.channels {
        let Some(reference) = &channel.connection.imported_config else {
            continue;
        };
        if saved.channels.iter().any(|c| {
            c.connection
                .imported_config
                .as_ref()
                .is_some_and(|r| r.id == reference.id)
        }) || mentions(raw, &reference.id)
        {
            continue;
        }
        let Ok(path) = copy_path(root, &reference.id) else {
            complete = false;
            continue;
        };
        match std::fs::symlink_metadata(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => {
                complete = false;
                continue;
            }
            Ok(_) => {}
        }
        let record = (|| -> Result<CopySourceV2> {
            let bytes = safety::read_file(&path, 65536).map_err(io)?;
            if digest(&bytes) != reference.digest {
                return Err(error("model_discard_copy_invalid"));
            }
            let record: CopySourceV2 =
                serde_json::from_value(parse_strict_json_v2(&bytes)?).map_err(json_error)?;
            if record.schema_version != 2
                || record.channel_id != channel.id
                || record.source_digest != draft.base.source_digest
            {
                return Err(error("model_discard_copy_invalid"));
            }
            Ok(record)
        })();
        if record.is_err() || std::fs::remove_file(&path).is_err() {
            complete = false;
        }
    }
    complete
}
#[cfg(test)]
#[path = "config_file_v2_discard_tests.rs"]
mod tests;
