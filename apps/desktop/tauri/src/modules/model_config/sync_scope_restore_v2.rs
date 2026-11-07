//! Local scope deselection is one exact CAS participant of scope undo. No cloud IO.
use super::*;
use crate::infrastructure::locking::TargetFileLock;
use crate::modules::backup::safety;
const RELATIVE: &str = "manager-state/sync-service-v2/scope.json";

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ScopeEditV2 {
    before: Option<Vec<u8>>,
    after: Vec<u8>,
}
fn scope_error(_: impl std::fmt::Debug) -> SyncFailure {
    err(ErrorCode::SyncLocalStateIo, Step::Persist)
}
pub(crate) fn lock(root: &Path) -> SyncResult<TargetFileLock> {
    let files = ServiceFilesV2::new(root)?;
    let path = files.path("scope.json")?;
    TargetFileLock::lock(&path).map_err(scope_error)
}
pub(crate) fn path(root: &Path) -> PathBuf {
    root.join(RELATIVE)
}
fn raw(root: &Path) -> SyncResult<Option<Vec<u8>>> {
    let path = path(root);
    safety::check_path(root, &path, true).map_err(scope_error)?;
    match std::fs::symlink_metadata(&path) {
        Ok(_) => safety::read_file(&path, 4 * 1024 * 1024)
            .map(Some)
            .map_err(scope_error),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(scope_error(e)),
    }
}
fn record(files: &ServiceFilesV2) -> SyncResult<Option<ScopeRecordV2>> {
    let record: Option<ScopeRecordV2> = files.read("scope.json")?;
    if record
        .as_ref()
        .is_some_and(|r| r.version != 2 || r.root_digest != files.root_digest)
    {
        return Err(err(ErrorCode::SyncJournalInvalid, Step::Persist));
    }
    Ok(record)
}
// Only exact identities are removed. This does not expand any selection.
pub(crate) fn filter(request: &mut ScopeRequestV2, excluded: &BTreeSet<String>) {
    let has = |kind: &str, id: &str| excluded.contains(&format!("{kind}:{id}"));
    request
        .selection
        .channels
        .retain(|c| !has("channel", &c.portable_channel_id));
    request
        .selection
        .template_refs
        .retain(|r| !has("template", &format!("{}:{}", r.template_id, r.revision_id)));
    request
        .selection
        .adopt_template_defaults
        .retain(|r| !has("template", &format!("{}:{}", r.template_id, r.revision_id)));
    if excluded.contains("preferences") {
        request.selection.manager.preferences = false;
    }
    if excluded.contains("extensions") {
        request.selection.manager.extensions = false;
    }
    request
        .selection
        .manager
        .account_ids
        .retain(|id| !has("account", id));
    request
        .selection
        .manager
        .webdav_ids
        .retain(|id| !has("webdav", id));
    request
        .selection
        .manager
        .webdav_password_ids
        .retain(|id| !has("webdav", id));
    request.selection.skill_names.retain(|n| !has("skill", n));
    request
        .selection
        .extension_credentials
        .skill_names
        .retain(|n| !has("skill", n));
    request
        .selection
        .mcp_sources
        .retain(|m| !has("mcp", &m.name));
    request
        .selection
        .extension_credentials
        .mcp_names
        .retain(|n| !has("mcp", n));
    request.mcp_clients.retain(|n, _| !has("mcp", n));
}
pub(crate) fn snapshot(root: &Path) -> SyncResult<(Option<ScopeRequestV2>, BTreeSet<String>)> {
    let _guard = lock(root)?;
    let files = ServiceFilesV2::new(root)?;
    match record(&files)? {
        Some(r) => {
            let mut request = r.request;
            filter(&mut request, &r.restored_absences);
            Ok((r.initialized.then_some(request), r.restored_absences))
        }
        None => Ok((None, BTreeSet::new())),
    }
}
pub(crate) fn save_manual(root: &Path, request: ScopeRequestV2) -> SyncResult<()> {
    let _guard = lock(root)?;
    let files = ServiceFilesV2::new(root)?;
    // An explicit user selection is allowed to select a previously removed item.
    files.write(
        "scope.json",
        ScopeRecordV2 {
            version: 2,
            root_digest: files.root_digest.clone(),
            initialized: true,
            request,
            restored_absences: BTreeSet::new(),
        },
    )
}
pub(crate) fn initialize(root: &Path, mut initial: ScopeRequestV2) -> SyncResult<ScopeRequestV2> {
    let _guard = lock(root)?;
    let files = ServiceFilesV2::new(root)?;
    let previous = record(&files)?;
    if let Some(r) = previous.as_ref().filter(|r| r.initialized) {
        let mut saved = r.request.clone();
        filter(&mut saved, &r.restored_absences);
        return Ok(saved);
    }
    let restored_absences = previous.map(|r| r.restored_absences).unwrap_or_default();
    filter(&mut initial, &restored_absences);
    files.write(
        "scope.json",
        ScopeRecordV2 {
            version: 2,
            root_digest: files.root_digest.clone(),
            initialized: true,
            request: initial.clone(),
            restored_absences,
        },
    )?;
    Ok(initial)
}
pub(crate) fn freeze(root: &Path, removed: BTreeSet<String>) -> SyncResult<Option<ScopeEditV2>> {
    if removed.is_empty() {
        return Ok(None);
    }
    let _guard = lock(root)?;
    let files = ServiceFilesV2::new(root)?;
    let before = raw(root)?;
    let after = projection(&files, before.as_deref(), &removed)?;
    Ok(Some(ScopeEditV2 { before, after }))
}
fn projection(
    files: &ServiceFilesV2,
    before: Option<&[u8]>,
    removed: &BTreeSet<String>,
) -> SyncResult<Vec<u8>> {
    let parsed = match before {
        Some(bytes) => {
            let tree = super::super::schema_v2::parse_strict_json_v2(bytes).map_err(scope_error)?;
            let checked: CheckedRecordV2<ScopeRecordV2> =
                serde_json::from_value(tree).map_err(scope_error)?;
            if checked.digest != store::digest(&store::canonical(&checked.body)?)
                || checked.body.version != 2
                || checked.body.root_digest != files.root_digest
            {
                return Err(err(ErrorCode::SyncJournalInvalid, Step::Persist));
            }
            Some(checked.body)
        }
        None => None,
    };
    let mut r = parsed.unwrap_or_else(|| ScopeRecordV2 {
        version: 2,
        root_digest: files.root_digest.clone(),
        initialized: false,
        request: ScopeRequestV2 {
            selection: SelectionV2::default(),
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        },
        restored_absences: BTreeSet::new(),
    });
    r.restored_absences.extend(removed.iter().cloned());
    filter(&mut r.request, &r.restored_absences);
    let after = store::canonical(&CheckedRecordV2 {
        digest: store::digest(&store::canonical(&r)?),
        body: r,
    })?;
    if after.len() > 4 * 1024 * 1024 {
        return Err(err(ErrorCode::SyncLimit, Step::Persist));
    }
    Ok(after)
}
pub(crate) fn validate(
    root: &Path,
    edit: Option<&ScopeEditV2>,
    removed: &BTreeSet<String>,
) -> SyncResult<()> {
    if removed.is_empty() {
        return if edit.is_none() {
            Ok(())
        } else {
            Err(err(ErrorCode::SyncScopeChanged, Step::SourceGuard))
        };
    }
    let edit = edit.ok_or_else(|| err(ErrorCode::SyncScopeChanged, Step::SourceGuard))?;
    let files = ServiceFilesV2::new(root)?;
    if projection(&files, edit.before.as_deref(), removed)? != edit.after {
        return Err(err(ErrorCode::SyncScopeChanged, Step::SourceGuard));
    }
    Ok(())
}
// Caller holds lock(root) continuously through protection, content restore and receipt.
pub(crate) fn check(
    root: &Path,
    edit: &ScopeEditV2,
    started: bool,
    applied: bool,
) -> SyncResult<()> {
    let now = raw(root)?;
    if (!applied && now == edit.before) || started && now.as_deref() == Some(edit.after.as_slice())
    {
        Ok(())
    } else {
        Err(err(ErrorCode::SyncScopeChanged, Step::SourceGuard))
    }
}
pub(crate) fn apply(root: &Path, edit: &ScopeEditV2) -> SyncResult<()> {
    let now = raw(root)?;
    if now.as_deref() == Some(edit.after.as_slice()) {
        return Ok(());
    }
    if now != edit.before {
        return Err(err(ErrorCode::SyncScopeChanged, Step::SourceGuard));
    }
    crate::infrastructure::atomic_write::atomic_write(&path(root), &edit.after, 0o600)
        .map_err(scope_error)?;
    if raw(root)?.as_deref() != Some(edit.after.as_slice()) {
        return Err(scope_error("scope readback"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::model_config::objects_overlay_v2::{
        ChannelSelectionV2, McpSourceSelectionV2,
    };
    fn request() -> ScopeRequestV2 {
        let mut selection = SelectionV2 {
            channels: ["removed", "keep"]
                .into_iter()
                .map(|id| ChannelSelectionV2 {
                    portable_channel_id: id.into(),
                    model_ids: None,
                    family_ids: None,
                    include_connection: true,
                    include_key: true,
                    include_authorization: false,
                })
                .collect(),
            skill_names: vec!["gone".into(), "other".into()],
            ..Default::default()
        };
        selection.extension_credentials.skill_names = selection.skill_names.clone();
        selection.mcp_sources = vec![McpSourceSelectionV2 {
            name: "gone-mcp".into(),
            definition_digest: "synthetic".into(),
        }];
        selection.extension_credentials.mcp_names = vec!["gone-mcp".into()];
        selection.manager.preferences = true;
        selection.manager.extensions = true;
        ScopeRequestV2 {
            selection,
            mcp_clients: BTreeMap::from([("gone-mcp".into(), "managed".into())]),
            assets_reviewed: false,
        }
    }
    fn removed() -> BTreeSet<String> {
        [
            "channel:removed",
            "skill:gone",
            "mcp:gone-mcp",
            "extensions",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect()
    }
    #[test]
    fn frozen_scope_update_keeps_other_choices_and_manual_reselection_survives_reopen() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().canonicalize().unwrap();
        save_manual(&d, request()).unwrap();
        let edit = freeze(&d, removed()).unwrap().unwrap();
        {
            let _guard = lock(&d).unwrap();
            check(&d, &edit, false, false).unwrap();
            apply(&d, &edit).unwrap();
            apply(&d, &edit).unwrap();
        }
        let (saved, _) = snapshot(&d).unwrap();
        let saved = saved.unwrap();
        assert_eq!(
            saved
                .selection
                .channels
                .iter()
                .map(|c| c.portable_channel_id.as_str())
                .collect::<Vec<_>>(),
            vec!["keep"]
        );
        assert_eq!(saved.selection.skill_names, vec!["other"]);
        assert_eq!(
            saved.selection.extension_credentials.skill_names,
            vec!["other"]
        );
        assert!(
            saved.selection.mcp_sources.is_empty()
                && saved.mcp_clients.is_empty()
                && saved.selection.extension_credentials.mcp_names.is_empty()
        );
        assert!(saved.selection.manager.preferences && !saved.selection.manager.extensions);
        save_manual(&d, request()).unwrap();
        assert!(snapshot(&d).unwrap().1.is_empty());
        assert_eq!(
            snapshot(&d).unwrap().0.unwrap().selection,
            request().selection
        );
        let _guard = lock(&d).unwrap();
        assert!(check(&d, &edit, true, true).is_err());
    }
    #[test]
    fn first_catalog_still_selects_every_other_object_but_not_restored_absences() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().canonicalize().unwrap();
        let edit = freeze(&d, removed()).unwrap().unwrap();
        assert!(snapshot(&d).unwrap().0.is_none()); // preview is not adoption
        {
            let _guard = lock(&d).unwrap();
            apply(&d, &edit).unwrap();
        }
        assert!(snapshot(&d).unwrap().0.is_none()); // no catalog initialized yet
        let initial = initialize(&d, request()).unwrap();
        assert_eq!(initial.selection.channels.len(), 1);
        assert_eq!(initial.selection.channels[0].portable_channel_id, "keep");
        assert_eq!(initial.selection.skill_names, vec!["other"]);
        assert!(initial.selection.manager.preferences && !initial.selection.manager.extensions);
        let mut expanded = request();
        expanded.selection.skill_names.push("later-new".into());
        assert_eq!(
            initialize(&d, expanded).unwrap().selection,
            initial.selection
        ); // later additions remain unchecked
    }
    #[test]
    fn changed_scope_rejects_the_frozen_plan_without_overwriting_user_choices() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().canonicalize().unwrap();
        save_manual(&d, request()).unwrap();
        let edit = freeze(&d, removed()).unwrap().unwrap();
        let mut changed = request();
        changed.selection.manager.preferences = false;
        save_manual(&d, changed.clone()).unwrap();
        {
            let _guard = lock(&d).unwrap();
            assert!(check(&d, &edit, false, false).is_err());
            assert!(apply(&d, &edit).is_err());
        }
        assert_eq!(
            snapshot(&d).unwrap().0.unwrap().selection,
            changed.selection
        );
    }
    #[test]
    fn edited_plan_cannot_deselect_an_unrelated_object_by_rewriting_its_checksum() {
        let t = tempfile::tempdir().unwrap();
        let d = t.path().canonicalize().unwrap();
        save_manual(&d, request()).unwrap();
        let mut edit = freeze(&d, removed()).unwrap().unwrap();
        validate(&d, Some(&edit), &removed()).unwrap();
        let mut checked: CheckedRecordV2<ScopeRecordV2> =
            serde_json::from_slice(&edit.after).unwrap();
        checked.body.request.selection.manager.preferences = false;
        checked.digest = store::digest(&store::canonical(&checked.body).unwrap());
        edit.after = store::canonical(&checked).unwrap();
        assert!(validate(&d, Some(&edit), &removed()).is_err());
        assert!(
            snapshot(&d)
                .unwrap()
                .0
                .unwrap()
                .selection
                .manager
                .preferences
        );
    }
}
