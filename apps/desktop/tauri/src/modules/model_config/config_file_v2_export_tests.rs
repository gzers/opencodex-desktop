use super::*;
use crate::modules::model_config::{
    objects_overlay_v2::{ChannelSelectionV2, SelectionV2},
    saved_objects_capture_v2::SelectedKeySourceV2,
    schema::Field,
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct NoCredentials(AtomicUsize);
impl CaptureCredentialsV2 for NoCredentials {
    fn channel_key(
        &self,
        _h: &Path,
        _s: &SelectedKeySourceV2<'_>,
    ) -> ModelResult<Zeroizing<String>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Err(invalid("unexpected_credential_read", "test"))
    }
}
struct Fixture {
    _temp: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    draft: DraftV2,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let d = temp.path().join("D");
        let h = temp.path().join("H");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir_all(&h).unwrap();
        let raw = json!({"providers":{"actual":{"name":"actual","adapter":"openai-chat","baseUrl":"https://actual.invalid/v1","authMode":"key","apiKey":"SYNTHETIC-SECRET-MUST-NOT-EXPORT","models":["vendor/custom","other"],"selectedModels":["vendor/custom","other"],"modelContextWindows":{"vendor/custom":8192}},"unselected":{"name":"unselected","adapter":"openai-chat","baseUrl":"https://unselected.invalid/v1","models":["other"],"selectedModels":["other"]}},"unrelated":{"keep":true}});
        std::fs::write(h.join("config.json"), canonical_bytes(&raw).unwrap()).unwrap();
        let s = super::super::snapshot(&d, &h).unwrap();
        let started = super::super::begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: None,
            },
        )
        .unwrap();
        Self {
            _temp: temp,
            d,
            h,
            draft: started.draft.unwrap(),
        }
    }
    fn id(&self) -> String {
        self.draft
            .policy
            .channels
            .iter()
            .find(|c| c.connection.provider_id == "actual")
            .unwrap()
            .id
            .clone()
    }
    fn request(&self) -> CaptureRequestV2 {
        CaptureRequestV2 {
            selection: SelectionV2 {
                usage: UsageV2::LocalFile,
                channels: vec![ChannelSelectionV2 {
                    portable_channel_id: self.id(),
                    model_ids: Some(vec!["vendor/custom".into()]),
                    family_ids: None,
                    include_connection: true,
                    include_key: false,
                    include_authorization: false,
                }],
                ..Default::default()
            },
            mcp_clients: BTreeMap::new(),
            assets_reviewed: false,
        }
    }
    fn intent(&self) -> SaveRequestV2 {
        SaveRequestV2 {
            expected: self.draft.base.clone(),
            edits: vec![],
            channel_intents: vec![],
            scope_channel_id: None,
        }
    }
    fn stage(&mut self, mut update: impl FnMut(&mut DraftV2)) {
        let mut next = self.draft.clone();
        update(&mut next);
        next.base.draft_generation += 1;
        self.draft = StoreV2::new(&self.d)
            .stage_draft(Some(&self.draft.base), next)
            .unwrap();
    }
    fn bytes(&self) -> BTreeMap<PathBuf, Vec<u8>> {
        fn visit(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for row in std::fs::read_dir(dir).unwrap() {
                let path = row.unwrap().path();
                if path.is_dir() {
                    visit(&path, out)
                } else if !path.to_string_lossy().ends_with(".lock") {
                    out.insert(path.clone(), std::fs::read(path).unwrap());
                }
            }
        }
        let mut out = BTreeMap::new();
        visit(&self.d, &mut out);
        visit(&self.h, &mut out);
        out
    }
}
#[test]
fn project_accepts_equivalent_disk_rewrite_without_writing_or_relaxing_real_conflicts() {
    let f = Fixture::new();
    let path = f.h.join("config.json");
    let (mut raw, _) = read_raw(&f.h).unwrap();
    raw["modelDiscovery"] = json!({"knownModels":{"actual":{"models":["other"]}}});
    std::fs::write(&path, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    let before = f.bytes();
    project(&f.d, &f.h, f.intent())
        .unwrap_or_else(|error| panic!("equivalent projection refused: {error:?}"));
    assert_eq!(before, f.bytes());
    raw["unrelated"]["keep"] = json!(false);
    std::fs::write(&path, serde_json::to_vec(&raw).unwrap()).unwrap();
    let before = f.bytes();
    assert_eq!(
        project(&f.d, &f.h, f.intent()).err().unwrap().code,
        "model_read_stale"
    );
    assert_eq!(before, f.bytes());
}

#[test]
fn project_applies_only_explicit_intents_compiles_and_changes_no_business_file() {
    let f = Fixture::new();
    let before = f.bytes();
    let mut r = f.intent();
    r.edits.push(ModelEditV2::SetOverride {
        channel_id: f.id(),
        raw_model_id: "vendor/custom".into(),
        field: "contextWindow".into(),
        value: json!(4096),
    });
    r.channel_intents.push(ChannelIntentV2::SetNaming {
        channel_id: f.id(),
        raw_model_id: "vendor/custom".into(),
        alias: Some("lab-model".into()),
        display_name: Some("Readable".into()),
    });
    let projected = project(&f.d, &f.h, r).unwrap();
    assert_eq!(projected.expected, f.draft.base);
    assert!(valid_digest(&projected.digest));
    let model = projected
        .effective
        .iter()
        .find(|m| m.channel_id == f.id() && m.raw_model_id == "vendor/custom")
        .unwrap();
    assert_eq!(model.rules.context_window, Field::Custom(4096));
    assert_eq!(
        projected
            .intended
            .channels
            .iter()
            .find(|c| c.id == f.id())
            .unwrap()
            .models[0]
            .source,
        f.draft
            .policy
            .channels
            .iter()
            .find(|c| c.id == f.id())
            .unwrap()
            .models[0]
            .source
    );
    assert!(!serde_json::to_string(&projected)
        .unwrap()
        .contains("SYNTHETIC-SECRET"));
    assert_eq!(before, f.bytes());
}
#[test]
fn project_rejects_actual_alias_collisions_and_invalid_aliases_without_writes() {
    let f = Fixture::new();
    let before = f.bytes();
    for alias in ["vendor-custom", "bad alias"] {
        let mut r = f.intent();
        r.channel_intents.push(ChannelIntentV2::SetNaming {
            channel_id: f.id(),
            raw_model_id: "other".into(),
            alias: Some(alias.into()),
            display_name: None,
        });
        assert!(project(&f.d, &f.h, r).is_err());
    }
    let mut r = f.intent();
    for (raw, alias) in [("other", "collision"), ("vendor/custom", "COLLISION")] {
        r.channel_intents.push(ChannelIntentV2::SetNaming {
            channel_id: f.id(),
            raw_model_id: raw.into(),
            alias: Some(alias.into()),
            display_name: None,
        });
    }
    assert_eq!(
        project(&f.d, &f.h, r).err().unwrap().code,
        "model_alias_collision"
    );
    assert_eq!(before, f.bytes());
}
#[test]
fn draft_capture_exports_selected_typed_edits_exact_dependencies_and_no_credentials() {
    let mut f = Fixture::new();
    let id = f.id();
    let library = StoreV2::new(&f.d).snapshot_saved().unwrap().library;
    let t = &library.templates[0];
    let old = &t.revisions[0];
    let reference = RefV2 {
        template_id: t.id.clone(),
        revision_id: old.id.clone(),
        digest: old.digest.clone(),
    };
    f.stage(|d| {
        let c = d.policy.channels.iter_mut().find(|c| c.id == id).unwrap();
        let m = c
            .models
            .iter_mut()
            .find(|m| m.raw_model_id == "vendor/custom")
            .unwrap();
        m.explicit_overrides.context_window = Field::Custom(555);
        m.binding = Some(ModelBindingV2::Manual {
            reference: reference.clone(),
        });
    });
    let before = f.bytes();
    let vault = NoCredentials(AtomicUsize::new(0));
    let output = ConfigDraftFileCaptureV2
        .capture(&f.d, &f.h, &f.draft.base, &f.request(), &vault)
        .unwrap();
    assert_eq!(before, f.bytes());
    assert_eq!(vault.0.load(Ordering::SeqCst), 0);
    let objects = output.objects.into_captured();
    let p = objects.models.payload.policy.as_ref().unwrap();
    assert_eq!(p.channels.len(), 1);
    assert_eq!(p.channels[0].models.len(), 1);
    assert_eq!(p.channels[0].models[0].raw_model_id, "vendor/custom");
    assert_eq!(
        p.channels[0].models[0].explicit_overrides.context_window,
        Field::Custom(555)
    );
    assert_eq!(
        p.channels[0].models[0].source.fields["contextWindow"].value,
        json!(8192)
    );
    assert_eq!(objects.models.payload.templates.len(), 1);
    assert_eq!(objects.models.payload.templates[0].revisions.len(), 1);
    assert!(objects.models.payload.templates[0]
        .default_revision
        .is_none());
    assert!(!serde_json::to_string(&objects.models)
        .unwrap()
        .contains("SYNTHETIC-SECRET"));
    assert_eq!(objects.channel_keys.get(&id), None);
}
#[test]
fn exact_base_baseline_and_pending_guards_never_recover_or_mutate() {
    let f = Fixture::new();
    let before = f.bytes();
    for n in 0..6 {
        let mut r = f.intent();
        match n {
            0 => r.expected.draft_generation += 1,
            1 => r.expected.root_revision = digest(b"other-root"),
            2 => r.expected.source_digest = digest(b"other-source"),
            3 => r.expected.library_digest = digest(b"other-library"),
            4 => r.expected.policy_digest = digest(b"other-policy"),
            _ => r.expected.draft_id = uuid::Uuid::new_v4().to_string(),
        };
        assert!(project(&f.d, &f.h, r).is_err());
    }
    assert_eq!(before, f.bytes());
    for file in [
        "saved-file-save-pending.json",
        "provider-rename-pending.json",
        "model-transaction-v2.pending.json",
        "credential-references-v2.pending.json",
    ] {
        let p = f.d.join("manager-state").join(file);
        std::fs::write(&p, b"invalid pending preserved").unwrap();
        let before = f.bytes();
        assert_eq!(
            project(&f.d, &f.h, f.intent()).err().unwrap().code,
            "model_transaction_pending"
        );
        assert_eq!(before, f.bytes());
        std::fs::remove_file(p).unwrap();
    }
    let mut f = f;
    f.stage(|d| {
        d.observed_baseline.as_mut().unwrap().channels[0].label = "tampered observation".into();
    });
    let before = f.bytes();
    assert_eq!(
        project(&f.d, &f.h, f.intent()).err().unwrap().code,
        "capture_draft_baseline_stale"
    );
    assert_eq!(before, f.bytes());
}
#[test]
fn draft_export_scope_is_local_explicit_and_saved_capture_still_ignores_bad_draft() {
    let f = Fixture::new();
    let vault = NoCredentials(AtomicUsize::new(0));
    let mut req = f.request();
    req.selection.channels[0].include_key = true;
    assert!(ConfigDraftFileCaptureV2
        .capture(&f.d, &f.h, &f.draft.base, &req, &vault)
        .is_err());
    req = f.request();
    req.selection.usage = UsageV2::RemoteSync;
    assert!(ConfigDraftFileCaptureV2
        .capture(&f.d, &f.h, &f.draft.base, &req, &vault)
        .is_err());
    assert_eq!(vault.0.load(Ordering::SeqCst), 0);
    std::fs::write(f.d.join(store_v2::DRAFT_PATH), b"not JSON").unwrap();
    let before = f.bytes();
    assert!(capture::capture(&f.d, &f.h, &f.request(), &vault).is_ok());
    assert!(ConfigDraftFileCaptureV2
        .capture(&f.d, &f.h, &f.draft.base, &f.request(), &vault)
        .is_err());
    assert_eq!(before, f.bytes());
}
#[test]
fn new_server_staged_channel_uses_template_protocol_without_saving_or_guessing_adapter() {
    let mut f = Fixture::new();
    let s = super::super::stage_connection(
        &f.d,
        &f.h,
        StageConnectionV2 {
            expected: Some(f.draft.base.clone()),
            expected_source: f.draft.base.source_digest.clone(),
            expected_library: f.draft.base.library_digest.clone(),
            channel_id: None,
            name: "Fresh Draft".into(),
            url: "https://fresh.invalid/v1".into(),
            nodes: None,
            active_node_id: None,
            auth_mode: "none".into(),
            api_key: None,
            raw_model_ids: vec!["new/raw".into()],
            selected_model_ids: vec!["new/raw".into()],
        },
    )
    .unwrap();
    f.draft = s.draft.unwrap();
    let library = StoreV2::new(&f.d).snapshot_saved().unwrap().library;
    let t = &library.templates[0];
    let rev = &t.revisions[0];
    let reference = RefV2 {
        template_id: t.id.clone(),
        revision_id: rev.id.clone(),
        digest: rev.digest.clone(),
    };
    let id = f
        .draft
        .policy
        .channels
        .iter()
        .find(|c| c.label == "Fresh Draft")
        .unwrap()
        .id
        .clone();
    f.stage(|d| {
        d.policy
            .channels
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap()
            .models[0]
            .binding = Some(ModelBindingV2::Manual {
            reference: reference.clone(),
        });
    });
    let mut req = f.request();
    req.selection.channels[0].portable_channel_id = id;
    req.selection.channels[0].model_ids = None;
    let before = f.bytes();
    let output = ConfigDraftFileCaptureV2
        .capture(
            &f.d,
            &f.h,
            &f.draft.base,
            &req,
            &NoCredentials(AtomicUsize::new(0)),
        )
        .unwrap();
    let captured = output.objects.into_captured();
    let c = &captured.models.payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(
        c.connection.as_ref().unwrap().nodes[0].url,
        "https://fresh.invalid/v1"
    );
    assert!(c.models[0].source.fields.is_empty());
    assert_eq!(
        c.models[0].binding.as_ref().unwrap().reference(),
        &reference
    );
    assert_eq!(before, f.bytes());
}

#[test]
fn scoped_project_keeps_other_unsaved_channels_in_the_returned_dto_and_on_disk() {
    let mut f = Fixture::new();
    let selected = f.id();
    f.stage(|draft| {
        draft
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id != selected)
            .unwrap()
            .defaults
            .context_window = Field::Custom(17777);
    });
    let untouched = f
        .draft
        .policy
        .channels
        .iter()
        .find(|c| c.id != selected)
        .unwrap()
        .clone();
    let mut request = f.intent();
    request.scope_channel_id = Some(selected.clone());
    request.edits.push(ModelEditV2::SetOverride {
        channel_id: selected,
        raw_model_id: "vendor/custom".into(),
        field: "contextWindow".into(),
        value: json!(1234),
    });
    let before = f.bytes();
    let dto = project(&f.d, &f.h, request).unwrap();
    assert_eq!(
        dto.intended.channels.iter().find(|c| c.id == untouched.id),
        Some(&untouched)
    );
    assert_eq!(before, f.bytes());
}
#[test]
fn copied_draft_export_reuses_actual_public_seed_without_copying_key_or_saving() {
    let mut f = Fixture::new();
    let s = super::super::copy_channel(
        &f.d,
        &f.h,
        CopyChannelV2 {
            reuse_source_key: false,
            expected: f.draft.base.clone(),
            channel_id: f.id(),
            name: "Draft Copy".into(),
        },
    )
    .unwrap();
    f.draft = s.draft.unwrap();
    let copied = f
        .draft
        .policy
        .channels
        .iter()
        .find(|c| c.label == "Draft Copy")
        .unwrap();
    let mut req = f.request();
    req.selection.channels[0].portable_channel_id = copied.id.clone();
    let before = f.bytes();
    let result = ConfigDraftFileCaptureV2
        .capture(
            &f.d,
            &f.h,
            &f.draft.base,
            &req,
            &NoCredentials(AtomicUsize::new(0)),
        )
        .unwrap();
    let objects = result.objects.into_captured();
    let c = &objects.models.payload.policy.as_ref().unwrap().channels[0];
    assert!(c
        .connection
        .as_ref()
        .unwrap()
        .provider
        .as_ref()
        .unwrap()
        .get("apiKey")
        .is_none());
    assert_eq!(
        c.models[0].source.fields["contextWindow"].value,
        json!(8192)
    );
    assert_eq!(before, f.bytes());
}

#[test]
fn real_file_transfer_owner_uses_draft_capture_and_roundtrips_v6_without_business_writes() {
    use crate::modules::model_config::file_transfer_v2::{
        DraftExportRequestV2, FileTransferOwnerV2,
    };
    use std::sync::Arc;
    let mut f = Fixture::new();
    let id = f.id();
    f.stage(|draft| {
        draft
            .policy
            .channels
            .iter_mut()
            .find(|c| c.id == id)
            .unwrap()
            .models
            .iter_mut()
            .find(|m| m.raw_model_id == "vendor/custom")
            .unwrap()
            .explicit_overrides
            .context_window = Field::Custom(12345);
    });
    let before = f.bytes();
    let credentials = Arc::new(NoCredentials(AtomicUsize::new(0)));
    let mut owner = FileTransferOwnerV2::new(f.d.clone(), f.h.clone(), credentials.clone())
        .unwrap()
        .with_recovery_keys(Arc::new(MemoryRecoveryKey))
        .with_draft_capture(Arc::new(ConfigDraftFileCaptureV2));
    let plan = owner
        .prepare_draft_export(
            DraftExportRequestV2 {
                expected: f.draft.base.clone(),
                capture: f.request(),
            },
            None,
        )
        .unwrap();
    assert!(!plan.encrypted);
    let bytes = owner.bytes(&plan.id).unwrap();
    let decoded = file_objects_v2::decode(bytes, None, &plan.schema, None).unwrap();
    let captured = decoded.objects.into_captured();
    assert_eq!(captured.models.format_version, 6);
    assert_eq!(
        captured.models.payload.policy.as_ref().unwrap().channels[0].models[0]
            .explicit_overrides
            .context_window,
        Field::Custom(12345)
    );
    assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
    let sealed =
        f.d.join("manager-state/file-transfer-v2")
            .join(format!("{}.sealed", plan.id));
    assert_only_owner_state_added(&before, &f.bytes(), &sealed);
    let stored = std::fs::read(&sealed).unwrap();
    assert!(!String::from_utf8_lossy(&stored).contains("SYNTHETIC-SECRET"));
    let mut reopened = FileTransferOwnerV2::new(f.d.clone(), f.h.clone(), credentials)
        .unwrap()
        .with_recovery_keys(Arc::new(MemoryRecoveryKey));
    assert_eq!(
        reopened.reopen_export(&plan.id).unwrap().digest,
        plan.digest
    );
    let mut damaged: Value = serde_json::from_slice(&stored).unwrap();
    let mut ciphertext = damaged["ciphertext"].as_str().unwrap().as_bytes().to_vec();
    ciphertext[0] = if ciphertext[0] == b'A' { b'B' } else { b'A' };
    damaged["ciphertext"] = json!(String::from_utf8(ciphertext).unwrap());
    std::fs::write(&sealed, canonical_bytes(&damaged).unwrap()).unwrap();
    assert_eq!(
        reopened.reopen_export(&plan.id).unwrap_err().code,
        "file_recovery_authentication"
    );
    std::fs::write(&sealed, stored).unwrap();
    assert_only_owner_state_added(&before, &f.bytes(), &sealed);
}

#[test]
fn queued_model_and_channel_intents_are_captured_without_staging_or_read_rematching() {
    let f = Fixture::new();
    let before = f.bytes();
    let mut intent = f.intent();
    intent.edits.push(ModelEditV2::SetOverride {
        channel_id: f.id(),
        raw_model_id: "vendor/custom".into(),
        field: "contextWindow".into(),
        value: json!(4567),
    });
    intent.channel_intents.push(ChannelIntentV2::SetNaming {
        channel_id: f.id(),
        raw_model_id: "vendor/custom".into(),
        alias: Some("queued-model".into()),
        display_name: Some("Queued model".into()),
    });
    let result = capture_typed_intents(&f.d, &f.h, &f.request(), intent).unwrap();
    let objects = result.objects.into_captured();
    let c = &objects.models.payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(
        c.models[0].explicit_overrides.context_window,
        Field::Custom(4567)
    );
    assert_eq!(
        c.models[0].source.fields["contextWindow"].value,
        json!(8192)
    );
    assert!(c.models[0].binding.is_none());
    assert_eq!(
        c.naming["vendor/custom"].alias.as_deref(),
        Some("queued-model")
    );
    assert_eq!(
        c.connection.as_ref().unwrap().provider.as_ref().unwrap()["modelContextWindows"]
            ["vendor/custom"],
        4567
    );
    assert_eq!(before, f.bytes());
}
#[test]
fn queued_binding_includes_exact_revision_and_invalid_or_stale_queue_never_writes() {
    let f = Fixture::new();
    let library = StoreV2::new(&f.d).snapshot_saved().unwrap().library;
    let template = &library.templates[0];
    let revision = &template.revisions[0];
    let reference = RefV2 {
        template_id: template.id.clone(),
        revision_id: revision.id.clone(),
        digest: revision.digest.clone(),
    };
    let mut intent = f.intent();
    intent.edits.push(ModelEditV2::SetManualBinding {
        channel_id: f.id(),
        raw_model_id: "vendor/custom".into(),
        reference: reference.clone(),
    });
    let before = f.bytes();
    let result = capture_typed_intents(&f.d, &f.h, &f.request(), intent).unwrap();
    let objects = result.objects.into_captured();
    assert_eq!(objects.models.payload.templates[0].revisions.len(), 1);
    assert_eq!(
        objects.models.payload.templates[0].revisions[0].digest,
        reference.digest
    );
    let mut stale = f.intent();
    stale.expected.draft_generation += 1;
    assert!(capture_typed_intents(&f.d, &f.h, &f.request(), stale).is_err());
    let mut bad = f.intent();
    bad.channel_intents.push(ChannelIntentV2::SetNaming {
        channel_id: f.id(),
        raw_model_id: "other".into(),
        alias: Some("vendor-custom".into()),
        display_name: None,
    });
    assert!(capture_typed_intents(&f.d, &f.h, &f.request(), bad).is_err());
    assert_eq!(before, f.bytes());
}

struct MemoryRecoveryKey;
impl crate::modules::model_config::file_transfer_v2::FileRecoveryKeysV2 for MemoryRecoveryKey {
    fn key(&self, _d: &Path, _h: &Path, _create: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
        Ok(Zeroizing::new([73; 32]))
    }
}
fn assert_only_owner_state_added(
    before: &BTreeMap<PathBuf, Vec<u8>>,
    after: &BTreeMap<PathBuf, Vec<u8>>,
    sealed: &Path,
) {
    // Hash/path diagnostics only: never dump captured business bytes or keys.
    let original = before
        .iter()
        .map(|(p, b)| (p.clone(), digest(b)))
        .collect::<BTreeMap<_, _>>();
    let retained = after
        .iter()
        .filter(|(p, _)| p.as_path() != sealed)
        .map(|(p, b)| (p.clone(), digest(b)))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(original, retained);
    let bytes = after
        .get(sealed)
        .expect("only the controlled owner operation is added");
    let protected: Value = serde_json::from_slice(bytes).unwrap();
    assert_eq!(protected["version"], 2);
    assert_eq!(
        protected
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        ["version", "id", "nonce", "ciphertext"]
            .map(str::to_owned)
            .into_iter()
            .collect()
    );
    assert!(!String::from_utf8_lossy(bytes).contains("SYNTHETIC-SECRET"));
}

#[test]
fn real_owner_dispatches_owned_queued_intents_without_persisting_draft_or_reading_credentials() {
    use crate::modules::model_config::file_transfer_v2::{
        DraftExportRequestV2, DraftFileIntentsV2, FileTransferOwnerV2,
    };
    use std::sync::Arc;
    let f = Fixture::new();
    let credentials = Arc::new(NoCredentials(AtomicUsize::new(0)));
    let mut owner = FileTransferOwnerV2::new(f.d.clone(), f.h.clone(), credentials.clone())
        .unwrap()
        .with_recovery_keys(Arc::new(MemoryRecoveryKey))
        .with_draft_capture(Arc::new(ConfigDraftFileCaptureV2));
    let before = f.bytes();
    let plan = owner
        .prepare_draft_export_with_intents(
            DraftExportRequestV2 {
                expected: f.draft.base.clone(),
                capture: f.request(),
            },
            None,
            DraftFileIntentsV2 {
                edits: vec![ModelEditV2::SetOverride {
                    channel_id: f.id(),
                    raw_model_id: "vendor/custom".into(),
                    field: "contextWindow".into(),
                    value: json!(31234),
                }],
                channel_intents: vec![ChannelIntentV2::SetNaming {
                    channel_id: f.id(),
                    raw_model_id: "vendor/custom".into(),
                    alias: Some("owner-queue".into()),
                    display_name: None,
                }],
            },
        )
        .unwrap();
    let result =
        file_objects_v2::decode(owner.bytes(&plan.id).unwrap(), None, &plan.schema, None).unwrap();
    let objects = result.objects.into_captured();
    let c = &objects.models.payload.policy.as_ref().unwrap().channels[0];
    assert_eq!(
        c.models[0].explicit_overrides.context_window,
        Field::Custom(31234)
    );
    assert_eq!(
        c.naming["vendor/custom"].alias.as_deref(),
        Some("owner-queue")
    );
    assert_eq!(credentials.0.load(Ordering::SeqCst), 0);
    let sealed =
        f.d.join("manager-state/file-transfer-v2")
            .join(format!("{}.sealed", plan.id));
    assert_only_owner_state_added(&before, &f.bytes(), &sealed);
}
