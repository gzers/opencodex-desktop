//! Real bounded files/locks/private atomic writes; no endpoint or secret owner.
use super::super::objects_overlay_v2::{ChannelSelectionV2, McpSourceSelectionV2};
use super::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use tempfile::TempDir;
const A: &str = "11111111-1111-4111-8111-111111111111";
const B: &str = "22222222-2222-4222-8222-222222222222";
const C: &str = "33333333-3333-4333-8333-333333333333";
fn fixture() -> (TempDir, PathBuf, PathBuf) {
    let t = tempfile::Builder::new()
        .prefix("ocxd023-file-scope-v2-")
        .tempdir_in(crate::modules::test_sandbox::temporary_parent())
        .unwrap();
    let d = t.path().join("D");
    let h = t.path().join("H");
    std::fs::create_dir_all(&d).unwrap();
    std::fs::create_dir_all(&h).unwrap();
    (t, d, h)
}
fn empty() -> ScopeRequestV2 {
    ScopeRequestV2 {
        selection: SelectionV2 {
            usage: UsageV2::LocalFile,
            ..Default::default()
        },
        mcp_clients: BTreeMap::new(),
        assets_reviewed: false,
    }
}
fn full() -> ScopeRequestV2 {
    let mut r = empty();
    let s = &mut r.selection;
    s.channels.push(ChannelSelectionV2 {
        portable_channel_id: A.into(),
        model_ids: Some(vec!["vendor/raw-model:variant".into()]),
        family_ids: None,
        include_connection: true,
        include_key: true,
        include_authorization: false,
    });
    s.template_refs.push(schema_v2::RefV2 {
        template_id: B.into(),
        revision_id: C.into(),
        digest: schema_v2::digest(b"exact revision"),
    });
    s.adopt_template_defaults = s.template_refs.clone();
    s.include_global_template_refs = true;
    s.manager.preferences = true;
    s.manager.extensions = true;
    s.manager.account_ids = vec![B.into()];
    s.manager.webdav_ids = vec!["default".into()];
    s.manager.webdav_password_ids = vec!["default".into()];
    s.skill_names = vec!["Fixture skill".into()];
    s.extension_credentials.skill_names = s.skill_names.clone();
    s.mcp_sources = vec![McpSourceSelectionV2 {
        name: "fixture-mcp".into(),
        definition_digest: schema_v2::digest(b"public mcp"),
    }];
    s.extension_credentials.mcp_names = vec!["fixture-mcp".into()];
    r.mcp_clients.insert("fixture-mcp".into(), "managed".into());
    r.assets_reviewed = true;
    r
}
fn equal(a: &ScopeRequestV2, b: &ScopeRequestV2) {
    assert_eq!(canonical(a).unwrap(), canonical(b).unwrap());
}
fn code<T>(result: ModelResult<T>) -> String {
    match result {
        Ok(_) => panic!("expected refusal"),
        Err(e) => e.code,
    }
}
fn stored(d: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(d.join(RELATIVE_PATH)).unwrap()).unwrap()
}
fn write_value(d: &Path, value: &Value) {
    atomic_write(&d.join(RELATIVE_PATH), &canonical(value).unwrap(), 0o600).unwrap();
}

#[test]
fn missing_read_is_none_and_does_not_create_state_or_default_choices() {
    let (_t, d, h) = fixture();
    assert!(read(&d, &h).unwrap().is_none());
    assert!(!d.join("manager-state").exists());
    assert_eq!(std::fs::read_dir(&h).unwrap().count(), 0);
}
#[test]
fn full_local_scope_round_trips_without_any_endpoint_capture_or_secret_access() {
    let (_t, d, h) = fixture();
    // If preferences read actual business sources this malformed material fails.
    let files = [
        d.join(schema_v2_path::LIBRARY),
        d.join(schema_v2_path::DRAFT),
        d.join("manager-state/model-policy.json"),
        h.join("config.json"),
        d.join("manager-state/sync-endpoints.json"),
        d.join("dailyKey"),
    ];
    for p in &files {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"synthetic forbidden source/secret").unwrap();
    }
    let r = full();
    save(&d, &h, r.clone()).unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &r);
    for p in &files {
        assert_eq!(
            std::fs::read(p).unwrap(),
            b"synthetic forbidden source/secret"
        );
    }
    let bytes = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    assert!(!text.contains("synthetic forbidden"));
    assert!(!text.contains(d.to_str().unwrap()));
    assert!(!text.contains(h.to_str().unwrap()));
    for field in [
        "payloadPassphrase",
        "password",
        "sourcePath",
        "keyValues",
        "endpointUrl",
    ] {
        assert!(!text.contains(field));
    }
    assert!(bytes.len() < BYTE_LIMIT);
    assert_eq!(bytes, canonical(&stored(&d)).unwrap());
    assert_eq!(stored(&d)["schemaVersion"], 2);
    assert_eq!(stored(&d)["initialized"], true);
    #[cfg(unix)]
    assert_eq!(
        crate::infrastructure::platform::mode_of(&d.join(RELATIVE_PATH)).unwrap(),
        0o600
    );
}
mod schema_v2_path {
    pub const LIBRARY: &str = "manager-state/model-family-templates.json";
    pub const DRAFT: &str = "manager-state/model-config-draft.json";
}
#[test]
fn explicit_empty_selection_stays_initialized_including_after_a_prior_full_selection() {
    let (_t, d, h) = fixture();
    save(&d, &h, full()).unwrap();
    save(&d, &h, empty()).unwrap();
    let r = read(&d, &h).unwrap().unwrap();
    equal(&r, &empty());
    assert_eq!(stored(&d)["initialized"], true);
    assert!(r.selection.channels.is_empty());
    // New saved business objects do not cause implicit selection expansion.
    std::fs::write(h.join("config.json"), b"new unrelated source").unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &empty());
}
#[test]
fn reviewed_flag_is_only_a_preference_not_authority_to_discover_new_objects() {
    let (_t, d, h) = fixture();
    let mut r = empty();
    r.assets_reviewed = true;
    save(&d, &h, r.clone()).unwrap();
    std::fs::create_dir_all(h.join(".codex/skills/new-skill")).unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &r);
    assert!(read(&d, &h)
        .unwrap()
        .unwrap()
        .selection
        .skill_names
        .is_empty());
}
#[test]
fn canonical_root_home_binding_rejects_copied_or_moved_preferences_and_save_cannot_reset_them() {
    let (t, d, h) = fixture();
    save(&d, &h, full()).unwrap();
    let original = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
    let h2 = t.path().join("H2");
    std::fs::create_dir(&h2).unwrap();
    assert_eq!(code(read(&d, &h2)), "file_scope_binding");
    assert_eq!(code(save(&d, &h2, empty())), "file_scope_binding");
    assert_eq!(std::fs::read(d.join(RELATIVE_PATH)).unwrap(), original);
    let d2 = t.path().join("D2");
    std::fs::create_dir_all(d2.join("manager-state")).unwrap();
    atomic_write(&d2.join(RELATIVE_PATH), &original, 0o600).unwrap();
    assert_eq!(code(read(&d2, &h)), "file_scope_binding");
}
#[test]
fn relative_missing_file_and_non_directory_roots_are_rejected_even_without_a_record() {
    let (t, d, h) = fixture();
    assert_eq!(code(read(Path::new("relative"), &h)), "file_scope_root");
    assert_eq!(code(read(&d, &t.path().join("absent"))), "file_scope_root");
    let file = t.path().join("file-root");
    std::fs::write(&file, b"file").unwrap();
    assert_eq!(code(save(&d, &file, empty())), "file_scope_root");
    assert_eq!(code(read(&d.join(".."), &h)), "file_scope_root");
    assert!(!d.join("manager-state").exists());
}
#[test]
fn bad_format_schema_uninitialized_and_unknown_fields_never_become_none_or_reset() {
    let (_t, d, h) = fixture();
    save(&d, &h, full()).unwrap();
    let original = stored(&d);
    let mut invalid = vec![];
    for (key, value) in [
        ("format", json!("OCXD_OLD_SCOPE")),
        ("schemaVersion", json!(999)),
        ("initialized", json!(false)),
        ("rootHomeDigest", json!("sha256:bad")),
        ("password", json!("do-not-echo")),
    ] {
        let mut v = original.clone();
        v[key] = value;
        invalid.push(v);
    }
    let mut v = original.clone();
    v["request"]["sourcePath"] = json!("/must/not/read");
    invalid.push(v);
    let mut v = original.clone();
    v["request"]["selection"]["unexpected"] = json!(true);
    invalid.push(v);
    for v in invalid {
        write_value(&d, &v);
        let before = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
        let error = read(&d, &h).unwrap_err();
        assert!(!error.message.contains("do-not-echo"));
        assert!(save(&d, &h, empty()).is_err());
        assert_eq!(std::fs::read(d.join(RELATIVE_PATH)).unwrap(), before);
    }
    atomic_write(&d.join(RELATIVE_PATH), b"not json", 0o600).unwrap();
    assert_eq!(code(read(&d, &h)), "file_scope_format");
}
#[test]
fn strict_recursive_duplicate_keys_trailing_data_and_omitted_explicit_fields_are_rejected() {
    let (_t, d, h) = fixture();
    save(&d, &h, full()).unwrap();
    let original = stored(&d);
    let bytes = canonical(&original).unwrap();
    let text = String::from_utf8(bytes).unwrap();
    for candidate in [
        text.replacen("{", "{\"format\":\"duplicate\",", 1),
        text.replace(
            "\"assetsReviewed\":true",
            "\"assetsReviewed\":false,\"assetsReviewed\":true",
        ),
        text.replace("\"managed\"", "\"managed\",\"fixture-mcp\":\"codex\""),
        format!("{text} {{}}"),
    ] {
        atomic_write(&d.join(RELATIVE_PATH), candidate.as_bytes(), 0o600).unwrap();
        assert_eq!(code(read(&d, &h)), "file_scope_format");
    }
    for field in ["channels", "manager", "extensionCredentials", "usage"] {
        let mut v = original.clone();
        v["request"]["selection"]
            .as_object_mut()
            .unwrap()
            .remove(field);
        write_value(&d, &v);
        assert_eq!(code(read(&d, &h)), "file_scope_format");
    }
    for field in ["includeKey", "includeConnection", "familyIds"] {
        let mut v = original.clone();
        v["request"]["selection"]["channels"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        write_value(&d, &v);
        assert_eq!(code(read(&d, &h)), "file_scope_format");
    }
}
#[test]
fn exact_byte_budget_is_exclusive_and_failed_save_preserves_prior_record() {
    let (_t, d, h) = fixture();
    save(&d, &h, empty()).unwrap();
    let before = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
    let mut padded = before.clone();
    padded.resize(BYTE_LIMIT - 1, b' ');
    atomic_write(&d.join(RELATIVE_PATH), &padded, 0o600).unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &empty());
    padded.push(b' ');
    atomic_write(&d.join(RELATIVE_PATH), &padded, 0o600).unwrap();
    assert_eq!(code(read(&d, &h)), "file_scope_limit");
    assert_eq!(code(save(&d, &h, empty())), "file_scope_limit");
    atomic_write(&d.join(RELATIVE_PATH), &before, 0o600).unwrap();
    let mut r = full();
    r.selection.channels[0].model_ids = Some(
        (0..400)
            .map(|i| format!("{i:04}-{}", "x".repeat(1500)))
            .collect(),
    );
    assert_eq!(code(save(&d, &h, r)), "file_scope_limit");
    assert_eq!(std::fs::read(d.join(RELATIVE_PATH)).unwrap(), before);
}

#[test]
fn local_usage_uuids_revision_hashes_raw_model_ids_and_duplicate_selections_are_validated() {
    let (_t, d, h) = fixture();
    save(&d, &h, empty()).unwrap();
    let before = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
    let mut variants = vec![];
    macro_rules! bad {
        ($r:ident, $body:expr) => {{
            let mut $r = full();
            $body;
            variants.push($r);
        }};
    }
    bad!(r, r.selection.usage = UsageV2::RemoteSync);
    bad!(
        r,
        r.selection.channels[0].portable_channel_id = "label-is-not-id".into()
    );
    bad!(
        r,
        r.selection.channels.push(r.selection.channels[0].clone())
    );
    bad!(r, r.selection.channels[0].family_ids = Some(vec![B.into()]));
    bad!(r, {
        r.selection.channels[0].model_ids = None;
        r.selection.channels[0].include_connection = false;
        r.selection.channels[0].include_key = false;
        r.selection.channels[0].family_ids = Some(vec!["not-uuid".into()]);
    });
    bad!(r, r.selection.channels[0].model_ids = Some(vec![]));
    bad!(
        r,
        r.selection.channels[0].model_ids = Some(vec!["same".into(), "same".into()])
    );
    bad!(
        r,
        r.selection.channels[0].model_ids = Some(vec!["model\nsecret".into()])
    );
    bad!(
        r,
        r.selection.channels[0].model_ids = Some(vec!["x".repeat(2049)])
    );
    bad!(r, r.selection.channels[0].include_authorization = true);
    bad!(r, r.selection.channels[0].include_connection = false);
    bad!(
        r,
        r.selection.template_refs[0].revision_id = "bad-ref".into()
    );
    bad!(r, r.selection.template_refs[0].digest = "SHA256:bad".into());
    bad!(
        r,
        r.selection.adopt_template_defaults[0].template_id = "bad-template".into()
    );
    bad!(
        r,
        r.selection
            .template_refs
            .push(r.selection.template_refs[0].clone())
    );
    bad!(
        r,
        r.selection
            .adopt_template_defaults
            .push(r.selection.adopt_template_defaults[0].clone())
    );
    bad!(
        r,
        r.selection.manager.account_ids = vec!["account label".into()]
    );
    bad!(
        r,
        r.selection.manager.webdav_ids = vec!["https://secret.invalid".into()]
    );
    bad!(
        r,
        r.selection.manager.webdav_password_ids = vec!["unselected".into()]
    );
    for r in variants {
        assert!(save(&d, &h, r).is_err());
        assert_eq!(std::fs::read(d.join(RELATIVE_PATH)).unwrap(), before);
    }
}
#[test]
fn asset_names_credential_subsets_and_mcp_sources_never_accept_paths_or_unknown_owners() {
    let (_t, d, h) = fixture();
    for name in [
        "",
        ".",
        "..",
        "../source",
        "dir/name",
        "dir\\name",
        "bad\nname",
        "name.",
        "name ",
        "C:source",
    ] {
        let mut r = full();
        r.selection.skill_names = vec![name.into()];
        r.selection.extension_credentials.skill_names.clear();
        assert!(save(&d, &h, r).is_err());
    }
    for source in [
        "",
        "unknown",
        "Codex",
        "/source/home",
        "https://secret.invalid",
        "keychain:secret",
    ] {
        let mut r = full();
        r.mcp_clients.insert("fixture-mcp".into(), source.into());
        assert!(save(&d, &h, r).is_err());
    }
    let mut r = full();
    r.mcp_clients.insert("unselected".into(), "codex".into());
    assert!(save(&d, &h, r).is_err());
    let mut r = full();
    r.selection.mcp_sources[0].definition_digest = "not-a-digest".into();
    assert!(save(&d, &h, r).is_err());
    let mut r = full();
    r.selection.skill_names.push("fixture SKILL".into());
    assert!(save(&d, &h, r).is_err());
    let mut r = full();
    r.selection
        .extension_credentials
        .mcp_names
        .push("unselected".into());
    assert!(save(&d, &h, r).is_err());
    for client in std::iter::once("managed".to_owned())
        .chain(CLIENT_IDS.iter().map(|c| portable_assets::client_key(*c)))
    {
        let mut r = full();
        r.mcp_clients.insert("fixture-mcp".into(), client);
        save(&d, &h, r.clone()).unwrap();
        equal(&read(&d, &h).unwrap().unwrap(), &r);
    }
}
#[test]
fn same_roots_need_no_process_singleton_and_multiple_roots_keep_independent_preferences() {
    let (t, d, h) = fixture();
    let d2 = t.path().join("other-D");
    std::fs::create_dir_all(&d2).unwrap();
    save(&d, &h, full()).unwrap();
    save(&d2, &h, empty()).unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &full());
    equal(&read(&d2, &h).unwrap().unwrap(), &empty());
    // Canonical lexical spelling does not create a different binding.
    equal(&read(&d.join("."), &h.join(".")).unwrap().unwrap(), &full());
}
#[test]
fn fixed_file_lock_serializes_writers_and_last_explicit_save_can_clear_scope() {
    let (_t, d, h) = fixture();
    save(&d, &h, empty()).unwrap();
    let guard = TargetFileLock::lock(&d.join(RELATIVE_PATH)).unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    let worker_d = d.clone();
    let worker_h = h.clone();
    let worker = std::thread::spawn(move || {
        tx.send("started").unwrap();
        save(&worker_d, &worker_h, full()).unwrap();
        tx.send("saved").unwrap();
    });
    assert_eq!(rx.recv().unwrap(), "started");
    assert!(rx
        .recv_timeout(std::time::Duration::from_millis(50))
        .is_err());
    drop(guard);
    assert_eq!(
        rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap(),
        "saved"
    );
    worker.join().unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &full());
    save(&d, &h, empty()).unwrap();
    equal(&read(&d, &h).unwrap().unwrap(), &empty());
}
#[test]
fn rejected_selection_leaves_existing_preferences_and_no_temporary_files() {
    let (_t, d, h) = fixture();
    save(&d, &h, empty()).unwrap();
    let before = std::fs::read(d.join(RELATIVE_PATH)).unwrap();
    // Invalid selection is rejected before reaching publication.
    let mut r = full();
    r.selection.skill_names = vec!["x".repeat(257)];
    assert!(save(&d, &h, r).is_err());
    assert_eq!(std::fs::read(d.join(RELATIVE_PATH)).unwrap(), before);
    assert!(std::fs::read_dir(d.join("manager-state"))
        .unwrap()
        .all(|e| !e.unwrap().file_name().to_string_lossy().contains(".tmp-")));
}
#[cfg(unix)]
#[test]
fn symlinked_record_parent_lock_or_root_and_hardlinked_record_are_never_followed() {
    use std::os::unix::fs::symlink;
    for kind in ["record", "parent", "lock", "root", "home", "hardlink"] {
        let (t, d, h) = fixture();
        let external = t.path().join("external");
        std::fs::create_dir(&external).unwrap();
        let original = external.join("do-not-touch");
        atomic_write(&original, b"outside bytes", 0o600).unwrap();
        if kind == "parent" {
            symlink(&external, d.join("manager-state")).unwrap();
        } else if kind == "root" || kind == "home" {
            let link = t.path().join("root-link");
            symlink(if kind == "root" { &d } else { &h }, &link).unwrap();
            assert!(read(
                if kind == "root" { &link } else { &d },
                if kind == "home" { &link } else { &h }
            )
            .is_err());
            continue;
        } else {
            std::fs::create_dir_all(d.join("manager-state")).unwrap();
            let target = if kind == "lock" {
                lock_path_for(&d.join(RELATIVE_PATH))
            } else {
                d.join(RELATIVE_PATH)
            };
            if kind == "hardlink" {
                std::fs::hard_link(&original, &target).unwrap();
            } else {
                symlink(&original, &target).unwrap();
            }
        }
        assert!(read(&d, &h).is_err());
        assert!(save(&d, &h, empty()).is_err());
        assert_eq!(std::fs::read(original).unwrap(), b"outside bytes");
    }
}
#[cfg(unix)]
#[test]
fn non_private_record_and_actual_write_failure_do_not_repair_or_overwrite() {
    use std::os::unix::fs::PermissionsExt;
    let (_t, d, h) = fixture();
    save(&d, &h, empty()).unwrap();
    let p = d.join(RELATIVE_PATH);
    let bytes = std::fs::read(&p).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(code(read(&d, &h)), "file_scope_path");
    assert_eq!(code(save(&d, &h, full())), "file_scope_path");
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
    let state = d.join("manager-state");
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o500)).unwrap();
    let result = save(&d, &h, full());
    std::fs::set_permissions(&state, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(code(result), "file_scope_io");
    assert_eq!(std::fs::read(p).unwrap(), bytes);
}
