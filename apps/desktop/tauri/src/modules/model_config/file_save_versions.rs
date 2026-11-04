//! Events for ordinary config-file saves. Call outside config/layout/store locks.
//! Transfers keep using save_observed_checked and inherit their source receipts.
use super::super::{
    config_file::{self, SaveRequest, SaveResult},
    config_remote::{self, Scope},
    schema::*,
};
use super::SaveState;
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

const JOURNAL: &str = "manager-state/saved-file-save-pending.json";
const JOURNAL_LIMIT: u64 = 8 * 1024 * 1024;
thread_local! { static ADMITTED: std::cell::RefCell<BTreeSet<PathBuf>> = const { std::cell::RefCell::new(BTreeSet::new()) }; }
/// Cross-process admission is reentrant only within the owning thread, so
/// source capture may call the normal snapshot API without releasing protection.
pub struct Admission {
    target: PathBuf,
    lock: Option<TargetFileLock>,
}
impl Drop for Admission {
    fn drop(&mut self) {
        if self.lock.is_some() {
            ADMITTED.with(|held| {
                held.borrow_mut().remove(&self.target);
            });
        }
    }
}
pub fn admit(root: &Path, home: &Path) -> ModelResult<Admission> {
    let target = root.join(JOURNAL);
    safety::check_path(root, &target, true)?;
    if ADMITTED.with(|held| held.borrow().contains(&target)) {
        return Ok(Admission { target, lock: None });
    }
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&target),
        true,
    )?;
    let lock = TargetFileLock::lock(&target)?;
    ADMITTED.with(|held| {
        held.borrow_mut().insert(target.clone());
    });
    let admission = Admission {
        target,
        lock: Some(lock),
    };
    recover(root, home).map_err(unconfirmed)?;
    Ok(admission)
}
pub struct Event {
    id: String,
    channel_ids: BTreeSet<String>,
    global: bool,
    before: SaveState,
    _admission: Admission,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Pending {
    id: String,
    home_binding: String,
    channel_ids: BTreeSet<String>,
    global: bool,
    before: SaveState,
    after: Option<SaveState>,
    raw_digest: String,
    metadata_digest: String,
    saved_at: String,
}
fn unconfirmed(error: ModelError) -> ModelError {
    if error.code.starts_with("rename_") {
        error
    } else {
        invalid(
            "saved_content_version_unconfirmed",
            "配置已保存，但保存版本待恢复认证；原保存事件已保留，请重新读取核对",
        )
    }
}
fn pending(root: &Path) -> ModelResult<Option<Pending>> {
    let path = root.join(JOURNAL);
    safety::check_path(root, &path, true)?;
    if !path.exists() {
        return Ok(None);
    }
    let event: Pending = serde_json::from_slice(&safety::read_file(&path, JOURNAL_LIMIT)?)?;
    require_uuid(&event.id)?;
    if chrono::DateTime::parse_from_rfc3339(&event.saved_at).is_err() {
        return Err(invalid("content_version", "原保存事件时间无效"));
    }
    Ok(Some(event))
}
fn persist(root: &Path, event: &Pending) -> ModelResult<()> {
    let path = root.join(JOURNAL);
    safety::check_path(root, &path, true)?;
    let bytes = serde_json::to_vec(event)?;
    if bytes.len() as u64 > JOURNAL_LIMIT {
        return Err(invalid("content_version", "保存事件超过上限"));
    }
    atomic_write(&path, &bytes, 0o600)?;
    Ok(())
}
fn clear(root: &Path) -> ModelResult<()> {
    std::fs::remove_file(root.join(JOURNAL))?;
    Ok(())
}
fn matches(root: &Path, home: &Path, event: &Pending, snapshot: &Snapshot) -> ModelResult<bool> {
    Ok(event.home_binding == super::key(&home)?
        && event.raw_digest == snapshot.policy_digest
        && event.metadata_digest == config_file::metadata_digest(root)?)
}
/// Runs in the owner's real readback callback, before a rename is marked
/// committed or cleans old credentials. Failure is still a core rollback.
pub fn acknowledge(
    root: &Path,
    home: &Path,
    event: &Event,
    snapshot: &Snapshot,
    raw: &serde_json::Value,
) -> ModelResult<()> {
    let after = freeze(root, home, &event.channel_ids, event.global, snapshot, raw)?;
    persist(
        root,
        &Pending {
            id: event.id.clone(),
            home_binding: super::key(&home)?,
            channel_ids: event.channel_ids.clone(),
            global: event.global,
            before: event.before.clone(),
            after: Some(after),
            raw_digest: snapshot.policy_digest.clone(),
            metadata_digest: config_file::metadata_digest(root)?,
            saved_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        },
    )
}
fn stable_bytes(root: &Path, home: &Path, event: &Pending) -> ModelResult<()> {
    let path = home.join("config.json");
    safety::check_path(home, &path, true)?;
    let raw = if path.exists() {
        safety::read_file(&path, 16 * 1024 * 1024)?
    } else {
        b"{}".to_vec()
    };
    if super::super::store::sha256(&raw) != event.raw_digest
        || config_file::metadata_digest(root)? != event.metadata_digest
    {
        return Err(invalid("file_changed", "原保存文件摘要已变化"));
    }
    Ok(())
}
fn frozen_matches_current(
    root: &Path,
    home: &Path,
    event: &Pending,
    snapshot: &Snapshot,
) -> ModelResult<bool> {
    let Some(after) = event.after.as_ref() else {
        return Ok(false);
    };
    stable_bytes(root, home, event)?;
    let raw = zeroize::Zeroizing::new(safety::read_file(
        &home.join("config.json"),
        16 * 1024 * 1024,
    )?);
    let mut value: serde_json::Value = serde_json::from_slice(&raw)?;
    let current = freeze(
        root,
        home,
        &event.channel_ids,
        event.global,
        snapshot,
        &value,
    );
    super::super::provider_auth::erase_value(&mut value);
    Ok(current? == *after)
}
fn finish_pending(root: &Path, home: &Path, event: &mut Pending) -> ModelResult<()> {
    checkpoint("before_replay_capture")?;
    let actual = config_file::snapshot(root, home)?;
    if !matches(root, home, event, &actual)? {
        return Err(invalid("file_changed", "原保存后内容改变，未认证旧事件"));
    }
    if !frozen_matches_current(root, home, event, &actual)? {
        return Err(invalid(
            "content_version_source_changed",
            "原保存凭据或内容已改变，未认证后来值",
        ));
    }
    checkpoint("before_version_commit")?;
    super::record_at_validated(
        root,
        &event.before,
        event.after.as_ref().unwrap(),
        &event.saved_at,
        &event.id,
        || stable_bytes(root, home, event),
    )?;
    checkpoint("after_version_commit")?;
    let actual = config_file::snapshot(root, home)?;
    if !matches(root, home, event, &actual)? || !frozen_matches_current(root, home, event, &actual)?
    {
        return Err(invalid(
            "file_changed",
            "清理保存事件前内容改变，保留原事件待核对",
        ));
    }
    clear(root)
}
fn recover(root: &Path, home: &Path) -> ModelResult<()> {
    let Some(mut event) = pending(root)? else {
        return Ok(());
    };
    if event.home_binding != super::key(&home)? {
        return Err(invalid("file_changed", "原保存事件属于其它配置目录"));
    }
    // snapshot first completes/compensates the existing rename journal. A
    // rolled-back/superseded operation can never certify its abandoned image.
    let actual = config_file::snapshot(root, home)?;
    if !matches(root, home, &event, &actual)?
        || !frozen_matches_current(root, home, &event, &actual)?
    {
        // An old unfrozen event or externally changed secret cannot be dated
        // with a past operation's time. Discard only the obsolete metadata event.
        return clear(root);
    }
    finish_pending(root, home, &mut event)
}
fn portable_state(result: ModelResult<SaveState>) -> ModelResult<SaveState> {
    match result {
        Err(error)
            if [
                "provider_fields",
                "provider_auth_reference",
                "provider_auth_portable",
                "credential_unavailable",
            ]
            .contains(&error.code.as_str()) =>
        {
            Ok(SaveState::new())
        }
        other => other,
    }
}
fn freeze(
    _root: &Path,
    home: &Path,
    ids: &BTreeSet<String>,
    global: bool,
    snapshot: &Snapshot,
    raw: &serde_json::Value,
) -> ModelResult<SaveState> {
    let mut out = SaveState::new();
    if global {
        let all = snapshot
            .policy
            .channels
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        out.extend(portable_state(
            super::super::file_objects::saved_file_atoms(home, snapshot, raw, &all, true, false),
        )?);
    }
    for channel in snapshot
        .policy
        .channels
        .iter()
        .filter(|c| ids.contains(&c.id))
    {
        out.extend(portable_state(
            super::super::file_objects::saved_file_atoms(
                home,
                snapshot,
                raw,
                std::slice::from_ref(&channel.id),
                false,
                false,
            ),
        )?);
        out.extend(portable_state(
            super::super::file_objects::saved_file_atoms(
                home,
                snapshot,
                raw,
                std::slice::from_ref(&channel.id),
                false,
                true,
            ),
        )?);
    }
    Ok(out)
}
fn capture_supported(root: &Path, scope: &Scope) -> ModelResult<SaveState> {
    match config_remote::capture_save(root, scope) {
        Ok(state) => Ok(state),
        // The ordinary save preserves fields and unresolved credential references
        // which the portable exporter deliberately cannot carry. Such content
        // remains uncertified; it must not prevent the owner's valid save or
        // be assigned a made-up time. IO/version-store failures still propagate.
        Err(e)
            if [
                "provider_fields",
                "provider_auth_reference",
                "provider_auth_portable",
                "credential_unavailable",
            ]
            .contains(&e.code.as_str()) =>
        {
            Ok(SaveState::new())
        }
        Err(e) => Err(e),
    }
}
fn capture(
    root: &Path,
    home: &Path,
    ids: &BTreeSet<String>,
    global: bool,
    snapshot: &Snapshot,
) -> ModelResult<SaveState> {
    let mut out = SaveState::new();
    if global {
        out.extend(capture_supported(
            root,
            &Scope {
                all_channels: true,
                include_global: true,
                preferences: false,
                extensions: false,
                upstream_root: Some(home.into()),
                ..Default::default()
            },
        )?);
    }
    for channel in snapshot
        .policy
        .channels
        .iter()
        .filter(|c| ids.contains(&c.id))
    {
        let scope = Scope {
            channel_ids: vec![channel.id.clone()],
            preferences: false,
            extensions: false,
            upstream_root: Some(home.into()),
            ..Default::default()
        };
        // Public-only receipts are independent of secret availability.
        out.extend(capture_supported(root, &scope)?);
        out.extend(capture_supported(
            root,
            &Scope {
                include_keys: true,
                credential_ids: Some(vec![format!("channel:{}", channel.id)]),
                ..scope
            },
        )?);
    }
    Ok(out)
}
/// Capture only this request's effective targets. New IDs need no prior object.
/// Unrelated channel drafts in a scoped request never cause credential reads.
pub fn begin(root: &Path, home: &Path, request: &SaveRequest) -> ModelResult<Event> {
    let admission = admit(root, home)?;
    let snapshot = config_file::snapshot(root, home)?;
    let channel_ids = if let Some(scope) = &request.scope {
        BTreeSet::from([scope.channel_id.clone()])
    } else {
        request
            .policy
            .channels
            .iter()
            .filter(|candidate| {
                snapshot
                    .policy
                    .channels
                    .iter()
                    .find(|old| old.id == candidate.id)
                    != Some(*candidate)
                    || request
                        .credentials
                        .iter()
                        .any(|edit| edit.channel_id == candidate.id)
            })
            .map(|c| c.id.clone())
            .collect()
    };
    let global = request.scope.is_none()
        && (snapshot.policy.template_refs != request.policy.template_refs
            || snapshot.policy.default_channel_id != request.policy.default_channel_id
            || snapshot
                .policy
                .channels
                .iter()
                .map(|c| &c.id)
                .collect::<Vec<_>>()
                != request
                    .policy
                    .channels
                    .iter()
                    .map(|c| &c.id)
                    .collect::<Vec<_>>());
    let before = capture(root, home, &channel_ids, global, &snapshot)?;
    if config_file::snapshot(root, home)? != snapshot {
        return Err(invalid(
            "file_changed",
            "捕获保存版本期间配置改变，尚未保存，请重新读取",
        ));
    }
    Ok(Event {
        id: uuid::Uuid::new_v4().to_string(),
        channel_ids,
        global,
        before,
        _admission: admission,
    })
}
/// Call only for Ok(SaveResult), after save_internal released all its guards.
/// Do not gate on result.changed: layout-only semantic saves also have receipts.
pub fn finish(root: &Path, home: &Path, event: Event, result: &SaveResult) -> ModelResult<()> {
    let work = || {
        let mut committed =
            pending(root)?.ok_or_else(|| invalid("content_version", "保存事件缺失"))?;
        if committed.id != event.id || committed.raw_digest != result.snapshot.policy_digest {
            return Err(invalid("content_version", "保存事件身份不同"));
        }
        finish_pending(root, home, &mut committed)
    };
    work().map_err(unconfirmed)
}

#[cfg(test)]
type TestHook = Box<dyn FnMut(&str) -> ModelResult<()>>;
#[cfg(test)]
thread_local! { static TEST_HOOK: std::cell::RefCell<Option<TestHook>> = const { std::cell::RefCell::new(None) }; }
fn checkpoint(stage: &str) -> ModelResult<()> {
    #[cfg(test)]
    {
        TEST_HOOK.with(|hook| match &mut *hook.borrow_mut() {
            Some(hook) => hook(stage),
            None => Ok(()),
        })
    }
    #[cfg(not(test))]
    {
        let _ = stage;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::{
        config_file::{CredentialEdit, SaveScope},
        file_objects,
    };
    use super::*;
    struct Fixture {
        _temp: tempfile::TempDir,
        root: std::path::PathBuf,
        home: std::path::PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            file_objects::test_credentials::install();
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().join("data");
            let home = temp.path().join("official");
            std::fs::create_dir_all(&root).unwrap();
            std::fs::create_dir_all(&home).unwrap();
            // A request-local hook must never resolve the unrelated missing key.
            std::fs::write(home.join("config.json"), serde_json::to_vec(&serde_json::json!({"providers":{
                "selected":{"adapter":"openai-chat","authMode":"key","baseUrl":"https://selected.invalid/v1","apiKey":"synthetic-selected","models":["model-a"]},
                "unrelated":{"adapter":"openai-chat","authMode":"key","baseUrl":"https://unrelated.invalid/v1","apiKey":"keychain:NEVER-READ-UNRELATED","models":["model-b"]}
            }})).unwrap()).unwrap();
            Self {
                _temp: temp,
                root,
                home,
            }
        }
        fn request(&self, scoped: bool) -> SaveRequest {
            let snapshot = config_file::snapshot(&self.root, &self.home).unwrap();
            let id = snapshot
                .policy
                .channels
                .iter()
                .find(|c| c.connection.provider_id == "selected")
                .unwrap()
                .id
                .clone();
            SaveRequest {
                expected_file: snapshot.policy_digest,
                policy: snapshot.policy,
                credentials: vec![],
                scope: scoped.then_some(SaveScope { channel_id: id }),
            }
        }
        fn fact(&self, id: String) -> file_objects::ContentFact {
            config_remote::source_fact(
                &self.root,
                &Scope {
                    channel_ids: vec![id],
                    include_keys: true,
                    preferences: false,
                    extensions: false,
                    upstream_root: Some(self.home.clone()),
                    ..Default::default()
                },
            )
            .unwrap()
        }
    }
    #[test]
    fn ordinary_save_is_bounded_and_covers_scoped_key_only_and_new_objects() {
        for scoped in [true, false] {
            let f = Fixture::new();
            let mut request = f.request(scoped);
            let id = request
                .policy
                .channels
                .iter()
                .find(|c| c.connection.provider_id == "selected")
                .unwrap()
                .id
                .clone();
            request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .nodes[0]
                .url = "https://saved.invalid/v1".into();
            let saved = config_file::save(&f.root, &f.home, request).unwrap();
            assert!(saved.changed);
            assert!(f.fact(id.clone()).saved_at.is_some());
            let original = f.fact(id.clone());
            let same = config_file::save(&f.root, &f.home, f.request(scoped)).unwrap();
            assert!(!same.changed);
            assert_eq!(f.fact(id.clone()), original);
            let mut metadata_only = f.request(scoped);
            metadata_only
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .label = "layout label".into();
            let saved = config_file::save(&f.root, &f.home, metadata_only).unwrap();
            assert!(
                !saved.changed,
                "file bytes are unchanged for layout-only updates"
            );
            assert_ne!(f.fact(id.clone()).content_digest, original.content_digest);
            assert!(f.fact(id.clone()).saved_at.is_some());
            let mut request = f.request(scoped);
            request.credentials.push(CredentialEdit {
                channel_id: id.clone(),
                action: "replace".into(),
                value: Some("synthetic-key-only".into()),
            });
            config_file::save(&f.root, &f.home, request).unwrap();
            assert_ne!(f.fact(id).content_digest, original.content_digest);
            assert!(f.fact(original_id(&f)).saved_at.is_some());
        }
        let rename = Fixture::new();
        let mut request = rename.request(true);
        let id = request.scope.as_ref().unwrap().channel_id.clone();
        let channel = request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap();
        channel.label = "renamed".into();
        channel.connection.provider_id = "renamed".into();
        let result = config_file::save(&rename.root, &rename.home, request).unwrap();
        assert_eq!(
            result
                .snapshot
                .policy
                .channels
                .iter()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .provider_id,
            "renamed"
        );
        assert!(rename.fact(id).saved_at.is_some());
        let f = Fixture::new();
        let mut request = f.request(true);
        let mut added = request
            .policy
            .channels
            .iter()
            .find(|c| c.connection.provider_id == "selected")
            .unwrap()
            .clone();
        let id = uuid::Uuid::new_v4().to_string();
        added.id = id.clone();
        added.label = "added".into();
        added.connection.provider_id = "added".into();
        request.policy.channels.push(added);
        request.scope = Some(SaveScope {
            channel_id: id.clone(),
        });
        request.credentials.push(CredentialEdit {
            channel_id: id.clone(),
            action: "replace".into(),
            value: Some("synthetic-new-object".into()),
        });
        config_file::save(&f.root, &f.home, request).unwrap();
        assert!(f.fact(id).saved_at.is_some());
    }
    fn original_id(f: &Fixture) -> String {
        f.request(true).scope.unwrap().channel_id
    }
    #[test]
    fn failure_and_post_save_receipt_failure_do_not_report_a_certified_save() {
        let f = Fixture::new();
        let before_file = std::fs::read(f.home.join("config.json")).unwrap();
        let mut invalid = f.request(true);
        invalid.expected_file = "stale".into();
        assert!(config_file::save(&f.root, &f.home, invalid).is_err());
        assert_eq!(
            std::fs::read(f.home.join("config.json")).unwrap(),
            before_file
        );
        assert!(!f
            .root
            .join("manager-state/saved-content-versions.json")
            .exists());
        let mut request = f.request(true);
        request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.connection.provider_id == "selected")
            .unwrap()
            .connection
            .nodes[0]
            .url = "https://saved.invalid/v2".into();
        let event = begin(&f.root, &f.home, &request).unwrap();
        let result = config_file::save(&f.root, &f.home, request).unwrap();
        let saved = std::fs::read(f.home.join("config.json")).unwrap();
        let version_file = f.root.join("manager-state/saved-content-versions.json");
        if version_file.exists() {
            std::fs::remove_file(&version_file).unwrap();
        }
        std::fs::create_dir(&version_file).unwrap();
        assert_eq!(
            finish(&f.root, &f.home, event, &result).unwrap_err().code,
            "saved_content_version_unconfirmed"
        );
        assert_eq!(std::fs::read(f.home.join("config.json")).unwrap(), saved);
    }
    #[test]
    fn post04_review_final_check_and_version_commit_serialize_real_saves() {
        let f = std::sync::Arc::new(Fixture::new());
        let (paused_tx, paused_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let a_fixture = f.clone();
        let a = std::thread::spawn(move || {
            TEST_HOOK.with(|h| {
                *h.borrow_mut() = Some(Box::new(move |stage| {
                    if stage == "before_version_commit" {
                        paused_tx.send(()).unwrap();
                        let _ = resume_rx.recv();
                    }
                    Ok(())
                }))
            });
            let mut request = a_fixture.request(true);
            let id = request.scope.as_ref().unwrap().channel_id.clone();
            request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .nodes[0]
                .url = "https://writer-a.invalid/v1".into();
            config_file::save(&a_fixture.root, &a_fixture.home, request)
        });
        paused_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let b_fixture = f.clone();
        let (b_ready_tx, b_ready_rx) = std::sync::mpsc::channel();
        let b = std::thread::spawn(move || {
            TEST_HOOK.with(|h| {
                *h.borrow_mut() = Some(Box::new(move |stage| {
                    if stage == "before_version_commit" {
                        b_ready_tx.send(()).unwrap();
                    }
                    Ok(())
                }))
            });
            let mut request = b_fixture.request(true);
            let id = request.scope.as_ref().unwrap().channel_id.clone();
            request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .nodes[0]
                .url = "https://writer-b.invalid/v1".into();
            let result = config_file::save(&b_fixture.root, &b_fixture.home, request);
            done_tx.send(()).unwrap();
            result
        });
        let serialized = b_ready_rx
            .recv_timeout(std::time::Duration::from_millis(1500))
            .is_err();
        if !serialized {
            done_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
        }
        resume_tx.send(()).unwrap();
        a.join().unwrap().unwrap();
        b.join().unwrap().unwrap();
        assert!(
            serialized,
            "writer B completed between A's final check and version commit"
        );
        let id = original_id(&f);
        let fact = f.fact(id);
        assert!(
            fact.saved_at.is_some(),
            "real writer B must retain its own matching saved version"
        );
        let raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(f.home.join("config.json")).unwrap()).unwrap();
        assert_eq!(
            raw["providers"]["selected"]["baseUrl"],
            "https://writer-b.invalid/v1"
        );
    }
    #[test]
    fn post04_review_committed_rename_replays_original_time_after_version_lock_timeout() {
        use crate::infrastructure::locking::TargetFileLock;
        for entry in ["snapshot", "save", "source_state"] {
            let f = Fixture::new();
            let slot = uuid::Uuid::new_v4().to_string();
            let old = format!("selected/{slot}");
            let new = format!("renamed/{slot}");
            keyring::Entry::new("opencodex.provider-api-key.v1", &old)
                .unwrap()
                .set_password("synthetic-moved-key")
                .unwrap();
            let mut raw: serde_json::Value =
                serde_json::from_slice(&std::fs::read(f.home.join("config.json")).unwrap())
                    .unwrap();
            raw["providers"]["selected"]["apiKey"] = serde_json::json!(format!("keychain:{old}"));
            std::fs::write(
                f.home.join("config.json"),
                serde_json::to_vec(&raw).unwrap(),
            )
            .unwrap();
            let mut request = f.request(true);
            let id = request.scope.as_ref().unwrap().channel_id.clone();
            let channel = request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap();
            channel.label = "renamed".into();
            channel.connection.provider_id = "renamed".into();
            let intended = request.policy.clone();
            let guard =
                TargetFileLock::lock(&f.root.join("manager-state/saved-content-versions.json"))
                    .unwrap();
            let error = config_file::save(&f.root, &f.home, request).err().unwrap();
            assert_eq!(error.code, "saved_content_version_unconfirmed");
            let actual = std::fs::read(f.home.join("config.json")).unwrap();
            let value: serde_json::Value = serde_json::from_slice(&actual).unwrap();
            assert!(value["providers"].get("selected").is_none());
            assert!(value["providers"].get("renamed").is_some());
            assert!(keyring::Entry::new("opencodex.provider-api-key.v1", &old)
                .unwrap()
                .get_password()
                .is_err());
            assert_eq!(
                keyring::Entry::new("opencodex.provider-api-key.v1", &new)
                    .unwrap()
                    .get_password()
                    .unwrap(),
                "synthetic-moved-key"
            );
            let original_time = pending(&f.root).unwrap().unwrap().saved_at;
            drop(guard);
            match entry {
                "snapshot" => {
                    config_file::snapshot(&f.root, &f.home).unwrap();
                }
                "save" => {
                    let noop = SaveRequest {
                        expected_file: super::super::super::store::sha256(&actual),
                        policy: intended,
                        credentials: vec![],
                        scope: Some(SaveScope {
                            channel_id: id.clone(),
                        }),
                    };
                    assert!(!config_file::save(&f.root, &f.home, noop).unwrap().changed);
                }
                _ => {
                    config_remote::source_state(
                        &f.root,
                        &Scope {
                            channel_ids: vec![id.clone()],
                            include_keys: true,
                            preferences: false,
                            extensions: false,
                            upstream_root: Some(f.home.clone()),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                }
            }
            let fact = f.fact(id.clone());
            assert_eq!(
                fact.saved_at
                    .as_deref()
                    .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
                Some(chrono::DateTime::parse_from_rfc3339(&original_time).unwrap()),
                "replay must use the original successful-save instant at {entry}"
            );
            assert!(pending(&f.root).unwrap().is_none());
            let versions =
                std::fs::read(f.root.join("manager-state/saved-content-versions.json")).unwrap();
            let snapshot = config_file::snapshot(&f.root, &f.home).unwrap();
            let noop = SaveRequest {
                expected_file: snapshot.policy_digest,
                policy: snapshot.policy,
                credentials: vec![],
                scope: Some(SaveScope {
                    channel_id: id.clone(),
                }),
            };
            config_file::save(&f.root, &f.home, noop).unwrap();
            assert_eq!(
                std::fs::read(f.root.join("manager-state/saved-content-versions.json")).unwrap(),
                versions,
                "noop cannot refresh a replayed receipt"
            );
        }
    }
    #[test]
    fn post04_review_noop_does_not_certify_unknown_legacy_content() {
        let f = Fixture::new();
        let id = original_id(&f);
        assert!(f.fact(id.clone()).saved_at.is_none());
        let request = f.request(true);
        let event = begin(&f.root, &f.home, &request).unwrap();
        let legacy = event.before.iter().map(|(key,digest)| (key.clone(), serde_json::json!({"digest":digest,"revision":7,"saved_at":"2026-10-30T00:00:00Z"}))).collect::<std::collections::BTreeMap<_,_>>();
        drop(event);
        let path = f.root.join("manager-state/saved-content-versions.json");
        std::fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        let original = std::fs::read(&path).unwrap();
        assert!(
            !config_file::save(&f.root, &f.home, request)
                .unwrap()
                .changed
        );
        assert!(f.fact(id).saved_at.is_none());
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
    #[test]
    fn post04_review_lost_metadata_reply_replays_once_without_new_time_or_revision() {
        let f = Fixture::new();
        let mut request = f.request(true);
        let id = request.scope.as_ref().unwrap().channel_id.clone();
        request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap()
            .connection
            .nodes[0]
            .url = "https://saved-before-reply.invalid/v1".into();
        TEST_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(|stage| {
                if stage == "after_version_commit" {
                    Err(invalid("fixture_lost_reply", "synthetic interruption"))
                } else {
                    Ok(())
                }
            }))
        });
        assert_eq!(
            config_file::save(&f.root, &f.home, request)
                .err()
                .unwrap()
                .code,
            "saved_content_version_unconfirmed"
        );
        TEST_HOOK.with(|h| *h.borrow_mut() = None);
        let original_time = pending(&f.root).unwrap().unwrap().saved_at;
        let path = f.root.join("manager-state/saved-content-versions.json");
        let original = std::fs::read(&path).unwrap();
        config_file::snapshot(&f.root, &f.home).unwrap();
        assert_eq!(std::fs::read(path).unwrap(), original);
        assert!(pending(&f.root).unwrap().is_none());
        assert_eq!(
            f.fact(id)
                .saved_at
                .as_deref()
                .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
            Some(chrono::DateTime::parse_from_rfc3339(&original_time).unwrap())
        );
    }
    #[test]
    fn post04_review_version_commit_never_waits_for_a_template_owner_lock() {
        let f = std::sync::Arc::new(Fixture::new());
        let (locked_tx, locked_rx) = std::sync::mpsc::channel();
        let (resume_tx, resume_rx) = std::sync::mpsc::channel();
        let a_fixture = f.clone();
        let a = std::thread::spawn(move || {
            super::super::TEST_VERSION_HOOK.with(|h| {
                *h.borrow_mut() = Some(Box::new(move |stage| {
                    if stage == "version_locked" {
                        locked_tx.send(()).unwrap();
                        resume_rx.recv().unwrap();
                    }
                    Ok(())
                }))
            });
            let mut request = a_fixture.request(true);
            let id = request.scope.as_ref().unwrap().channel_id.clone();
            request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .nodes[0]
                .url = "https://file-owner.invalid/v1".into();
            config_file::save(&a_fixture.root, &a_fixture.home, request)
        });
        locked_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        let (waiting_tx, waiting_rx) = std::sync::mpsc::channel();
        let b_fixture = f.clone();
        let b = std::thread::spawn(move || {
            let store = super::super::super::store::ModelStore::new(&b_fixture.root);
            let snapshot = store.load_library().unwrap();
            super::super::TEST_VERSION_HOOK.with(|h| {
                *h.borrow_mut() = Some(Box::new(move |stage| {
                    if stage == "before_version_lock" {
                        waiting_tx.send(()).unwrap();
                    }
                    Ok(())
                }))
            });
            let mut content = TemplateContent::default();
            content.matching.include.push(Pattern {
                kind: MatchKind::Prefix,
                value: "parallel-owner".into(),
            });
            store.save(SaveTemplate {
                expected_library: snapshot.library_digest,
                template_id: None,
                base_revision: None,
                label: "parallel-template".into(),
                content,
            })
        });
        waiting_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .unwrap();
        resume_tx.send(()).unwrap();
        let file_result = a.join().unwrap();
        let template_result = b.join().unwrap();
        assert!(
            file_result.is_ok(),
            "file commit must not take ModelStore under version lock: {:?}",
            file_result.err()
        );
        assert!(
            template_result.is_ok(),
            "template committed file must retain a certified receipt: {:?}",
            template_result.err()
        );
        assert!(f.fact(original_id(&f)).saved_at.is_some());
    }
    #[test]
    fn post04_review_recovery_never_dates_future_keychain_content_with_the_original_time() {
        let f = Fixture::new();
        let account = format!("selected/{}", uuid::Uuid::new_v4());
        let credential = keyring::Entry::new("opencodex.provider-api-key.v1", &account).unwrap();
        credential.set_password("synthetic-K1").unwrap();
        let mut raw: serde_json::Value =
            serde_json::from_slice(&std::fs::read(f.home.join("config.json")).unwrap()).unwrap();
        raw["providers"]["selected"]["apiKey"] = serde_json::json!(format!("keychain:{account}"));
        std::fs::write(
            f.home.join("config.json"),
            serde_json::to_vec(&raw).unwrap(),
        )
        .unwrap();
        let mut request = f.request(true);
        let id = request.scope.as_ref().unwrap().channel_id.clone();
        request
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap()
            .connection
            .nodes[0]
            .url = "https://saved-K1.invalid/v1".into();
        TEST_HOOK.with(|h| {
            *h.borrow_mut() = Some(Box::new(|stage| {
                if stage == "before_replay_capture" {
                    Err(invalid(
                        "fixture_interrupted",
                        "after business readback before version capture",
                    ))
                } else {
                    Ok(())
                }
            }))
        });
        assert_eq!(
            config_file::save(&f.root, &f.home, request)
                .err()
                .unwrap()
                .code,
            "saved_content_version_unconfirmed"
        );
        TEST_HOOK.with(|h| *h.borrow_mut() = None);
        let bytes = std::fs::read(f.home.join("config.json")).unwrap();
        let original_time = pending(&f.root).unwrap().unwrap().saved_at;
        credential.set_password("synthetic-K2").unwrap();
        config_file::snapshot(&f.root, &f.home).unwrap();
        let current = f.fact(id);
        assert!(
            current.saved_at.is_none(),
            "K2 was never saved at {original_time}"
        );
        assert_eq!(std::fs::read(f.home.join("config.json")).unwrap(), bytes);
    }
    #[test]
    fn post04_review_crash_readback_freezes_credentials_across_real_process_exit() {
        const CHILD: &str = "OCX_POST04_CRASH_RECEIPT_MANIFEST";
        if let Some(marker) = std::env::var_os(CHILD) {
            let f = Fixture::new();
            let account = format!("selected/{}", uuid::Uuid::new_v4());
            keyring::Entry::new("opencodex.provider-api-key.v1", &account)
                .unwrap()
                .set_password("synthetic-original-K1")
                .unwrap();
            let mut raw: serde_json::Value =
                serde_json::from_slice(&std::fs::read(f.home.join("config.json")).unwrap())
                    .unwrap();
            raw["providers"]["selected"]["apiKey"] =
                serde_json::json!(format!("keychain:{account}"));
            std::fs::write(
                f.home.join("config.json"),
                serde_json::to_vec(&raw).unwrap(),
            )
            .unwrap();
            let mut request = f.request(true);
            let id = request.scope.as_ref().unwrap().channel_id.clone();
            request
                .policy
                .channels
                .iter_mut()
                .find(|c| c.id == id)
                .unwrap()
                .connection
                .nodes[0]
                .url = "https://crash-readback.invalid/v1".into();
            let root = f.root.clone();
            let home = f.home.clone();
            TEST_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(move |stage| {
                if stage == "before_replay_capture" {
                    let event = pending(&root)?.unwrap();
                    assert!(event.after.is_some(), "successful readback must already freeze credential content");
                    let manifest = serde_json::json!({"root":root,"home":home,"id":id,"account":account,"savedAt":event.saved_at});
                    std::fs::write(&marker, serde_json::to_vec(&manifest).unwrap()).unwrap(); std::process::exit(86);
                }
                Ok(())
            })));
            let _ = config_file::save(&f.root, &f.home, request);
            panic!("child must exit after successful business readback");
        }
        file_objects::test_credentials::install();
        for drift in [false, true] {
            let manifest_dir = tempfile::tempdir().unwrap();
            let marker = manifest_dir.path().join("manifest.json");
            let status = std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact", "modules::model_config::content_versions::file_saves::tests::post04_review_crash_readback_freezes_credentials_across_real_process_exit", "--nocapture", "--test-threads=1"]).env(CHILD, &marker).status().unwrap();
            assert_eq!(status.code(), Some(86));
            let manifest: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&marker).unwrap()).unwrap();
            let root = PathBuf::from(manifest["root"].as_str().unwrap());
            let home = PathBuf::from(manifest["home"].as_str().unwrap());
            assert!(root.starts_with(std::env::temp_dir()));
            assert!(home.starts_with(std::env::temp_dir()));
            let journal = std::fs::read(root.join(JOURNAL)).unwrap();
            assert!(!String::from_utf8_lossy(&journal).contains("synthetic-original-K1"));
            keyring::Entry::new(
                "opencodex.provider-api-key.v1",
                manifest["account"].as_str().unwrap(),
            )
            .unwrap()
            .set_password(if drift {
                "synthetic-future-K2"
            } else {
                "synthetic-original-K1"
            })
            .unwrap();
            let before = std::fs::read(home.join("config.json")).unwrap();
            let snapshot = config_file::snapshot(&root, &home).unwrap();
            let scope = Scope {
                channel_ids: vec![manifest["id"].as_str().unwrap().into()],
                include_keys: true,
                preferences: false,
                extensions: false,
                upstream_root: Some(home.clone()),
                ..Default::default()
            };
            let fact = config_remote::source_fact(&root, &scope).unwrap();
            if drift {
                assert!(fact.saved_at.is_none(), "later K2 must remain unknown");
            } else {
                assert_eq!(
                    fact.saved_at
                        .as_deref()
                        .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
                    Some(
                        chrono::DateTime::parse_from_rfc3339(manifest["savedAt"].as_str().unwrap())
                            .unwrap()
                    )
                );
            }
            assert!(pending(&root).unwrap().is_none());
            assert_eq!(std::fs::read(home.join("config.json")).unwrap(), before);
            config_file::save(
                &root,
                &home,
                SaveRequest {
                    expected_file: snapshot.policy_digest,
                    policy: snapshot.policy,
                    credentials: vec![],
                    scope: Some(SaveScope {
                        channel_id: manifest["id"].as_str().unwrap().into(),
                    }),
                },
            )
            .unwrap();
            assert_eq!(
                config_remote::source_fact(&root, &scope).unwrap().saved_at,
                fact.saved_at
            );
            let owner = root.parent().unwrap();
            assert_eq!(home.parent(), Some(owner));
            std::fs::remove_dir_all(owner).unwrap();
        }
    }
}
