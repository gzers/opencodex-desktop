use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
};

struct Fixture {
    _program: crate::modules::runtime::program_selection::tests::Fixture,
    data: PathBuf,
    client: PathBuf,
    context: ProcessContext,
}
impl Fixture {
    fn new() -> Self {
        let p = crate::modules::runtime::program_selection::tests::Fixture::new();
        let data = p.root.join("desktop");
        let profile = p.root.join("profile");
        let home = profile.join("opencodex-home");
        let client = profile.join(".codex");
        for path in [&data, &profile, &home, &client] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        fs::write(home.join("config.json"), br#"{"providers":{}}"#).unwrap();
        let runtime = crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &p.root);
        let context = ProcessContext {
            runtime,
            working_directory: profile,
            opencodex_home: home,
        };
        Self {
            _program: p,
            data,
            client,
            context,
        }
    }
    fn session(&self) -> StartupSession {
        // Storage/reference-only seam. This synthetic program is NOT official
        // service admission, and never enters begin's native lease/OS gate.
        let originals = Originals::begin(&self.data, &self.client).unwrap();
        let (root, selection) = source(&self.context).unwrap();
        StartupSession::persist(&self.context, originals, root, selection).unwrap()
    }
}

#[test]
fn immutable_startup_scope_reopens_only_with_independent_original_and_scope_references() {
    let f = Fixture::new();
    let session = f.session();
    let refs = session.references().unwrap();
    let reopened = StartupSession::open(&f.context, &f.data, &f.client, &refs).unwrap();
    assert_eq!(
        reopened.references().unwrap().scope_sha256,
        refs.scope_sha256
    );
    let mut wrong = refs.clone();
    wrong.scope_sha256 = "a".repeat(64);
    assert!(StartupSession::open(&f.context, &f.data, &f.client, &wrong).is_err());
    let mut wrong = refs.clone();
    wrong.original_sha256 = "b".repeat(64);
    assert!(StartupSession::open(&f.context, &f.data, &f.client, &wrong).is_err());
    assert!(reopened.live(&f.context).is_err());
    assert!(reopened
        .seal_pre_stop(
            &f.context,
            Instant::now() + std::time::Duration::from_secs(1)
        )
        .is_err());
}

#[test]
fn scope_cannot_move_to_another_home_profile_client_or_program() {
    let f = Fixture::new();
    let session = f.session();
    let refs = session.references().unwrap();
    for profile in [true, false] {
        let mut context = f.context.clone();
        let other = f._program.root.join(if profile {
            "other-profile"
        } else {
            "other-home"
        });
        fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
        if profile {
            context.working_directory = other;
        } else {
            context.opencodex_home = other;
        }
        assert!(StartupSession::open(&context, &f.data, &f.client, &refs).is_err());
    }
    let other = f._program.root.join("another-client");
    fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
    assert!(StartupSession::open(&f.context, &f.data, &other, &refs).is_err());
    let changed = Fixture::new();
    assert!(StartupSession::open(&changed.context, &f.data, &f.client, &refs).is_err());
}

#[test]
fn replaced_home_directory_and_tampered_scope_are_preserved_and_rejected() {
    let f = Fixture::new();
    let session = f.session();
    let refs = session.references().unwrap();
    let scope = session
        .originals
        .directory
        .path()
        .join("startup-scope.json");
    let raw = fs::read(&scope).unwrap();
    fs::set_permissions(&scope, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&scope, b"{\"schemaVersion\":1,\"schemaVersion\":1}").unwrap();
    fs::set_permissions(&scope, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(StartupSession::open(&f.context, &f.data, &f.client, &refs).is_err());
    assert!(session.scope(&f.context).is_err());
    assert!(!raw.is_empty());
    assert_eq!(
        fs::read(&scope).unwrap(),
        b"{\"schemaVersion\":1,\"schemaVersion\":1}"
    );
    let other = Fixture::new();
    let session = other.session();
    let refs = session.references().unwrap();
    let h = &other.context.opencodex_home;
    fs::rename(h, h.with_extension("retained-original")).unwrap();
    fs::DirBuilder::new().mode(0o700).create(h).unwrap();
    assert!(StartupSession::open(&other.context, &other.data, &other.client, &refs).is_err());
}

#[test]
fn missing_or_untrusted_runtime_record_cannot_create_a_ready_binding() {
    let f = Fixture::new();
    let home = PinnedDirectory::open(&f.context.opencodex_home).unwrap();
    assert!(fresh_ready(
        &f.context,
        &home,
        Instant::now() + std::time::Duration::from_secs(1)
    )
    .is_err());
    let path = f.context.opencodex_home.join("runtime-port.json");
    fs::write(&path, b"{\"pid\":1,\"port\":0}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(private_runtime(&home).is_err());
    fs::write(&path, b"{\"pid\":1,\"port\":2,\"pid\":3}").unwrap();
    assert!(private_runtime(&home).is_err());
    fs::write(&path, b"{\"pid\":1,\"port\":2}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(private_runtime(&home).is_err());
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o644
    );
}

#[test]
fn a_coordinator_client_path_is_not_the_effective_home_of_the_cleared_production_runtime() {
    let f = Fixture::new();
    assert_eq!(effective_client(&f.context).unwrap(), f.client);
    let wrong = f.context.working_directory.join("codex");
    fs::DirBuilder::new().mode(0o700).create(&wrong).unwrap();
    let originals = Originals::begin(&f.data, &wrong).unwrap();
    let (root, selection) = source(&f.context).unwrap();
    assert!(StartupSession::persist(&f.context, originals, root, selection).is_err());
    assert_eq!(fs::read_dir(&wrong).unwrap().count(), 0);
    assert_eq!(fs::read_dir(&f.client).unwrap().count(), 0);
}

#[test]
fn persisted_live_reference_is_not_an_endpoint_or_process_authority() {
    let f = Fixture::new();
    let mut session = f.session();
    let ready = session.originals.seal(Phase::Ready).unwrap();
    let live = Live {
        schema_version: 1,
        scope_sha256: session.scope_sha256.clone(),
        ready_sha256: ready,
        record_sha256: "a".repeat(64),
        generation: [42, unsafe { libc::geteuid() }.into(), 1, 1],
        port: 12345,
        parent_generation: [41, unsafe { libc::geteuid() }.into(), 1, 1],
    };
    let bytes = encode(&live).unwrap();
    create(&session.originals.directory, "startup-live.json", &bytes).unwrap();
    session.live_sha256 = Some(digest(&bytes));
    // Parsing these protected explicit test bytes is not a verified process.
    assert!(session.live(&f.context).is_ok());
    assert!(session
        .seal_pre_stop(
            &f.context,
            Instant::now() + std::time::Duration::from_secs(1)
        )
        .is_err());
    assert!(session.originals.read(Phase::PreStop).is_err());
    let mut refs = session.references().unwrap();
    refs.live_sha256 = Some("b".repeat(64));
    assert!(StartupSession::open(&f.context, &f.data, &f.client, &refs).is_err());
}

#[test]
fn ready_generation_fields_and_phase_references_are_bounded_and_typed() {
    let f = Fixture::new();
    let session = f.session();
    let ready = session.originals.seal(Phase::Ready).unwrap();
    let original = Live {
        schema_version: 1,
        scope_sha256: session.scope_sha256.clone(),
        ready_sha256: ready,
        record_sha256: "a".repeat(64),
        generation: [42, unsafe { libc::geteuid() }.into(), 1, 1],
        port: 12345,
        parent_generation: [41, unsafe { libc::geteuid() }.into(), 1, 1],
    };
    assert!(original.process().is_ok());
    for index in 0..4 {
        let mut changed = original.clone();
        changed.generation[index] = u64::MAX;
        assert!(changed.process().is_err());
    }
    let mut wrong = original.clone();
    wrong.port = 0;
    assert!(wrong.process().is_err());
    assert!(parse::<Live>(br#"{"schemaVersion":1,"schemaVersion":1}"#).is_err());
    assert!(fresh_ready(&f.context, &session.home, Instant::now()).is_err());
}

#[test]
fn independent_static_route_declaration_uses_official_slash_encoding_and_refuses_alias_collision() {
    let static_graph = serde_json::json!({"defaultModelAliases":false,"providers":{"Team_One":{"liveModels":false,"defaultAliases":false,"selectedModels":["gpt-6-sol","native/family-5.5"]}}});
    let bytes = serde_json::to_vec(&static_graph).unwrap();
    assert_eq!(
        declaration(&bytes).unwrap(),
        Some(vec![
            "Team_One/gpt-6-sol".into(),
            "Team_One/native-family-5.5".into()
        ])
    );
    let mut normalized = static_graph.clone();
    normalized["fastRows"] = true.into();
    assert_eq!(
        declaration(&serde_json::to_vec(&normalized).unwrap()).unwrap(),
        declaration(&bytes).unwrap()
    );
    normalized["fastRows"] = serde_json::json!("malformed");
    assert_eq!(
        declaration(&serde_json::to_vec(&normalized).unwrap()).unwrap(),
        None
    );
    let mut collision = static_graph.clone();
    collision["providers"]["Team_One"]["selectedModels"] =
        serde_json::json!(["native/family", "native-family"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&collision).unwrap()).unwrap(),
        None
    );
    let mut dynamic = static_graph.clone();
    dynamic["providers"]["Team_One"]["liveModels"] = true.into();
    dynamic["providers"]["Team_One"]["selectedModels"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&dynamic).unwrap()).unwrap(),
        None
    );
    let mut alias = static_graph;
    alias["customModels"] = serde_json::json!([{"id":"custom"}]);
    assert_eq!(
        declaration(&serde_json::to_vec(&alias).unwrap()).unwrap(),
        None
    );
    assert_eq!(declaration(br#"{"providers":null}"#).unwrap(), None);
    assert_eq!(declaration(br#"{}"#).unwrap(), None);
    assert_eq!(declaration(br#"{"providers":{}}"#).unwrap(), Some(vec![]));
    assert!(declaration(br#"{"providers":{},"providers":{}}"#).is_err());
}
#[test]
fn display_alias_flags_never_become_declared_route_ids_and_omitted_flags_are_valid() {
    let mut config = serde_json::json!({"providers":{"Team_One":{"liveModels":false,"selectedModels":["claude-opus-5-5","gpt-6-sol","vendor/gemini-3.5-pro"]}}});
    let expected = Some(vec![
        "Team_One/claude-opus-5-5".into(),
        "Team_One/gpt-6-sol".into(),
        "Team_One/vendor-gemini-3.5-pro".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for global in [false, true] {
        for local in [None, Some(false), Some(true)] {
            config["defaultModelAliases"] = global.into();
            if let Some(v) = local {
                config["providers"]["Team_One"]["defaultAliases"] = v.into();
            } else {
                config["providers"]["Team_One"]
                    .as_object_mut()
                    .unwrap()
                    .remove("defaultAliases");
            }
            config["providers"]["Team_One"]["modelAliases"] =
                serde_json::json!({"gpt-6-sol":"custom-sol"});
            config["fastRows"] = true.into();
            assert_eq!(
                declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
                expected
            );
        }
    }
    config["providers"]["Off"] = serde_json::json!({"disabled":true,"liveModels":true});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for (field, value) in [
        ("defaultAliases", serde_json::json!("bad")),
        ("modelAliases", serde_json::json!({"gpt-6-sol":3})),
        ("disabled", serde_json::json!("true")),
    ] {
        let mut bad = config.clone();
        bad["providers"]["Team_One"][field] = value;
        assert_eq!(
            declaration(&serde_json::to_vec(&bad).unwrap()).unwrap(),
            None
        );
    }
}
#[test]
fn explicit_nonempty_live_selection_is_independent_while_open_discovery_remains_unknown() {
    let mut config = serde_json::json!({"providers":{"Live":{"liveModels":true,"models":["stale-known-model"],"selectedModels":["claude-opus-5-5","vendor/gemini-3.5-pro"]}}});
    let expected = Some(vec![
        "Live/claude-opus-5-5".into(),
        "Live/vendor-gemini-3.5-pro".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Live"]
        .as_object_mut()
        .unwrap()
        .remove("liveModels");
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    for models in [serde_json::json!([]), serde_json::json!(null)] {
        let mut c = config.clone();
        c["providers"]["Live"]["selectedModels"] = models;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
    config["providers"]["Live"]["liveModels"] = serde_json::json!("true");
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
}
#[test]
fn selection_defaults_static_roster_unions_models_and_retained_but_not_unselected_default() {
    let mut config = serde_json::json!({"providers":{"Static":{"liveModels":false,"models":["a","b"],"retainModels":["b","vendor/c"],"defaultModel":"not-in-explicit-roster"}}});
    let expected = Some(vec![
        "Static/a".into(),
        "Static/b".into(),
        "Static/vendor-c".into(),
    ]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["selectedModels"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["selectedModels"] = serde_json::json!(["vendor/c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/vendor-c".into()])
    );
}
#[test]
fn selection_defaults_seed_only_static_empty_models_and_preserve_unknown_live_graph() {
    let mut config = serde_json::json!({"providers":{"Static":{"liveModels":false,"defaultModel":"seed","retainModels":["retained"]}}});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/retained".into(), "Static/seed".into()])
    );
    config["providers"]["Static"]["models"] = serde_json::json!([]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["Static/retained".into(), "Static/seed".into()])
    );
    config["providers"]["Static"]["liveModels"] = true.into();
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
}
#[test]
fn selection_defaults_disabled_equivalence_is_scoped_and_does_not_grant_colliding_ids() {
    let mut config = serde_json::json!({"disabledModels":["Static/vendor/c","native-only"],"providers":{"Static":{"liveModels":false,"models":["a","vendor/c"]},"Other":{"liveModels":false,"models":["vendor/c"]}}});
    let expected = Some(vec!["Other/vendor-c".into(), "Static/a".into()]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["disabledModels"] = serde_json::json!(["Static/vendor-c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        expected
    );
    config["providers"]["Static"]["models"] = serde_json::json!(["vendor/c", "vendor-c"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
    for bad in [
        serde_json::json!([3]),
        serde_json::json!(null),
        serde_json::json!({"Static/a":true}),
    ] {
        let mut c = config.clone();
        c["disabledModels"] = bad;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
}
#[test]
fn custom_declaration_overlays_same_model_and_adds_only_enabled_selected_custom_routes() {
    let mut config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["base"]}},"customModels":[{"provider":"P","modelId":"base","displayName":"friendly"},{"provider":"P","modelId":"vendor/custom","contextWindow":200000}]});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/base".into(), "P/vendor-custom".into()])
    );
    config["providers"]["P"]["selectedModels"] = serde_json::json!(["base"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/base".into()])
    );
    config["providers"]["P"]["selectedModels"] = serde_json::json!(["vendor/custom"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/vendor-custom".into()])
    );
    config["disabledModels"] = serde_json::json!(["P/vendor-custom"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec![])
    );
}
#[test]
fn custom_declaration_refuses_ambiguous_or_unbound_custom_routes_without_learning_from_catalog() {
    let config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["vendor/custom"]}},"customModels":[{"provider":"P","modelId":"vendor-custom"}]});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        None
    );
    for custom in [
        serde_json::json!(null),
        serde_json::json!({}),
        serde_json::json!([{"provider":"Missing","modelId":"x"}]),
        serde_json::json!([{"provider":"P","modelId":""}]),
        serde_json::json!([{"provider":"P","modelId":3}]),
        serde_json::json!([{"provider":"P","modelId":"same"},{"provider":"P","modelId":"same"}]),
    ] {
        let mut c = config.clone();
        c["customModels"] = custom;
        assert_eq!(declaration(&serde_json::to_vec(&c).unwrap()).unwrap(), None);
    }
}
#[test]
fn combo_declaration_uses_canonical_or_public_alias_without_relabeling_target_routes() {
    let mut config = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["a","b"]}},"combos":{"mixed":{"strategy":"failover","targets":[{"provider":"P","model":"a"},{"provider":"P","model":"b"}]},"named":{"alias":"Public/mixed","targets":[{"provider":"P","model":"a"}]}}});
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec![
            "P/a".into(),
            "P/b".into(),
            "Public/mixed".into(),
            "combo/mixed".into()
        ])
    );
    config["disabledModels"] = serde_json::json!(["Public/mixed"]);
    assert_eq!(
        declaration(&serde_json::to_vec(&config).unwrap()).unwrap(),
        Some(vec!["P/a".into(), "P/b".into(), "combo/mixed".into()])
    );
}
#[test]
fn combo_declaration_rejects_shadowing_missing_targets_nested_or_native_alias_authority() {
    let base = serde_json::json!({"providers":{"P":{"liveModels":false,"models":["a"]}},"combos":{"mixed":{"targets":[{"provider":"P","model":"a"}]}}});
    for change in [
        serde_json::json!({"alias":"P/a"}),
        serde_json::json!({"alias":"combo/other"}),
        serde_json::json!({"alias":"a/b/c"}),
        serde_json::json!({"nativeAlias":true,"alias":"gpt-5.4"}),
        serde_json::json!({"targets":[]}),
        serde_json::json!({"targets":[{"provider":"Missing","model":"a"}]}),
        serde_json::json!({"targets":[{"provider":"P","model":"missing"}]}),
        serde_json::json!({"targets":[{"provider":"combo","model":"nested"}]}),
        serde_json::json!({"targets":[{"provider":"P","model":"a"},{"provider":"P","model":"a"}]}),
    ] {
        let mut c = base.clone();
        for (k, v) in change.as_object().unwrap() {
            c["combos"]["mixed"][k] = v.clone();
        }
        assert_eq!(
            declaration(&serde_json::to_vec(&c).unwrap()).unwrap(),
            None,
            "{change}"
        );
    }
}
#[test]
fn audit_projection_preserves_canonical_private_path_and_refuses_incomplete_secret_redaction() {
    assert_eq!(
        audit_home(Path::new("/private/tmp/fixture/profile/opencodex-home")).unwrap(),
        "/private/tmp/fixture/profile/opencodex-home"
    );
    assert_eq!(
        audit_home(Path::new("/Users/example/.codex-desktop/opencodex/home")).unwrap(),
        "/Users/[USER]/.codex-desktop/opencodex/home"
    );
    assert!(audit_home(Path::new("/Users/example/secret-directory/home")).is_err());
    assert!(audit_home(Path::new(&format!("/private/tmp/{}", "x".repeat(256)))).is_err());
}

#[cfg(target_arch = "aarch64")]
#[test]
#[ignore = "explicit pinned official299 resources; new private profile; binding only, whole client restore still expected to fail"]
fn official299_real_startup_originals_ready_binding_and_completed_stop_roundtrip() {
    use crate::{
        commands::{
            process_action_with_runner, runtime_recovery_v2_owner::SystemServiceOwnerProofProvider,
        },
        infrastructure::{
            official_client_coordination::{
                native::with_native_coordination, Context as CoordinationContext,
            },
            process_runner::ControlledProcessRunner,
            runtime_log::RuntimeLog,
        },
        modules::{
            process::{LifecycleAction, LifecycleResult},
            runtime,
        },
        types::process_action::ProcessActionRequest,
    };
    let root =
        PathBuf::from(std::env::var_os("OCXD_CLIENT_SESSION_ROOT").expect("explicit private root"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
    );
    assert!(root
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("ocxd024-client-session-"));
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let data = profile.join("desktop");
    let home = profile.join("opencodex-home");
    let client = profile.join(".codex");
    let controls = root.join("control");
    for p in [&data, &home, &client, &controls] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    fs::create_dir(data.join("logs")).unwrap();
    fs::write(
        client.join("config.toml"),
        b"model = \"gpt-5.4\"\nmodel_provider = \"openai\"\n",
    )
    .unwrap();
    fs::write(client.join("opencodex-catalog.json"),serde_json::to_vec(&serde_json::json!({"models":[{"slug":"gpt-5.4","display_name":"Native fixture","visibility":"list","supported_in_api":true}]})).unwrap()).unwrap();
    let config = serde_json::json!({"hostname":"127.0.0.1","port":0,"openaiProviderTierVersion":2,"defaultProvider":"session-fixture","defaultModelAliases":false,"providers":{"session-fixture":{"adapter":"openai-chat","baseUrl":"http://127.0.0.1:1/v1","allowPrivateNetwork":true,"apiKey":"synthetic-unused","models":["session-model"],"selectedModels":["session-model"],"liveModels":false,"defaultAliases":false}}});
    fs::write(
        home.join("config.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    fs::set_permissions(home.join("config.json"), fs::Permissions::from_mode(0o600)).unwrap();
    let descriptor = fs::read(root.join("resources/runtime-bundle.manifest.json")).unwrap();
    let pin = digest(&descriptor);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let reuse = std::env::var_os("OCXD_CLIENT_SESSION_REUSE").map(PathBuf::from);
    let program = reuse.clone().unwrap_or_else(|| root.join("program"));
    if reuse.is_none() {
        runtime::bundled::versioned::provision(
            &program,
            &data,
            &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
            &descriptor,
            &pin,
        )
        .unwrap();
    }
    let selected = runtime::program_selection::read(
        &program,
        crate::modules::data_root::layout::Platform::Macos,
        "darwin-arm64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(selected.selection.version, "2.99.0");
    assert_eq!(
        selected.image.official_archive_sha256,
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    runtime::program_image::verify(&selected).unwrap();
    let runtime = runtime::RuntimeHandle::initialize_versioned(&data, &program);
    let context = ProcessContext {
        runtime: runtime.clone(),
        working_directory: profile.clone(),
        opencodex_home: home.clone(),
    };
    let coordination = CoordinationContext {
        runtime,
        user_profile: profile.clone(),
        codex_home: client.clone(),
        opencodex_home: home.clone(),
        control_root: controls,
    };
    let mut report = serde_json::json!({"accepted":false,"scope":"official299 native startup originals->signed live binding->real normal Stop binding; no client mutation permit","result":"incomplete"});
    let output = root.join("native-summary.json");
    let write =
        |r: &serde_json::Value| fs::write(&output, serde_json::to_vec_pretty(r).unwrap()).unwrap();
    write(&report);
    let start_time = Instant::now();
    let start_deadline = start_time + std::time::Duration::from_secs(30);
    let mut session = with_native_coordination(&coordination, start_deadline, |lease| {
        StartupSession::begin(&context, &data, &client, lease, start_deadline)
    })
    .unwrap();
    let runner = std::sync::Mutex::new(ControlledProcessRunner::new(&profile));
    let log = RuntimeLog::new(&data);
    let started = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Start,
            confirm: true,
        },
        &runner,
        &context,
        &log,
        None,
    )
    .unwrap();
    report["start"] = serde_json::to_value(&started).unwrap();
    write(&report);
    assert_eq!(started.result, Some(LifecycleResult::Started));
    let ready = with_native_coordination(&coordination, start_deadline, |lease| {
        session.bind_ready(&context, lease, start_deadline)
    });
    report["readyBound"] = serde_json::json!(ready.is_ok());
    report["startupTotalMs"] = serde_json::json!(start_time.elapsed().as_millis());
    if let Err(ref e) = ready {
        report["readyError"] = serde_json::json!(e.to_string());
    }
    write(&report);
    ready.unwrap();
    let refs = session.references().unwrap();
    fs::write(
        root.join("independent-session-references.json"),
        serde_json::to_vec_pretty(&refs).unwrap(),
    )
    .unwrap();
    let reopened = StartupSession::open(&context, &data, &client, &refs).unwrap();
    let stop_time = Instant::now();
    let stop_deadline = stop_time + std::time::Duration::from_secs(10);
    let mut pair = reopened.watch_stop_pair(&context, stop_deadline).unwrap();
    reopened.seal_pre_stop(&context, stop_deadline).unwrap();
    let owner = SystemServiceOwnerProofProvider::new(program, context.clone()).unwrap();
    let stopped = owner.stop_lifecycle_verified(&context, stop_deadline, &data);
    report["ordinaryStopVerified"] = serde_json::json!(stopped.is_ok());
    write(&report);
    let stopped = stopped.unwrap();
    let pair = pair.wait(stop_deadline).unwrap();
    let bound = with_native_coordination(&coordination, stop_deadline, |lease| {
        reopened.seal_completed_stop(&context, &stopped, &pair, lease, stop_deadline)
    });
    report["completedStopBound"] = serde_json::json!(bound.is_ok());
    report["stopTotalMs"] = serde_json::json!(stop_time.elapsed().as_millis());
    if let Err(ref e) = bound {
        report["stopBindingError"] = serde_json::json!(e.to_string());
    }
    write(&report);
    bound.unwrap();
    fs::write(
        root.join("completed-session-references.json"),
        serde_json::to_vec_pretty(&reopened.references().unwrap()).unwrap(),
    )
    .unwrap();
    let original = reopened.originals.read(Phase::Original).unwrap();
    let after = reopened.originals.read(Phase::Stopped).unwrap();
    report["threeOriginalByteImagesEqualAfterStop"] = serde_json::json!(original
        .images
        .iter()
        .zip(&after.images)
        .all(|(a, b)| match (a, b) {
            (Image::Absent, Image::Absent) => true,
            (
                Image::File {
                    bytes: a,
                    protection: ap,
                    ..
                },
                Image::File {
                    bytes: b,
                    protection: bp,
                    ..
                },
            ) => a == b && ap == bp,
            _ => false,
        }));
    assert!(report["startupTotalMs"].as_u64().unwrap() < 30000);
    assert!(report["stopTotalMs"].as_u64().unwrap() < 10000);
    report["result"] = serde_json::json!("passed-binding-only");
    write(&report);
    println!("OFFICIAL299_BOUND_SESSION {report}");
}
