//! Real wrapper/capture/receiver/backup/file transaction in temporary D/H.
use super::super::objects_overlay_v2::ChannelSelectionV2;
use super::*;
use serde_json::{json, Value};
use tempfile::TempDir;
const PASSWORD: &str = "synthetic-file-only-passphrase-023";
const SECRET: &str = "synthetic-unselected-key-023";
struct NoCredentials;
impl CaptureCredentialsV2 for NoCredentials {
    fn channel_key(
        &self,
        _: &Path,
        _: &saved_objects_capture_v2::SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        panic!("unselected key access");
    }
    fn authorization(
        &self,
        _: &Path,
        _: &str,
        _: &Value,
        _: &Value,
    ) -> ModelResult<PortableAuthorization> {
        panic!("unselected auth access");
    }
    fn webdav_password(&self, _: &Path, _: &str) -> ModelResult<Zeroizing<String>> {
        panic!("unselected password access");
    }
}
struct Fixture {
    _t: TempDir,
    d: PathBuf,
    h: PathBuf,
}
impl Fixture {
    fn new(provider: bool) -> Self {
        let t = tempfile::Builder::new()
            .prefix("ocxd023-filetransfer-")
            .tempdir_in(crate::modules::test_sandbox::temporary_parent())
            .unwrap();
        let d = t.path().join("D");
        let h = t.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir(&h).unwrap();
        let raw = if provider {
            json!({"providers":{"fixture":{"adapter":"openai-chat","baseUrl":"https://fixture.invalid/v1","authMode":"key","apiKey":SECRET,"models":["vendor/custom","other"],"selectedModels":["vendor/custom","other"],"modelCosts":{"vendor/custom":{"input":1,"futureNested":{"keep":[1,2]}}},"modelPreset":{"mode":"custom"}}},"untouched":{"keep":true}})
        } else {
            json!({"providers":{},"untouched":{"keep":true}})
        };
        std::fs::write(h.join("config.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
        Self { _t: t, d, h }
    }
    fn owner(&self) -> FileTransferOwnerV2 {
        FileTransferOwnerV2::new(self.d.clone(), self.h.clone(), Arc::new(NoCredentials))
            .unwrap()
            .with_recovery_keys(Arc::new(MemoryRecoveryKey))
    }
    fn request(&self) -> CaptureRequestV2 {
        let inv = inventory(&self.d, &self.h).unwrap();
        CaptureRequestV2 {
            selection: SelectionV2 {
                usage: UsageV2::LocalFile,
                channels: inv
                    .policy
                    .channels
                    .iter()
                    .map(|c| ChannelSelectionV2 {
                        portable_channel_id: c.id.clone(),
                        model_ids: None,
                        family_ids: None,
                        include_connection: true,
                        include_key: false,
                        include_authorization: false,
                    })
                    .collect(),
                ..Default::default()
            },
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        }
    }
    fn business(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        let mut out = BTreeMap::new();
        for p in [
            self.h.join("config.json"),
            self.d.join(store_v2::LIBRARY_PATH),
            self.d.join(store_v2::POLICY_PATH),
            self.d.join(store_v2::LEDGER_PATH),
            self.d.join(store_v2::MAPPING_PATH),
            self.d
                .join(super::super::content_versions::SAVED_VERSIONS_PATH),
        ] {
            if let Ok(b) = std::fs::read(&p) {
                out.insert(p, b);
            }
        }
        out
    }
}
fn code<T>(r: ModelResult<T>) -> String {
    match r {
        Err(e) => e.code,
        Ok(_) => panic!("expected refusal"),
    }
}
fn exported(f: &Fixture, encrypted: bool) -> (Vec<u8>, SelectionV2) {
    let r = f.request();
    let scope = r.selection.clone();
    let mut o = f.owner();
    let p = o
        .prepare_export(r, encrypted.then(|| Zeroizing::new(PASSWORD.into())))
        .unwrap();
    (o.bytes(&p.id).unwrap().to_vec(), scope)
}
fn new_binding(scope: &SelectionV2) -> ImportBindingsV2 {
    ImportBindingsV2 {
        channels: scope
            .channels
            .iter()
            .map(|c| ChannelBindingChoiceV2::New {
                portable_channel_id: c.portable_channel_id.clone(),
            })
            .collect(),
        ..Default::default()
    }
}
#[test]
fn saved_export_native_bytes_encryption_cancel_and_unknown_output_freeze() {
    let f = Fixture::new(true);
    let before = f.business();
    let mut o = f.owner();
    let p = o
        .prepare_export(f.request(), Some(Zeroizing::new(PASSWORD.into())))
        .unwrap();
    assert!(p.encrypted);
    assert_eq!(p.summary.channel_key_count, 0);
    let public = serde_json::to_string(&p).unwrap();
    assert!(!public.contains(SECRET));
    assert!(!public.contains(PASSWORD));
    let b = o.bytes(&p.id).unwrap().to_vec();
    assert!(!String::from_utf8_lossy(&b).contains(SECRET));
    assert!(!o.cancel_export(&p.id).unwrap());
    assert_eq!(
        code(o.finish_export(&p.id, &schema_v2::digest(b"partial"))),
        "file_output_unknown"
    );
    assert_eq!(o.bytes(&p.id).unwrap(), b);
    let decoded = decode(&b, Some(PASSWORD)).unwrap();
    let c = &decoded
        .objects
        .models()
        .payload
        .policy
        .as_ref()
        .unwrap()
        .channels[0];
    assert_eq!(
        c.connection.as_ref().unwrap().provider.as_ref().unwrap()["modelCosts"]["vendor/custom"]
            ["futureNested"],
        json!({"keep":[1,2]})
    );
    let output = f._t.path().join("explicit-user-output.ocxdconf");
    std::fs::write(&output, &b).unwrap();
    let r = o
        .finish_export(&p.id, &schema_v2::digest(&std::fs::read(output).unwrap()))
        .unwrap();
    assert_eq!(r.operation_id, p.id);
    assert_eq!(f.business(), before);
    let p = o.prepare_export(f.request(), None).unwrap();
    assert!(o.cancel_export(&p.id).unwrap());
    assert_eq!(code(o.bytes(&p.id)), "file_operation_missing");
}
#[test]
fn preview_reselect_no_business_write_confirm_real_receiver_and_no_reapply_after_edits() {
    let source = Fixture::new(true);
    let (bytes, scope) = exported(&source, true);
    let f = Fixture::new(false);
    let before = f.business();
    let mut o = f.owner();
    let p = o
        .prepare_import(
            bytes,
            Some(Zeroizing::new(PASSWORD.into())),
            None,
            Default::default(),
        )
        .unwrap();
    assert!(!p.can_confirm);
    assert_eq!(p.objects.iter().filter(|o| o.kind == "channel").count(), 1);
    assert_eq!(f.business(), before);
    let p = o
        .reselect(
            &p.operation_id,
            &p.plan_digest,
            scope.clone(),
            new_binding(&scope),
        )
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    assert_eq!(f.business(), before);
    let id = p.operation_id.clone();
    let receipt = o.confirm_import(&id, &p.plan_digest, None).unwrap();
    assert_eq!(receipt.operation_id, id);
    let raw: Value =
        serde_json::from_slice(&std::fs::read(f.h.join("config.json")).unwrap()).unwrap();
    assert_eq!(raw["untouched"]["keep"], true);
    assert!(raw["providers"]
        .as_object()
        .unwrap()
        .values()
        .any(|p| p["models"] == json!(["vendor/custom", "other"])));
    for path in [
        store_v2::LIBRARY_PATH,
        store_v2::POLICY_PATH,
        store_v2::LEDGER_PATH,
        store_v2::MAPPING_PATH,
    ] {
        assert!(f.d.join(path).exists(), "{path}");
    }
    assert!(
        !crate::modules::backup::list_action_records(&f.d.join("backups"), "sync-overwrite")
            .unwrap()
            .is_empty()
    );
    std::fs::write(f.h.join("config.json"), b"later user edit").unwrap();
    assert_eq!(
        o.confirm_import(&id, &p.plan_digest, None).unwrap(),
        receipt
    );
    // Re-enter the actual durable participant (not just owner's cached receipt).
    let op = &o.imports[&id];
    let app = op.application.as_ref().unwrap();
    app.host
        .apply_once(
            &id,
            &app.payload,
            &codec::supported_schema(),
            &app.scope,
            &app.target,
            true,
        )
        .unwrap();
    assert_eq!(
        std::fs::read(f.h.join("config.json")).unwrap(),
        b"later user edit"
    );
}
#[test]
fn stale_target_fails_without_overwrite_and_same_id_bytes_are_retained() {
    let source = Fixture::new(true);
    let (bytes, scope) = exported(&source, false);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(
            bytes.clone(),
            None,
            Some(scope.clone()),
            new_binding(&scope),
        )
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    let changed = br#"{"providers":{},"userEdit":true}"#;
    std::fs::write(f.h.join("config.json"), changed).unwrap();
    assert!(o
        .confirm_import(&p.operation_id, &p.plan_digest, None)
        .is_err());
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), changed);
    assert_eq!(&*o.imports[&p.operation_id].original, &bytes);
    assert!(!o.cancel_import(&p.operation_id).unwrap());
    assert_eq!(
        code(o.reselect(
            &p.operation_id,
            &p.plan_digest,
            scope.clone(),
            new_binding(&scope)
        )),
        "file_operation_frozen"
    );
}
#[test]
fn legacy_unknown_digest_and_password_fail_closed() {
    let f = Fixture::new(false);
    let mut o = f.owner();
    assert_eq!(
        code(o.prepare_import(
            br#"{"format":"OCXD_MODEL_PACKAGE","formatVersion":5}"#.to_vec(),
            None,
            None,
            Default::default()
        )),
        "file_legacy_format_unsupported"
    );
    let s = Fixture::new(true);
    let (bytes, _) = exported(&s, true);
    assert!(o
        .prepare_import(
            bytes.clone(),
            Some(Zeroizing::new("wrong".into())),
            None,
            Default::default()
        )
        .is_err());
    let mut value: Value = serde_json::from_slice(&bytes).unwrap();
    value["unknown"] = json!(true);
    assert!(o
        .prepare_import(
            serde_json::to_vec(&value).unwrap(),
            Some(Zeroizing::new(PASSWORD.into())),
            None,
            Default::default()
        )
        .is_err());
    value.as_object_mut().unwrap().remove("unknown");
    value["manifest"]["documentDigest"] = json!(schema_v2::digest(b"tampered"));
    assert!(o
        .prepare_import(
            serde_json::to_vec(&value).unwrap(),
            Some(Zeroizing::new(PASSWORD.into())),
            None,
            Default::default()
        )
        .is_err());
}
#[test]
fn mapping_never_accepts_provider_from_ui_and_cancel_never_applies() {
    assert!(serde_json::from_value::<ChannelBindingChoiceV2>(json!({"kind":"new","portableChannelId":"11111111-1111-4111-8111-111111111111","providerId":"openai"})).is_err());
    let s = Fixture::new(true);
    let (bytes, scope) = exported(&s, false);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let before = f.business();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    assert_eq!(
        code(o.confirm_import(&p.operation_id, &schema_v2::digest(b"stale"), None)),
        "file_plan_changed"
    );
    assert!(o.cancel_import(&p.operation_id).unwrap());
    assert_eq!(f.business(), before);
}

const DAV: &str = "11111111-1111-4111-8111-111111111111";
const DAV_PASSWORD: &str = "synthetic-candidate-password-023";
fn password_file(f: &Fixture) -> (Vec<u8>, SelectionV2) {
    let (bytes, _) = exported(f, false);
    let mut captured = decode(&bytes, None).unwrap().objects.into_captured();
    captured.manager.webdav = vec![portable_manager::Endpoint {
        id: DAV.into(),
        url: "https://dav.invalid".into(),
        remote_path: "/configs".into(),
        username: "fixture-user".into(),
        conflict_policy: "ask".into(),
    }];
    captured
        .webdav_passwords
        .insert(DAV.into(), Zeroizing::new(DAV_PASSWORD.into()))
        .unwrap();
    let objects = SavedObjects::capture(captured, Some(&Auth)).unwrap();
    let bytes = codec::encode(
        &objects,
        "22222222-2222-4222-8222-222222222222",
        "2026-11-07T00:00:00Z",
        None,
        Some(&Auth),
    )
    .unwrap()
    .into_bytes()
    .to_vec();
    let scope = SelectionV2 {
        usage: UsageV2::LocalFile,
        manager: objects_overlay_v2::ManagerSelectionV2 {
            webdav_ids: vec![DAV.into()],
            webdav_password_ids: vec![DAV.into()],
            ..Default::default()
        },
        ..Default::default()
    };
    (bytes, scope)
}
#[test]
fn local_file_password_installs_only_in_d_as_inactive_candidate_and_private_receipt() {
    use crate::infrastructure::{credentials, keychain::WEBDAV_CREDENTIAL_PURPOSE};
    let source = Fixture::new(true);
    let (bytes, scope) = password_file(&source);
    let f = Fixture::new(false);
    let other = Fixture::new(false);
    let mut o = f.owner();
    let before = f.business();
    let p = o
        .prepare_import(bytes, None, Some(scope), Default::default())
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    assert_eq!(f.business(), before);
    let public = serde_json::to_string(&p).unwrap();
    assert!(!public.contains(DAV_PASSWORD));
    assert!(!public.contains("cred_"));
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    let candidates = portable_manager::load_candidates(&f.d).unwrap();
    assert_eq!(candidates.len(), 1);
    let reference = candidates[0].credential_ref.as_ref().unwrap();
    assert_eq!(
        credentials::load_optional_at(&f.d, WEBDAV_CREDENTIAL_PURPOSE, reference)
            .unwrap()
            .unwrap()
            .as_str(),
        DAV_PASSWORD
    );
    assert!(
        credentials::load_optional_at(&other.d, WEBDAV_CREDENTIAL_PURPOSE, reference)
            .unwrap()
            .is_none()
    );
    assert!(crate::modules::sync::config::SyncConfigStore::new(&f.d)
        .load()
        .unwrap()
        .endpoints
        .is_empty());
    let journal = std::fs::read(password_path(&f.d, &p.operation_id).unwrap()).unwrap();
    assert!(!String::from_utf8_lossy(&journal).contains(DAV_PASSWORD));
    assert!(!String::from_utf8_lossy(&journal).contains(PASSWORD));
    assert!(SystemWebDavPasswordInstallerV2
        .receipted(&f.d, &p.operation_id, &p.plan_digest)
        .unwrap());
}
struct CutPasswordReceipt(std::sync::atomic::AtomicBool);
impl RootScopedWebDavPasswordInstallerV2 for CutPasswordReceipt {
    fn prepare_and_protect(
        &self,
        d: &Path,
        id: &str,
        plan: &str,
        o: &SavedObjects,
        b: bool,
    ) -> ModelResult<()> {
        SystemWebDavPasswordInstallerV2.prepare_and_protect(d, id, plan, o, b)
    }
    fn apply_once(&self, d: &Path, id: &str, plan: &str, o: &SavedObjects) -> ModelResult<()> {
        SystemWebDavPasswordInstallerV2.apply_once(d, id, plan, o)?;
        if self.0.swap(false, std::sync::atomic::Ordering::SeqCst) {
            return Err(err("test_cut_after_password_receipt"));
        }
        Ok(())
    }
    fn receipted(&self, d: &Path, id: &str, plan: &str) -> ModelResult<bool> {
        SystemWebDavPasswordInstallerV2.receipted(d, id, plan)
    }
}
#[test]
fn password_retry_same_operation_preserves_user_changes_after_durable_participant_receipt() {
    let source = Fixture::new(true);
    let (bytes, scope) = password_file(&source);
    let f = Fixture::new(false);
    let mut o = f
        .owner()
        .with_password_installer(Arc::new(CutPasswordReceipt(
            std::sync::atomic::AtomicBool::new(true),
        )));
    let p = o
        .prepare_import(bytes.clone(), None, Some(scope), Default::default())
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    assert_eq!(
        code(o.confirm_import(&p.operation_id, &p.plan_digest, None)),
        "test_cut_after_password_receipt"
    );
    drop(o);
    let mut o = f.owner();
    let changed = b"later user candidate edit";
    std::fs::write(f.d.join(portable_manager::CANDIDATES), changed).unwrap();
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    assert_eq!(
        std::fs::read(f.d.join(portable_manager::CANDIDATES)).unwrap(),
        changed
    );
    assert_eq!(&*o.imports[&p.operation_id].original, &bytes);
}
#[test]
fn no_password_choice_never_creates_credentials_and_existing_identity_requires_explicit_mapping() {
    let s = Fixture::new(true);
    let (bytes, mut scope) = password_file(&s);
    scope.manager.webdav_password_ids.clear();
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope), Default::default())
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    assert!(portable_manager::load_candidates(&f.d).unwrap()[0]
        .credential_ref
        .is_none());
    assert!(!f.d.join("manager-state/credentials").exists());
    let (bytes, scope) = exported(&s, false);
    let mut o = s.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    assert!(!p.can_confirm);
    assert_eq!(
        p.reason_code.as_deref(),
        Some("file_binding_identity_conflict")
    );
    let choices = ImportBindingsV2 {
        channels: vec![ChannelBindingChoiceV2::Existing {
            portable_channel_id: scope.channels[0].portable_channel_id.clone(),
            local_channel_id: scope.channels[0].portable_channel_id.clone(),
        }],
        ..Default::default()
    };
    let p = o
        .reselect(&p.operation_id, &p.plan_digest, scope, choices)
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
}
#[test]
fn model_subset_from_whole_file_is_scoped_before_receive_without_overwriting_other_models() {
    let s = Fixture::new(true);
    let (bytes, mut scope) = exported(&s, false);
    scope.channels[0].model_ids = Some(vec!["vendor/custom".into()]);
    // New provider receives exactly the selected projection, including public
    // custom maps; neither the other model nor its source attachments expand it.
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    let raw: Value =
        serde_json::from_slice(&std::fs::read(f.h.join("config.json")).unwrap()).unwrap();
    let provider = raw["providers"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap();
    assert_eq!(provider["models"], json!(["vendor/custom"]));
}

#[test]
fn original_provider_public_field_domain_survives_saved_full_export() {
    use super::super::provider_fields;
    let f = Fixture::new(true);
    let mut raw: Value =
        serde_json::from_slice(&std::fs::read(f.h.join("config.json")).unwrap()).unwrap();
    let provider = raw["providers"]["fixture"].as_object_mut().unwrap();
    for field in provider_fields::PUBLIC {
        provider.entry((*field).to_owned()).or_insert_with(|| {
            if field.starts_with("model") {
                json!({})
            } else if field.ends_with("Models") || *field == "retainModels" {
                json!([])
            } else {
                Value::Null
            }
        });
    }
    for (key, value) in [
        ("retainModels", json!([])),
        ("noReasoningModels", json!([])),
        ("contextWindow", json!(4096)),
        ("reasoningEfforts", json!(["low", "high"])),
        ("pinnedReasoningEffort", json!("high")),
        ("reasoningEffortMap", json!({})),
        ("supportsServiceTier", json!(false)),
        ("modelReasoningEffortsAuthoritative", json!(false)),
    ] {
        provider.insert(key.into(), value);
    }
    std::fs::write(f.h.join("config.json"), serde_json::to_vec(&raw).unwrap()).unwrap();
    let (bytes, _) = exported(&f, false);
    let decoded = decode(&bytes, None).unwrap();
    let public = decoded
        .objects
        .models()
        .payload
        .policy
        .as_ref()
        .unwrap()
        .channels[0]
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap();
    assert_eq!(
        public,
        &provider_fields::public_from_raw(&raw["providers"]["fixture"]).unwrap()
    );
    for field in provider_fields::PUBLIC {
        assert!(public.get(field).is_some(), "{field}");
    }
}
#[test]
fn skill_and_mcp_contents_review_and_credentials_are_separate_import_scopes() {
    let source = Fixture::new(true);
    let (bytes, _) = exported(&source, false);
    let mut captured = decode(&bytes, None).unwrap().objects.into_captured();
    let skill_dir = source._t.path().join("incoming-skill");
    std::fs::create_dir(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        b"# Fixture\nExact incoming instruction.\n",
    )
    .unwrap();
    captured.assets.skills =
        vec![portable_assets::snapshot_skill(&skill_dir, "fixture-skill", false, true).unwrap()];
    captured.assets.mcps=vec![portable_assets::split_mcp(discovery::RawServerDefinition {
        name:"fixture-mcp".into(),value:json!({"command":"fixture-command","args":["--local"],"env":{"TOKEN":"synthetic-mcp-secret"}}),
    },true,true).unwrap()];
    let digest = objects_overlay_v2::mcp_source_digest(&captured.assets.mcps[0]).unwrap();
    let objects = SavedObjects::capture(captured, Some(&Auth)).unwrap();
    let bytes = codec::encode(
        &objects,
        "33333333-3333-4333-8333-333333333333",
        "2026-11-07T00:00:00Z",
        None,
        Some(&Auth),
    )
    .unwrap()
    .into_bytes()
    .to_vec();
    let scope = SelectionV2 {
        usage: UsageV2::LocalFile,
        skill_names: vec!["fixture-skill".into()],
        mcp_sources: vec![objects_overlay_v2::McpSourceSelectionV2 {
            name: "fixture-mcp".into(),
            definition_digest: digest,
        }],
        ..Default::default()
    };
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), Default::default())
        .unwrap();
    assert!(!p.can_confirm);
    assert_eq!(
        p.reason_code.as_deref(),
        Some("capture_assets_review_required")
    );
    let choices = ImportBindingsV2 {
        assets_reviewed: true,
        mcp_clients: BTreeMap::from([("fixture-mcp".into(), "managed".into())]),
        ..Default::default()
    };
    let p = o
        .reselect(&p.operation_id, &p.plan_digest, scope, choices)
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    let public = serde_json::to_string(&p).unwrap();
    assert!(!public.contains("synthetic-mcp-secret"));
    assert!(!public.contains("secretFields"));
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    assert_eq!(
        std::fs::read(discovery::skills_store_dir(&f.d).join("fixture-skill/SKILL.md")).unwrap(),
        b"# Fixture\nExact incoming instruction.\n"
    );
    let mcp = std::fs::read(f.d.join("manager-state/mcp-definitions.json")).unwrap();
    assert!(!String::from_utf8_lossy(&mcp).contains("synthetic-mcp-secret"));
}

#[test]
fn password_storage_failure_resumes_same_refs_and_does_not_reapply_model_files() {
    let source = Fixture::new(true);
    let (bytes, scope) = password_file(&source);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes.clone(), None, Some(scope), Default::default())
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    let blocked = f.d.join("manager-state/credentials");
    std::fs::write(&blocked, b"blocked credential directory").unwrap();
    assert_eq!(
        code(o.confirm_import(&p.operation_id, &p.plan_digest, None)),
        "file_password_store_io"
    );
    let frozen = password_load(&f.d, &p.operation_id).unwrap().unwrap().refs;
    assert!(portable_manager::load_candidates(&f.d).unwrap()[0]
        .credential_ref
        .is_none());
    let later = br#"{"providers":{},"laterUserEdit":true}"#;
    std::fs::write(f.h.join("config.json"), later).unwrap();
    std::fs::remove_file(blocked).unwrap();
    drop(o);
    let mut o = f.owner();
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    assert_eq!(
        password_load(&f.d, &p.operation_id).unwrap().unwrap().refs,
        frozen
    );
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), later);
    assert_eq!(&*o.imports[&p.operation_id].original, &bytes);
}
#[test]
fn real_backup_failure_prevents_business_writes_and_can_retry_original_operation() {
    let source = Fixture::new(true);
    let (bytes, scope) = exported(&source, false);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    std::fs::write(f.d.join("backups"), b"blocked backup directory").unwrap();
    let before = f.business();
    assert!(o
        .confirm_import(&p.operation_id, &p.plan_digest, None)
        .is_err());
    assert_eq!(f.business(), before);
    std::fs::remove_file(f.d.join("backups")).unwrap();
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    assert_ne!(f.business(), before);
}
#[test]
fn oversized_and_duplicate_wrapper_and_old_inner_document_are_rejected_precisely() {
    let f = Fixture::new(false);
    let mut o = f.owner();
    assert_eq!(
        code(o.prepare_import(
            vec![b' '; codec::FILE_LIMIT + 1],
            None,
            None,
            Default::default()
        )),
        "portable_limit"
    );
    let legacy = br#"{"format":"OCXD_CONFIG_OBJECTS","version":1,"models":{}}"#;
    let bytes = crate::modules::sync::portable_file::seal(
        crate::modules::sync::portable_file::Manifest {
            id: "44444444-4444-4444-8444-444444444444".into(),
            created_at: "2026-11-07T00:00:00Z".into(),
            sections: vec!["models".into()],
            secret_objects: 0,
            document_digest: schema_v2::digest(legacy),
        },
        legacy,
        None,
    )
    .unwrap();
    assert_eq!(
        code(o.prepare_import(bytes, None, None, Default::default())),
        "file_legacy_format_unsupported"
    );
    let duplicate=br#"{"format":"OCXDCONF","format":"OCXDCONF","formatVersion":3,"manifest":{},"payload":{}}"#;
    assert_eq!(
        code(o.prepare_import(duplicate.to_vec(), None, None, Default::default())),
        "portable_objects_v2_format"
    );
    assert_eq!(
        code(codec::decode_document(
            &vec![b' '; codec::DOCUMENT_LIMIT + 1],
            Some(&Auth)
        )),
        "portable_limit"
    );
}
#[test]
fn saved_export_does_not_silently_export_unsaved_draft() {
    let f = Fixture::new(true);
    let (before, _) = exported(&f, false);
    let s = config::snapshot(&f.d, &f.h).unwrap();
    let s = config::begin_draft(
        &f.d,
        &f.h,
        config::ReadRequestV2 {
            expected_source: s.source_digest.clone(),
            expected_library: s.library_digest.clone(),
            expected_draft: s.draft.as_ref().map(|d| d.base.clone()),
        },
    )
    .unwrap();
    let mut draft = s.draft.unwrap();
    draft.policy.channels[0].label = "explicit-unsaved-draft-label".into();
    draft.policy.channels[0].models[0]
        .explicit_overrides
        .context_window = super::super::schema::Field::Custom(12345);
    std::fs::write(
        f.d.join(store_v2::DRAFT_PATH),
        serde_json::to_vec(&draft).unwrap(),
    )
    .unwrap();
    let (after, _) = exported(&f, false);
    assert_eq!(
        codec::semantic_digest(&decode(&before, None).unwrap().objects).unwrap(),
        codec::semantic_digest(&decode(&after, None).unwrap().objects).unwrap()
    );
    let mut o = f.owner();
    assert_eq!(
        code(o.prepare_draft_export(
            DraftExportRequestV2 {
                expected: draft.base,
                capture: f.request()
            },
            None
        )),
        "file_draft_capture_not_connected"
    );
}

#[test]
fn exact_template_refs_manual_and_auto_source_intents_survive_real_file_receive() {
    use super::super::schema_v2::{self as v2, ModelBindingV2};
    let source = Fixture::new(true);
    let (bytes, scope) = exported(&source, false);
    let mut captured = decode(&bytes, None).unwrap().objects.into_captured();
    let mut template = v2::seed_library_v2().unwrap().templates.remove(0);
    template.id = "55555555-5555-4555-8555-555555555555".into();
    let mut revision = template.revisions.remove(0);
    revision.id = "66666666-6666-4666-8666-666666666666".into();
    revision.base_revision = None;
    revision.content.condition_overrides.clear();
    revision.content.locked_fields.clear();
    revision.content.common_rules = super::super::schema::ModelRules {
        protocol: super::super::schema::Field::Custom("openai-chat".into()),
        ..Default::default()
    };
    revision.digest = v2::content_digest(&revision.saved_label, &revision.content).unwrap();
    template.default_revision = Some(revision.id.clone());
    template.revisions = vec![revision];
    let r = v2::RefV2 {
        template_id: template.id.clone(),
        revision_id: template.revisions[0].id.clone(),
        digest: template.revisions[0].digest.clone(),
    };
    captured.models.payload.templates = vec![template];
    let c = &mut captured.models.payload.policy.as_mut().unwrap().channels[0];
    c.models[0].binding = Some(ModelBindingV2::Manual {
        reference: r.clone(),
    });
    c.models[1].binding = Some(ModelBindingV2::AutoRead {
        reference: r.clone(),
    });
    c.models[0].explicit_overrides.context_window = super::super::schema::Field::Custom(12345);
    c.models[1].explicit_overrides.protocol = super::super::schema::Field::Custom("google".into());
    let expected = c.models.clone();
    captured.models.digest = hash(&captured.models.payload).unwrap();
    let objects = SavedObjects::capture(captured, Some(&Auth)).unwrap();
    let bytes = codec::encode(
        &objects,
        "77777777-7777-4777-8777-777777777777",
        "2026-11-07T00:00:00Z",
        None,
        Some(&Auth),
    )
    .unwrap()
    .into_bytes()
    .to_vec();
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    let offered = p.objects.iter().find(|o| o.kind == "template").unwrap();
    assert_eq!(offered.template_references, vec![r.clone()]);
    o.confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    let saved = store_v2::StoreV2::new(&f.d).snapshot_saved().unwrap();
    let c = saved
        .policy
        .channels
        .iter()
        .find(|c| c.id == scope.channels[0].portable_channel_id)
        .unwrap();
    for model in &expected {
        let actual = c
            .models
            .iter()
            .find(|m| m.raw_model_id == model.raw_model_id)
            .unwrap();
        assert_eq!(actual.binding, model.binding);
        assert_eq!(actual.explicit_overrides, model.explicit_overrides);
        for (field, value) in &model.source.fields {
            assert_eq!(actual.source.fields[field].value, value.value);
        }
    }
    let imported = saved
        .library
        .templates
        .iter()
        .find(|t| t.id == r.template_id)
        .unwrap();
    assert_eq!(imported.revisions[0].digest, r.digest);
    assert!(imported.default_revision.is_none());
}

struct MemoryRecoveryKey;
impl FileRecoveryKeysV2 for MemoryRecoveryKey {
    fn key(&self, _d: &Path, _h: &Path, _create: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
        Ok(Zeroizing::new([73; 32]))
    }
}

#[test]
fn encrypted_export_reopens_exact_ciphertext_after_source_changes_and_keeps_unknown_output() {
    let f = Fixture::new(true);
    let mut o = f.owner();
    let p = o
        .prepare_export(f.request(), Some(Zeroizing::new(PASSWORD.into())))
        .unwrap();
    let bytes = o.bytes(&p.id).unwrap().to_vec();
    drop(o);
    std::fs::write(
        f.h.join("config.json"),
        b"later invalid config must not be read",
    )
    .unwrap();
    let mut o = f.owner();
    let reopened = o.reopen_export(&p.id).unwrap();
    assert_eq!(reopened.digest, p.digest);
    assert_eq!(o.bytes(&p.id).unwrap(), bytes);
    assert!(!o.cancel_export(&p.id).unwrap());
    assert_eq!(
        code(o.finish_export(&p.id, &schema_v2::digest(b"partial"))),
        "file_output_unknown"
    );
    let receipt = o.finish_export(&p.id, &schema_v2::digest(&bytes)).unwrap();
    drop(o);
    let mut o = f.owner();
    assert_eq!(o.finish_export(&p.id, &p.digest).unwrap(), receipt);
    assert_eq!(
        std::fs::read(f.h.join("config.json")).unwrap(),
        b"later invalid config must not be read"
    );
}
#[test]
fn prepared_import_reopen_uses_frozen_target_and_applied_repeat_is_read_only_original_fact() {
    let s = Fixture::new(true);
    let (bytes, scope) = exported(&s, true);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(
            bytes.clone(),
            Some(Zeroizing::new(PASSWORD.into())),
            Some(scope.clone()),
            new_binding(&scope),
        )
        .unwrap();
    assert!(p.can_confirm, "{:?}", p.reason_code);
    let payload = o.imports[&p.operation_id]
        .application
        .as_ref()
        .unwrap()
        .payload
        .to_vec();
    drop(o);
    let mut o = f.owner();
    let restored = o.reopen_import(&p.operation_id).unwrap();
    assert_eq!(restored.plan_digest, p.plan_digest);
    assert_eq!(&*o.imports[&p.operation_id].original, &bytes);
    assert_eq!(
        &*o.imports[&p.operation_id]
            .application
            .as_ref()
            .unwrap()
            .payload,
        &payload
    );
    assert!(o.imports[&p.operation_id].passphrase.is_none());
    let receipt = o
        .confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    drop(o);
    std::fs::write(f.h.join("config.json"), b"user's later invalid config").unwrap();
    std::fs::write(
        f.d.join(store_v2::LIBRARY_PATH),
        b"user's later invalid library",
    )
    .unwrap();
    let before = std::fs::read(recovery_path(&f.d, &p.operation_id).unwrap()).unwrap();
    let mut o = f.owner();
    assert_eq!(
        o.confirm_import(&p.operation_id, &p.plan_digest, None)
            .unwrap(),
        receipt
    );
    assert_eq!(
        std::fs::read(recovery_path(&f.d, &p.operation_id).unwrap()).unwrap(),
        before
    );
    assert_eq!(
        std::fs::read(f.h.join("config.json")).unwrap(),
        b"user's later invalid config"
    );
    assert_eq!(
        std::fs::read(f.d.join(store_v2::LIBRARY_PATH)).unwrap(),
        b"user's later invalid library"
    );
}
#[test]
fn prepared_reopen_cannot_recapture_a_changed_target_as_new_baseline() {
    let s = Fixture::new(true);
    let (bytes, scope) = exported(&s, false);
    let f = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    drop(o);
    let changed = br#"{"providers":{},"userLater":true}"#;
    std::fs::write(f.h.join("config.json"), changed).unwrap();
    let mut o = f.owner();
    assert_eq!(
        o.reopen_import(&p.operation_id).unwrap().plan_digest,
        p.plan_digest
    );
    assert!(o
        .confirm_import(&p.operation_id, &p.plan_digest, None)
        .is_err());
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), changed);
}
#[test]
fn recovery_is_authenticated_root_bound_and_contains_no_plain_secrets_or_file_passphrase() {
    let s = Fixture::new(true);
    let (bytes, scope) = password_file(&s);
    let f = Fixture::new(false);
    let other = Fixture::new(false);
    let mut o = f.owner();
    let p = o
        .prepare_import(
            bytes,
            Some(Zeroizing::new(PASSWORD.into())),
            Some(scope),
            Default::default(),
        )
        .unwrap();
    let path = recovery_path(&f.d, &p.operation_id).unwrap();
    let original = std::fs::read(&path).unwrap();
    let text = String::from_utf8_lossy(&original);
    assert!(!text.contains(DAV_PASSWORD));
    assert!(!text.contains(PASSWORD));
    assert!(!text.contains("document_b64"));
    let (frozen, _) = o.load_recovery(&p.operation_id).unwrap();
    let inner = Zeroizing::new(serde_json::to_vec(&frozen).unwrap());
    assert!(!String::from_utf8_lossy(&inner).contains(PASSWORD));
    let copied = recovery_path(&other.d, &p.operation_id).unwrap();
    std::fs::create_dir_all(copied.parent().unwrap()).unwrap();
    crate::infrastructure::atomic_write::atomic_write(&copied, &original, 0o600).unwrap();
    assert_eq!(
        code(other.owner().reopen_import(&p.operation_id)),
        "file_recovery_authentication"
    );
    let mut doc: serde_json::Value = serde_json::from_slice(&original).unwrap();
    let ciphertext = doc["ciphertext"].as_str().unwrap();
    let mut bad = ciphertext.as_bytes().to_vec();
    bad[10] = if bad[10] == b'A' { b'B' } else { b'A' };
    doc["ciphertext"] = serde_json::Value::String(String::from_utf8(bad).unwrap());
    crate::infrastructure::atomic_write::atomic_write(
        &path,
        &serde_json::to_vec(&doc).unwrap(),
        0o600,
    )
    .unwrap();
    assert_eq!(
        code(f.owner().reopen_import(&p.operation_id)),
        "file_recovery_authentication"
    );
}
#[test]
fn stale_owner_returns_newly_completed_original_fact_without_overwriting_later_edits() {
    let s = Fixture::new(true);
    let (bytes, scope) = exported(&s, false);
    let f = Fixture::new(false);
    let mut a = f.owner();
    let p = a
        .prepare_import(bytes, None, Some(scope.clone()), new_binding(&scope))
        .unwrap();
    let mut stale = f.owner();
    stale.reopen_import(&p.operation_id).unwrap();
    let receipt = a
        .confirm_import(&p.operation_id, &p.plan_digest, None)
        .unwrap();
    std::fs::write(
        f.h.join("config.json"),
        b"later edits after completed receipt",
    )
    .unwrap();
    assert_eq!(
        stale
            .confirm_import(&p.operation_id, &p.plan_digest, None)
            .unwrap(),
        receipt
    );
    assert_eq!(
        std::fs::read(f.h.join("config.json")).unwrap(),
        b"later edits after completed receipt"
    );
}
#[test]
fn cancel_removes_only_unattempted_protected_artifact_and_listing_survives_owner_restart() {
    let f = Fixture::new(true);
    let mut o = f.owner();
    let p = o.prepare_export(f.request(), None).unwrap();
    drop(o);
    let mut o = f.owner();
    let list = o.recoverable_operations().unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].operation_id, p.id);
    assert!(!list[0].attempted);
    assert!(o.cancel_export(&p.id).unwrap());
    assert!(o.recoverable_operations().unwrap().is_empty());
    assert!(!recovery_path(&f.d, &p.id).unwrap().exists());
}
#[test]
fn namespace_adoption_is_an_explicit_whole_channel_intent_never_an_implicit_import() {
    let source = Fixture::new(true);
    let (bytes, scope) = exported(&source, false);
    let mut c = decode(&bytes, None).unwrap().objects.into_captured();
    c.models.payload.policy.as_mut().unwrap().channels[0].namespace =
        Some(super::super::schema_v2::NamespaceV2 {
            channel_id: scope.channels[0].portable_channel_id.clone(),
            remove_leading_prefixes: vec!["vendor/".into()],
        });
    c.models.digest = hash(&c.models.payload).unwrap();
    let objects = SavedObjects::capture(c, Some(&Auth)).unwrap();
    let bytes = codec::encode(
        &objects,
        "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
        "2026-11-07T00:00:00Z",
        None,
        Some(&Auth),
    )
    .unwrap()
    .into_bytes()
    .to_vec();
    let f = Fixture::new(false);
    let mut o = f.owner();
    let choices = new_binding(&scope);
    let p = o
        .prepare_import(bytes, None, Some(scope.clone()), choices.clone())
        .unwrap();
    assert_eq!(
        p.reason_code.as_deref(),
        Some("model_receive_namespace_adoption_required")
    );
    let mut adopt = choices;
    adopt.adopt_namespace_channel_ids = vec![scope.channels[0].portable_channel_id.clone()];
    let p = o
        .reselect(
            &p.operation_id,
            &p.plan_digest,
            scope.clone(),
            adopt.clone(),
        )
        .unwrap();
    assert_eq!(
        p.reason_code.as_deref(),
        Some("file_namespace_receiver_not_connected")
    );
    let mut partial = scope;
    partial.channels[0].model_ids = Some(vec!["vendor/custom".into()]);
    let p = o
        .reselect(&p.operation_id, &p.plan_digest, partial, adopt)
        .unwrap();
    assert_eq!(
        p.reason_code.as_deref(),
        Some("file_namespace_scope_invalid")
    );
}

#[test]
fn import_inventory_preserves_only_actual_public_account_platform_metadata() {
    let f = Fixture::new(false);
    let request = CaptureRequestV2 {
        selection: SelectionV2 {
            usage: UsageV2::LocalFile,
            ..Default::default()
        },
        mcp_clients: Default::default(),
        assets_reviewed: false,
    };
    // An authenticated wrapper supplies account metadata; UI never names the
    // provider/origin to be adopted or supplies a credential identity.
    let mut captured = saved_objects_capture_v2::capture(&f.d, &f.h, &request, &NoCredentials)
        .unwrap()
        .objects
        .into_captured();
    let account = portable_accounts::AccountInfo {
        id: "11111111-1111-4111-8111-111111111111".into(),
        provider: "chatgpt".into(),
        subject: Some("synthetic-subject".into()),
        label: "Synthetic account".into(),
        email: None,
        origin: Some(portable_accounts::AccountOrigin {
            version: 1,
            account_platform: "chatgpt".into(),
            owner: "opencodex".into(),
            auth_kind: "oauth".into(),
            source_namespace: "codex-account-pool".into(),
            source_account_id: "fixture-slot".into(),
            tenant: None,
        }),
    };
    captured.manager.accounts.push(account.clone());
    let objects = SavedObjects::capture(captured, Some(&Auth)).unwrap();
    let file = codec::encode(
        &objects,
        "99999999-9999-4999-8999-999999999999",
        "2026-11-07T00:00:00Z",
        None,
        Some(&Auth),
    )
    .unwrap();
    let preview = f
        .owner()
        .prepare_import(
            file.into_bytes().to_vec(),
            None,
            Some(request.selection.clone()),
            ImportBindingsV2::default(),
        )
        .unwrap();
    let item = preview
        .objects
        .iter()
        .find(|o| o.kind == "account")
        .unwrap();
    assert_eq!(item.account_info.as_ref(), Some(&account));
    assert!(item.information_supported);
    assert_eq!(item.label, "Synthetic account");
    assert!(!serde_json::to_string(item).unwrap().contains("accessToken"));
}
