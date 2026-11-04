//! Real temporary stores and the production immutable/conditional WebDAV path.
use super::super::{content_versions, remote::tests::Memory, sync_state};
use super::*;
use crate::modules::preferences::{self, Preferences, PreferencesStore};
struct Device {
    data: tempfile::TempDir,
    config_home: tempfile::TempDir,
}
impl Device {
    fn new() -> Self {
        file_objects::test_credentials::install();
        let d = Self {
            data: tempfile::tempdir().unwrap(),
            config_home: tempfile::tempdir().unwrap(),
        };
        std::fs::write(
            d.config_home.path().join("config.json"),
            br#"{"providers":{}}"#,
        )
        .unwrap();
        save_scope(d.root(), d.scope()).unwrap();
        d
    }
    fn root(&self) -> &Path {
        self.data.path()
    }
    fn scope(&self) -> Scope {
        Scope {
            preferences: true,
            extensions: false,
            upstream_root: Some(self.config_home.path().into()),
            ..Default::default()
        }
    }
    fn preferences(&self) -> Preferences {
        PreferencesStore::new(self.root()).load().unwrap()
    }
    // Actual save/read-back with a deterministic historical clock. The separate
    // lifecycle test below exercises the production now() hook.
    fn save_at(&self, scale: i32, time: &str) {
        let before = capture_save(self.root(), &self.scope()).unwrap();
        let value = Preferences {
            interface_scale: scale,
            ..self.preferences()
        };
        preferences::save_preferences(&PreferencesStore::new(self.root()).path(), &value).unwrap();
        assert_eq!(self.preferences(), value);
        let after = capture_save(self.root(), &self.scope()).unwrap();
        content_versions::record_at(self.root(), &before, &after, time).unwrap();
    }
    fn fact(&self) -> file_objects::ContentFact {
        source_fact(self.root(), &self.scope()).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        std::fs::read(PreferencesStore::new(self.root()).path()).unwrap()
    }
}
fn config() -> WebDavConfig {
    WebDavConfig {
        base_url: "https://fixture.invalid".into(),
        remote_path: "isolated".into(),
        username: "synthetic".into(),
        password: "synthetic-transport-only".into(),
    }
}
async fn check(d: &Device, remote: &Memory, binding: &str) -> sync_state::Check {
    check_source_with_password(d.root(), remote, &config(), binding, &d.scope(), None, None)
        .await
        .unwrap()
}
async fn upload(d: &Device, remote: &Memory) -> UploadView {
    assert_eq!(check(d, remote, "fixture").await.source, "local");
    let p = prepare(
        d.root(),
        remote,
        &config(),
        "fixture",
        d.scope(),
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    publish(d.root(), remote, &config(), &p.id, "fixture", || Ok(()))
        .await
        .unwrap()
}
async fn download(d: &Device, remote: &Memory) -> IncomingPlan {
    download_sync(
        d.root(),
        remote,
        &config(),
        None,
        None,
        d.config_home.path(),
        "fixture",
        || Ok(()),
    )
    .await
    .unwrap()
}
fn same_instant(a: Option<&str>, b: Option<&str>) -> bool {
    a.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        == b.and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
}
#[tokio::test]
async fn post021_version_observation_reads_existing_snapshot_without_business_writes() {
    let cloud = Memory::default();
    let empty = observe_version(&cloud, &config(), None, None)
        .await
        .unwrap();
    assert!(!empty.remote_exists);
    assert!(cloud.writes().is_empty());
    let legacy = Device::new();
    preferences::save_preferences(
        &PreferencesStore::new(legacy.root()).path(),
        &Preferences {
            interface_scale: 125,
            ..Default::default()
        },
    )
    .unwrap();
    upload(&legacy, &cloud).await;
    let before = legacy.bytes();
    let writes = cloud.writes();
    let unknown = observe_version(&cloud, &config(), None, None)
        .await
        .unwrap();
    assert!(unknown.remote_exists && unknown.remote_revision.is_some());
    assert!(
        unknown.remote_saved_at.is_none(),
        "upload time must not certify unknown legacy content"
    );
    assert_eq!(before, legacy.bytes());
    assert_eq!(writes, cloud.writes());
    if let Some(out) = std::env::var_os("OCXD_POST021_FIXTURE_DIR") {
        cloud.write_synthetic_fixture(&Path::new(&out).join("unknown"));
    }
    legacy.save_at(150, "2026-11-04T04:00:00Z");
    let known_cloud = Memory::default();
    upload(&legacy, &known_cloud).await;
    let known = observe_version(&known_cloud, &config(), None, None)
        .await
        .unwrap();
    if let Some(out) = std::env::var_os("OCXD_POST021_FIXTURE_DIR") {
        known_cloud.write_synthetic_fixture(&Path::new(&out).join("known"));
    }
    assert_ne!(unknown.remote_revision, known.remote_revision);
    assert!(same_instant(
        known.remote_saved_at.as_deref(),
        Some("2026-11-04T04:00:00Z")
    ));
}

#[tokio::test]
async fn post021_first_empty_remote_catalog_selects_local_objects_once() {
    file_objects::test_credentials::install();
    let root = tempfile::tempdir().unwrap();
    let upstream = tempfile::tempdir().unwrap();
    let home = tempfile::tempdir().unwrap();
    std::fs::write(upstream.path().join("config.json"), br#"{"providers":{"local":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","models":["one"],"selectedModels":["one"]}}}"#).unwrap();
    let cloud = Memory::default();
    let first = open_scope(
        root.path(),
        &cloud,
        &config(),
        "fixture",
        upstream.path(),
        home.path(),
        None,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    assert_eq!(first.channel_ids.len(), 1);
    assert!(!first.template_ids.is_empty());
    let mut chosen = first;
    chosen.channel_ids.clear();
    save_scope(root.path(), chosen.clone()).unwrap();
    let next = open_scope(
        root.path(),
        &cloud,
        &config(),
        "fixture",
        upstream.path(),
        home.path(),
        None,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    assert_eq!(
        chosen, next,
        "an explicit empty choice is not a new first run"
    );
    assert!(cloud.writes().is_empty());
}
#[test]
fn actual_preference_save_readback_noop_touch_copy_and_legacy_are_distinct() {
    let d = Device::new();
    assert!(d.fact().empty);
    let legacy = Preferences {
        interface_scale: 125,
        ..Default::default()
    };
    preferences::save_preferences(&PreferencesStore::new(d.root()).path(), &legacy).unwrap();
    assert!(d.fact().saved_at.is_none());
    PreferencesStore::new(d.root()).save(&legacy).unwrap();
    assert!(
        d.fact().saved_at.is_none(),
        "no-op must not certify discovered legacy data"
    );
    let start = chrono::Utc::now();
    let saved = Preferences {
        interface_scale: 150,
        ..legacy
    };
    PreferencesStore::new(d.root()).save(&saved).unwrap();
    let original = d.fact();
    let instant =
        chrono::DateTime::parse_from_rfc3339(original.saved_at.as_ref().unwrap()).unwrap();
    assert!(instant >= start && instant <= chrono::Utc::now());
    assert_eq!(d.preferences(), saved);
    let f = std::fs::OpenOptions::new()
        .write(true)
        .open(PreferencesStore::new(d.root()).path())
        .unwrap();
    f.set_times(std::fs::FileTimes::new().set_modified(std::time::SystemTime::now()))
        .unwrap();
    PreferencesStore::new(d.root()).save(&saved).unwrap();
    assert_eq!(d.fact(), original);
    let copy = Device::new();
    std::fs::copy(
        PreferencesStore::new(d.root()).path(),
        PreferencesStore::new(copy.root()).path(),
    )
    .unwrap();
    assert_eq!(copy.fact().content_digest, original.content_digest);
    assert!(copy.fact().saved_at.is_none());
    assert_eq!(source_fact(d.root(), &d.scope()).unwrap(), original);
}
#[tokio::test]
async fn no_baseline_and_both_modified_choose_saved_time_in_both_directions_and_inherit() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-10-30T00:00:00Z");
    upload(&a, &cloud).await;
    let b = Device::new();
    b.save_at(150, "2026-11-04T00:00:00Z");
    let c = check(&b, &cloud, "new-connection-no-baseline").await;
    assert_eq!(c.source, "local");
    assert_eq!(c.reason, "local_saved_later");
    assert!(same_instant(
        c.local_saved_at.as_deref(),
        Some("2026-11-04T00:00:00Z")
    ));
    upload(&b, &cloud).await;
    let older = Device::new();
    older.save_at(100, "2026-10-30T00:00:00Z");
    assert_eq!(check(&older, &cloud, "fixture").await.source, "remote");
    let before = older.bytes();
    let incoming = download(&older, &cloud).await;
    assert_eq!(
        older.bytes(),
        before,
        "preview/cancel never changes business files"
    );
    sync_state::verify_pending(older.root(), &incoming.id, "fixture", &cloud, &config())
        .await
        .unwrap();
    file_objects::accept_extended(older.root(), &incoming.id, false, false, false, None).unwrap();
    assert_eq!(older.preferences().interface_scale, 150);
    assert!(same_instant(
        older.fact().saved_at.as_deref(),
        b.fact().saved_at.as_deref()
    ));
    assert_eq!(
        check(&older, &cloud, "different-binding").await.source,
        "unchanged"
    );
    assert_eq!(
        prepare(
            older.root(),
            &cloud,
            &config(),
            "different-binding",
            older.scope(),
            None,
            || Ok(())
        )
        .await
        .unwrap_err()
        .code,
        "sync_unchanged"
    );
    older.save_at(175, "2026-11-05T00:00:00Z");
    assert_eq!(check(&older, &cloud, "fixture").await.source, "local");
    b.save_at(200, "2026-11-06T00:00:00Z");
    upload(&b, &cloud).await;
    assert_eq!(check(&older, &cloud, "fixture").await.source, "remote");
}
#[tokio::test]
async fn unknown_bad_same_instant_or_unbound_versions_refuse_both_write_directions() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-11-04T00:00:00Z");
    upload(&a, &cloud).await;
    for mode in [
        "legacy",
        "bad-time",
        "wrong-digest",
        "clock",
        "same-instant",
    ] {
        let b = Device::new();
        if mode == "legacy" {
            preferences::save_preferences(
                &PreferencesStore::new(b.root()).path(),
                &Preferences {
                    interface_scale: 150,
                    ..Default::default()
                },
            )
            .unwrap();
        } else {
            b.save_at(150, "2026-11-04T08:00:00+08:00");
            let p = b.root().join("manager-state/saved-content-versions.json");
            let mut records: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
            for record in records.as_object_mut().unwrap().values_mut() {
                match mode {
                    "bad-time" => record["savedAt"] = serde_json::json!("invalid-time"),
                    "wrong-digest" => record["contentDigest"] = serde_json::json!("stale-digest"),
                    "clock" => record["timeKind"] = serde_json::json!("clockUnreliable"),
                    _ => {}
                }
            }
            std::fs::write(p, serde_json::to_vec(&records).unwrap()).unwrap();
        }
        let before = b.bytes();
        let writes = cloud.writes();
        assert_eq!(
            check(&b, &cloud, "fixture").await.source,
            "undetermined",
            "{mode}"
        );
        assert_eq!(
            prepare(
                b.root(),
                &cloud,
                &config(),
                "fixture",
                b.scope(),
                None,
                || Ok(())
            )
            .await
            .unwrap_err()
            .code,
            "sync_undetermined",
            "{mode}"
        );
        assert_eq!(
            download_sync(
                b.root(),
                &cloud,
                &config(),
                None,
                None,
                b.config_home.path(),
                "fixture",
                || Ok(())
            )
            .await
            .unwrap_err()
            .code,
            "sync_undetermined",
            "{mode}"
        );
        assert_eq!(b.bytes(), before);
        assert_eq!(cloud.writes(), writes);
    }
}
#[tokio::test]
async fn check_then_prepare_cannot_rebase_onto_a_new_cloud_pointer_even_if_local_still_newer() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-10-30T00:00:00Z");
    upload(&a, &cloud).await;
    let b = Device::new();
    b.save_at(150, "2026-11-04T00:00:00Z");
    assert_eq!(check(&b, &cloud, "fixture").await.source, "local");
    a.save_at(175, "2026-11-02T00:00:00Z");
    upload(&a, &cloud).await;
    let writes = cloud.writes();
    assert_eq!(
        prepare(
            b.root(),
            &cloud,
            &config(),
            "fixture",
            b.scope(),
            None,
            || Ok(())
        )
        .await
        .unwrap_err()
        .code,
        "sync_preview_changed"
    );
    assert_eq!(cloud.writes(), writes);
    assert_eq!(check(&b, &cloud, "fixture").await.source, "local");
    a.save_at(200, "2026-11-06T00:00:00Z");
    upload(&a, &cloud).await;
    assert_eq!(
        prepare(
            b.root(),
            &cloud,
            &config(),
            "fixture",
            b.scope(),
            None,
            || Ok(())
        )
        .await
        .unwrap_err()
        .code,
        "sync_preview_changed"
    );
    assert_eq!(check(&b, &cloud, "fixture").await.source, "remote");
}
#[tokio::test]
async fn confirmation_rechecks_local_versions_remote_pointer_and_scope_without_business_writes() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-10-30T00:00:00Z");
    upload(&a, &cloud).await;
    let b = Device::new();
    b.save_at(150, "2026-11-04T00:00:00Z");
    check(&b, &cloud, "fixture").await;
    let plan = prepare(
        b.root(),
        &cloud,
        &config(),
        "fixture",
        b.scope(),
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    b.save_at(175, "2026-11-05T00:00:00Z");
    let writes = cloud.writes();
    assert_eq!(
        publish(b.root(), &cloud, &config(), &plan.id, "fixture", || Ok(()))
            .await
            .unwrap_err()
            .code,
        "sync_source_changed"
    );
    assert_eq!(cloud.writes(), writes);
    check(&b, &cloud, "fixture").await;
    let plan = prepare(
        b.root(),
        &cloud,
        &config(),
        "fixture",
        b.scope(),
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    let versions_path = b.root().join("manager-state/saved-content-versions.json");
    let mut versions: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&versions_path).unwrap()).unwrap();
    for version in versions.as_object_mut().unwrap().values_mut() {
        version["revision"] = serde_json::json!(version["revision"].as_u64().unwrap() + 1);
    }
    std::fs::write(versions_path, serde_json::to_vec(&versions).unwrap()).unwrap();
    let writes = cloud.writes();
    assert_eq!(
        publish(b.root(), &cloud, &config(), &plan.id, "fixture", || Ok(()))
            .await
            .unwrap_err()
            .code,
        "sync_source_changed",
        "metadata-only save version drift is rechecked too"
    );
    assert_eq!(cloud.writes(), writes);
    check(&b, &cloud, "fixture").await;
    let plan = prepare(
        b.root(),
        &cloud,
        &config(),
        "fixture",
        b.scope(),
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    a.save_at(200, "2026-11-06T00:00:00Z");
    upload(&a, &cloud).await;
    assert!(
        publish(b.root(), &cloud, &config(), &plan.id, "fixture", || Ok(()))
            .await
            .is_err()
    );
    check(&b, &cloud, "fixture").await;
    let incoming = download(&b, &cloud).await;
    b.save_at(150, "2026-11-07T00:00:00Z");
    let before = b.bytes();
    assert_eq!(
        sync_state::verify_pending(b.root(), &incoming.id, "fixture", &cloud, &config())
            .await
            .unwrap_err()
            .code,
        "sync_source_changed"
    );
    assert_eq!(b.bytes(), before);
    let c = Device::new();
    check(&c, &cloud, "fixture").await;
    let incoming = download(&c, &cloud).await;
    save_scope(
        c.root(),
        Scope {
            preferences: true,
            extensions: true,
            upstream_root: Some(c.config_home.path().into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        sync_state::verify_pending(c.root(), &incoming.id, "fixture", &cloud, &config())
            .await
            .unwrap_err()
            .code,
        "sync_source_changed"
    );
    assert!(!PreferencesStore::new(c.root()).path().exists());
}

#[tokio::test]
async fn partial_upload_and_download_preserve_outside_content_and_its_save_receipts() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-10-30T00:00:00Z");
    upload(&a, &cloud).await;
    let template = super::tests::create(a.root(), "outside-preferences");
    let template_scope = Scope {
        template_ids: vec![template.clone()],
        preferences: false,
        extensions: false,
        upstream_root: Some(a.config_home.path().into()),
        ..Default::default()
    };
    let original_template = source_fact(a.root(), &template_scope).unwrap();
    assert!(original_template.saved_at.is_some());
    save_scope(a.root(), template_scope.clone()).unwrap();
    assert_eq!(
        check_source_with_password(
            a.root(),
            &cloud,
            &config(),
            "fixture",
            &template_scope,
            None,
            None
        )
        .await
        .unwrap()
        .source,
        "local"
    );
    let p = prepare(
        a.root(),
        &cloud,
        &config(),
        "fixture",
        template_scope.clone(),
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    publish(a.root(), &cloud, &config(), &p.id, "fixture", || Ok(()))
        .await
        .unwrap();
    let head = remote::pointer_for(remote::CONFIG, &cloud, &config())
        .await
        .unwrap()
        .unwrap();
    let bytes = remote::fetch_for(remote::CONFIG, &cloud, &config(), &head.pointer)
        .await
        .unwrap();
    let cloud_preferences = remote_projection(&bytes, None, &a.scope()).unwrap();
    assert!(same_instant(
        cloud_preferences.saved_at.as_deref(),
        Some("2026-10-30T00:00:00Z")
    ));
    assert_eq!(cloud_preferences.content_digest, a.fact().content_digest);
    let b = Device::new();
    b.save_at(150, "2026-11-04T00:00:00Z");
    upload(&b, &cloud).await;
    let head = remote::pointer_for(remote::CONFIG, &cloud, &config())
        .await
        .unwrap()
        .unwrap();
    let bytes = remote::fetch_for(remote::CONFIG, &cloud, &config(), &head.pointer)
        .await
        .unwrap();
    assert_eq!(
        remote_projection(&bytes, None, &template_scope).unwrap(),
        original_template,
        "outside object versions survive the merge unchanged"
    );
    save_scope(a.root(), a.scope()).unwrap();
    assert_eq!(check(&a, &cloud, "fixture").await.source, "remote");
    let original_library = std::fs::read(a.root().join(LIBRARY_PATH)).unwrap();
    let incoming = download(&a, &cloud).await;
    sync_state::verify_pending(a.root(), &incoming.id, "fixture", &cloud, &config())
        .await
        .unwrap();
    file_objects::accept_extended(a.root(), &incoming.id, false, false, false, None).unwrap();
    assert_eq!(a.preferences().interface_scale, 150);
    assert_eq!(
        std::fs::read(a.root().join(LIBRARY_PATH)).unwrap(),
        original_library
    );
    assert_eq!(
        source_fact(a.root(), &template_scope).unwrap(),
        original_template
    );
    assert_eq!(check(&a, &cloud, "fixture").await.source, "unchanged");
}

#[test]
fn extension_save_noop_and_migration_do_not_relabel_old_content_as_new() {
    use crate::modules::extensions::{projection, ClientId, ClientTarget, ExtensionConfig};
    let d = Device::new();
    let home = tempfile::tempdir().unwrap();
    let targets = [ClientTarget::user_target(
        ClientId::Codex,
        home.path(),
        true,
        true,
    )];
    let mut config = ExtensionConfig::default();
    projection::save_with_config(d.root(), &mut config, &targets).unwrap();
    let scope = Scope {
        preferences: false,
        extensions: true,
        ..d.scope()
    };
    let original = source_fact(d.root(), &scope).unwrap();
    assert!(original.saved_at.is_some());
    projection::save_with_config(d.root(), &mut config, &targets).unwrap();
    assert_eq!(
        source_fact(d.root(), &scope).unwrap(),
        original,
        "revision-only writes are not new content"
    );
    std::fs::remove_file(d.root().join("manager-state/saved-content-versions.json")).unwrap();
    let transaction = projection::ConfigTransaction::acquire(d.root(), &targets).unwrap();
    config.sync_method = "copy".into();
    transaction.save(&mut config).unwrap();
    drop(transaction);
    assert!(
        source_fact(d.root(), &scope).unwrap().saved_at.is_none(),
        "migration must not generate now"
    );
}

#[tokio::test]
async fn a_reintroduced_old_cloud_snapshot_cannot_win_just_because_the_pointer_is_new() {
    let cloud = Memory::default();
    let a = Device::new();
    a.save_at(125, "2026-10-30T00:00:00Z");
    upload(&a, &cloud).await;
    let old = remote::pointer_for(remote::CONFIG, &cloud, &config())
        .await
        .unwrap()
        .unwrap();
    let bytes = remote::fetch_for(remote::CONFIG, &cloud, &config(), &old.pointer)
        .await
        .unwrap();
    let b = Device::new();
    b.save_at(150, "2026-11-04T00:00:00Z");
    upload(&b, &cloud).await;
    let baseline = remote::pointer_for(remote::CONFIG, &cloud, &config())
        .await
        .unwrap()
        .unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    remote::publish_blob(
        remote::CONFIG,
        &cloud,
        &config(),
        &id,
        Some(&baseline),
        bytes,
        false,
        |_| Ok(()),
        || Ok(()),
    )
    .await
    .unwrap();
    assert_eq!(check(&b, &cloud, "fixture").await.source, "local");
    let incoming = download_sync(
        b.root(),
        &cloud,
        &config(),
        None,
        None,
        b.config_home.path(),
        "fixture",
        || Ok(()),
    )
    .await
    .unwrap_err();
    assert_eq!(incoming.code, "sync_direction_changed");
    assert_eq!(b.preferences().interface_scale, 150);
}

#[test]
fn post04_review_preferences_and_templates_replay_receipts_after_real_version_lock_contention() {
    use crate::infrastructure::locking::TargetFileLock;
    let instant = |value: &str| chrono::DateTime::parse_from_rfc3339(value).unwrap();
    let receipt_time = |root: &Path, owner: &str| {
        let receipt: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                root.join("manager-state/content-save-events")
                    .join(format!("{owner}.json")),
            )
            .unwrap(),
        )
        .unwrap();
        receipt["savedAt"].as_str().unwrap().to_owned()
    };
    let d = Device::new();
    let guard =
        TargetFileLock::lock(&d.root().join("manager-state/saved-content-versions.json")).unwrap();
    let desired = Preferences {
        interface_scale: 150,
        ..Default::default()
    };
    assert_eq!(
        PreferencesStore::new(d.root()).save(&desired).unwrap_err(),
        preferences::PreferencesError::VersionPending
    );
    assert_eq!(
        preferences::load_preferences(&PreferencesStore::new(d.root()).path()).unwrap(),
        desired
    );
    let original_time = receipt_time(d.root(), "preferences");
    drop(guard);
    PreferencesStore::new(d.root()).load().unwrap();
    assert_eq!(
        instant(d.fact().saved_at.as_ref().unwrap()),
        instant(&original_time)
    );
    let versions =
        std::fs::read(d.root().join("manager-state/saved-content-versions.json")).unwrap();
    PreferencesStore::new(d.root()).save(&desired).unwrap();
    assert_eq!(
        std::fs::read(d.root().join("manager-state/saved-content-versions.json")).unwrap(),
        versions
    );
    let root = tempfile::tempdir().unwrap();
    let store = ModelStore::new(root.path());
    let snapshot = store.load_library().unwrap();
    let guard = TargetFileLock::lock(
        &root
            .path()
            .join("manager-state/saved-content-versions.json"),
    )
    .unwrap();
    let mut content = TemplateContent::default();
    content.matching.include.push(Pattern {
        kind: MatchKind::Prefix,
        value: "locked-template-unique".into(),
    });
    let error = store
        .save(SaveTemplate {
            expected_library: snapshot.library_digest,
            template_id: None,
            base_revision: None,
            label: "locked-template".into(),
            content: content.clone(),
        })
        .unwrap_err();
    assert_eq!(error.code, "saved_content_version_unconfirmed");
    let original_time = receipt_time(root.path(), "templates");
    let written: Library =
        serde_json::from_slice(&std::fs::read(root.path().join(LIBRARY_PATH)).unwrap()).unwrap();
    assert!(written
        .templates
        .iter()
        .any(|t| t.label == "locked-template"));
    drop(guard);
    let (library, digest) = store.library_only().unwrap();
    let template = library
        .templates
        .iter()
        .find(|t| t.label == "locked-template")
        .unwrap();
    let state = file_objects::template_save_state(&Library {
        schema_version: library.schema_version,
        templates: vec![template.clone()],
        submission_receipts: Default::default(),
    })
    .unwrap();
    assert!(content_versions::observe(root.path(), &state)
        .unwrap()
        .iter()
        .all(|v| v.instant() == Some(instant(&original_time))));
    let versions = std::fs::read(
        root.path()
            .join("manager-state/saved-content-versions.json"),
    )
    .unwrap();
    let no_op = store
        .save(SaveTemplate {
            expected_library: digest,
            template_id: Some(template.id.clone()),
            base_revision: Some(template.revisions[0].id.clone()),
            label: template.label.clone(),
            content,
        })
        .unwrap();
    assert!(!no_op.changed);
    assert_eq!(
        std::fs::read(
            root.path()
                .join("manager-state/saved-content-versions.json")
        )
        .unwrap(),
        versions
    );
}
#[test]
fn post04_review_extension_receipt_replays_after_version_contention_and_noop_keeps_time() {
    use crate::{
        infrastructure::locking::TargetFileLock,
        modules::extensions::{projection, ClientId, ClientTarget, ExtensionConfig},
    };
    let d = Device::new();
    let home = tempfile::tempdir().unwrap();
    let targets = [ClientTarget::user_target(
        ClientId::Codex,
        home.path(),
        true,
        true,
    )];
    let mut config = ExtensionConfig::default();
    projection::save_with_config(d.root(), &mut config, &targets).unwrap();
    config.sync_method = "copy".into();
    let guard =
        TargetFileLock::lock(&d.root().join("manager-state/saved-content-versions.json")).unwrap();
    assert_eq!(
        projection::save_with_config(d.root(), &mut config, &targets).unwrap_err(),
        projection::ProjectionError::VersionPending
    );
    let pending: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            d.root()
                .join("manager-state/content-save-events/extensions.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let original_time =
        chrono::DateTime::parse_from_rfc3339(pending["savedAt"].as_str().unwrap()).unwrap();
    let actual: serde_json::Value =
        serde_json::from_slice(&std::fs::read(projection::config_path(d.root())).unwrap()).unwrap();
    assert_eq!(actual["config"]["sync_method"], "copy");
    drop(guard);
    let (mut loaded, _, _) = projection::load(d.root(), &targets).unwrap();
    let scope = Scope {
        preferences: false,
        extensions: true,
        ..d.scope()
    };
    assert_eq!(
        source_fact(d.root(), &scope)
            .unwrap()
            .saved_at
            .as_deref()
            .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
        Some(original_time)
    );
    let versions =
        std::fs::read(d.root().join("manager-state/saved-content-versions.json")).unwrap();
    projection::save_with_config(d.root(), &mut loaded, &targets).unwrap();
    assert_eq!(
        std::fs::read(d.root().join("manager-state/saved-content-versions.json")).unwrap(),
        versions
    );
}

#[tokio::test]
async fn post04_review3_restoring_saved_content_is_a_new_event_not_a_digest_replay() {
    use crate::infrastructure::locking::TargetFileLock;
    let local = Device::new();
    local.save_at(125, "2026-10-30T00:00:00Z");
    let original = local.fact();
    let external = Preferences {
        interface_scale: 150,
        ..local.preferences()
    };
    preferences::save_preferences(&PreferencesStore::new(local.root()).path(), &external).unwrap();
    assert!(
        local.fact().saved_at.is_none(),
        "external B has no saved-content event"
    );
    let desired = Preferences {
        interface_scale: 125,
        ..external
    };
    let lock = TargetFileLock::lock(
        &local
            .root()
            .join("manager-state/saved-content-versions.json"),
    )
    .unwrap();
    assert_eq!(
        PreferencesStore::new(local.root())
            .save(&desired)
            .unwrap_err(),
        preferences::PreferencesError::VersionPending
    );
    assert_eq!(
        preferences::load_preferences(&PreferencesStore::new(local.root()).path()).unwrap(),
        desired
    );
    let journal = local
        .root()
        .join("manager-state/content-save-events/preferences.json");
    let pending = std::fs::read(&journal).unwrap();
    let receipt: serde_json::Value = serde_json::from_slice(&pending).unwrap();
    let time = chrono::DateTime::parse_from_rfc3339(receipt["savedAt"].as_str().unwrap()).unwrap();
    drop(lock);
    PreferencesStore::new(local.root()).load().unwrap();
    let restored = local.fact();
    assert_eq!(restored.content_digest, original.content_digest);
    assert_eq!(
        restored
            .saved_at
            .as_deref()
            .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
        Some(time),
        "new B-to-A save must not inherit the old A timestamp"
    );
    assert!(time > chrono::DateTime::parse_from_rfc3339("2026-11-01T00:00:00Z").unwrap());
    let version_file = local
        .root()
        .join("manager-state/saved-content-versions.json");
    let certified = std::fs::read(&version_file).unwrap();
    let versions: serde_json::Value = serde_json::from_slice(&certified).unwrap();
    let key = file_objects::preferences_save_state(Some(&desired))
        .unwrap()
        .into_keys()
        .next()
        .unwrap();
    assert_eq!(
        versions[&key]["lastEvent"], receipt["id"],
        "certification must bind the durable UUID"
    );
    assert!(receipt["readbackVerified"].as_bool().unwrap());
    std::fs::write(&journal, &pending).unwrap(); // Same durable event's lost reply.
    PreferencesStore::new(local.root()).load().unwrap();
    assert_eq!(
        std::fs::read(&version_file).unwrap(),
        certified,
        "only same-event replay is idempotent"
    );
    PreferencesStore::new(local.root()).save(&desired).unwrap();
    assert_eq!(
        std::fs::read(&version_file).unwrap(),
        certified,
        "ordinary noop keeps the new event time"
    );
    let cloud = Memory::default();
    let other = Device::new();
    other.save_at(175, "2026-11-01T00:00:00Z");
    upload(&other, &cloud).await;
    assert_eq!(
        check(&local, &cloud, "fixture").await.source,
        "local",
        "cloud Nov 1 must not win against the real new local save"
    );
}

#[test]
fn post04_review3_agent_rename_preserves_only_matching_portable_preference_receipts() {
    use super::super::{agent, config_file};
    use crate::{
        infrastructure::{locking::TargetFileLock, runtime_executable::FixedRuntimeExecutable},
        state::ProcessContext,
    };
    for mode in ["agent_only", "portable_changed", "unverified_intent"] {
        let device = Device::new();
        std::fs::write(device.config_home.path().join("config.json"), br#"{"providers":{"selected":{"adapter":"openai-chat","authMode":"key","baseUrl":"https://receipt-fixture.invalid/v1","apiKey":"synthetic-inline-only","models":["fixture-model"]}}}"#).unwrap();
        let snapshot = config_file::snapshot(device.root(), device.config_home.path()).unwrap();
        config_file::record_layout(device.root(), &snapshot.policy);
        let id = snapshot.policy.channels[0].id.clone();
        preferences::save_preferences(
            &PreferencesStore::new(device.root()).path(),
            &Preferences::default(),
        )
        .unwrap();
        agent::save_settings(
            device.root(),
            agent::AgentSettings {
                provider_id: "selected".into(),
                model_id: "fixture-model".into(),
                reasoning_effort: None,
                fast: None,
                channel_id: Some(id.clone()),
                connection_digest: None,
            },
        )
        .unwrap();
        let desired = Preferences {
            interface_scale: 150,
            ..Default::default()
        };
        let lock = TargetFileLock::lock(
            &device
                .root()
                .join("manager-state/saved-content-versions.json"),
        )
        .unwrap();
        assert_eq!(
            PreferencesStore::new(device.root())
                .save(&desired)
                .unwrap_err(),
            preferences::PreferencesError::VersionPending
        );
        let journal = device
            .root()
            .join("manager-state/content-save-events/preferences.json");
        let receipt: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&journal).unwrap()).unwrap();
        let original_time =
            chrono::DateTime::parse_from_rfc3339(receipt["savedAt"].as_str().unwrap()).unwrap();
        let original_bytes = device.bytes();
        drop(lock);
        let current = config_file::snapshot(device.root(), device.config_home.path()).unwrap();
        let mut policy = current.policy;
        let channel = policy.channels.iter_mut().find(|c| c.id == id).unwrap();
        channel.label = "renamed".into();
        channel.connection.provider_id = "renamed".into();
        let context = ProcessContext {
            runtime: FixedRuntimeExecutable::resolved(
                device.config_home.path().join("not-executed"),
            ),
            working_directory: device.config_home.path().into(),
            opencodex_home: device.config_home.path().into(),
        };
        config_file::save_with_context(
            device.root(),
            &context,
            config_file::SaveRequest {
                expected_file: current.policy_digest,
                policy,
                credentials: vec![],
                scope: Some(config_file::SaveScope { channel_id: id }),
            },
        )
        .unwrap();
        assert_ne!(
            device.bytes(),
            original_bytes,
            "real rename rewrites the same preferences file"
        );
        let raw: serde_json::Value = serde_json::from_slice(&device.bytes()).unwrap();
        assert_eq!(
            raw[preferences::MODEL_AGENT_LOCAL_NAMESPACE]["providerId"],
            "renamed"
        );
        if mode == "unverified_intent" {
            let mut intent = receipt.clone();
            intent["readbackVerified"] = serde_json::json!(false);
            std::fs::write(&journal, serde_json::to_vec(&intent).unwrap()).unwrap();
        }
        if mode == "portable_changed" {
            preferences::save_preferences(
                &PreferencesStore::new(device.root()).path(),
                &Preferences {
                    interface_scale: 175,
                    ..desired.clone()
                },
            )
            .unwrap();
        }
        PreferencesStore::new(device.root()).load().unwrap();
        let current = device.fact();
        if mode != "agent_only" {
            assert!(
                current.saved_at.is_none(),
                "later portable changes cannot inherit the earlier receipt time"
            );
            assert_eq!(
                device.preferences().interface_scale,
                if mode == "portable_changed" { 175 } else { 150 }
            );
        } else {
            assert_eq!(
                current
                    .saved_at
                    .as_deref()
                    .map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap()),
                Some(original_time),
                "Agent-only rename must not discard the saved portable preference event"
            );
            assert_eq!(device.preferences().interface_scale, 150);
        }
        assert!(!journal.exists());
    }
}
