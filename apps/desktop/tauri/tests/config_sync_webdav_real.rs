//! Explicit local/owned HTTPS WebDAV acceptance for the new object domain.
//! This test never deletes remote objects. Point it at an isolated test directory.
#![cfg(feature = "integration-test")]
use opencodex_desktop_lib::{
    infrastructure::webdav_client::{
        ReqwestWebDavTransport, WebDavConfig, WebDavMethod, WebDavTransport,
    },
    modules::{
        model_config::{
            config_remote::{self, Scope},
            file_objects,
            schema::*,
            ModelStore,
        },
        preferences::{Preferences, PreferencesStore},
    },
};
fn endpoint() -> WebDavConfig {
    let config = WebDavConfig {
        base_url: std::env::var("OCXD_CONFIG_WEBDAV_URL").expect("set owned test TLS URL"),
        remote_path: std::env::var("OCXD_CONFIG_WEBDAV_PATH")
            .expect("set an isolated owned directory"),
        username: std::env::var("OCXD_CONFIG_WEBDAV_USER").expect("set fixture user"),
        password: std::env::var("OCXD_CONFIG_WEBDAV_PASSWORD").expect("set fixture password"),
    };
    config.validate().unwrap();
    config
}
fn template(root: &std::path::Path, label: &str) -> String {
    let store = ModelStore::new(root);
    let snapshot = store.load().unwrap();
    let mut content = TemplateContent::default();
    content.matching.include.push(Pattern {
        kind: MatchKind::Prefix,
        value: label.into(),
    });
    store
        .save(SaveTemplate {
            expected_library: snapshot.library_digest,
            template_id: None,
            base_revision: None,
            label: label.into(),
            content,
        })
        .unwrap()
        .template_id
}
#[tokio::test]
#[ignore = "requires the explicit owned HTTPS fixture; writes only temporary local configurations"]
async fn current_file_channel_roundtrips_over_verified_https_before_explicit_file_save() {
    use opencodex_desktop_lib::modules::model_config::config_file;
    use serde_json::json;
    let mut config = endpoint();
    config.remote_path += "/file-authority";
    let ca = std::fs::read(std::env::var_os("OCXD_CONFIG_WEBDAV_CA").expect("fixture CA")).unwrap();
    let transport = ReqwestWebDavTransport::with_test_root_certificate(&ca).unwrap();
    let source = tempfile::tempdir().unwrap();
    let target = tempfile::tempdir().unwrap();
    let source_home = source.path().join("home");
    let target_home = target.path().join("home");
    for (root, home) in [(source.path(), &source_home), (target.path(), &target_home)] {
        std::fs::create_dir_all(home).unwrap();
        std::fs::create_dir_all(root.join("manager-state")).unwrap();
        std::fs::write(root.join(POLICY_PATH), b"broken legacy policy").unwrap();
        std::fs::write(root.join(DRAFT_PATH), b"broken legacy draft").unwrap();
    }
    let source_raw = json!({"providers":{"fixture":{"adapter":"openai-chat","authMode":"local","baseUrl":"https://site.example.test/v1","models":["site/custom-id"],"selectedModels":["site/custom-id"],"note":"real TLS roundtrip"}}});
    std::fs::write(
        source_home.join("config.json"),
        serde_json::to_vec(&source_raw).unwrap(),
    )
    .unwrap();
    let target_bytes = br#"{"providers":{},"unrelated":{"keep":true}}"#;
    std::fs::write(target_home.join("config.json"), target_bytes).unwrap();
    let scope = config_remote::save_scope(
        source.path(),
        Scope {
            all_channels: true,
            include_global: true,
            preferences: false,
            extensions: false,
            upstream_root: Some(source_home.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    let prepared = config_remote::prepare(
        source.path(),
        &transport,
        &config,
        "isolated-tls",
        scope,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    let published = config_remote::publish(
        source.path(),
        &transport,
        &config,
        &prepared.id,
        "isolated-tls",
        || Ok(()),
    )
    .await
    .unwrap();
    assert_eq!(published.phase, "published");
    config_remote::save_scope(
        target.path(),
        Scope {
            all_channels: true,
            include_global: true,
            preferences: false,
            extensions: false,
            ..Default::default()
        },
    )
    .unwrap();
    let incoming = config_remote::download_context_file(
        target.path(),
        &transport,
        &config,
        None,
        None,
        &target_home,
        || Ok(()),
    )
    .await
    .unwrap();
    assert!(std::fs::read(target_home.join("config.json")).unwrap() == target_bytes);
    let adopted = file_objects::accept(target.path(), &incoming.id, false, None).unwrap();
    let candidate = adopted.workspace.unwrap();
    assert_eq!(candidate.base_policy, adopted.policy_digest);
    assert!(std::fs::read(target_home.join("config.json")).unwrap() == target_bytes);
    let saved = config_file::save(
        target.path(),
        &target_home,
        config_file::SaveRequest {
            scope: None,
            expected_file: adopted.policy_digest,
            policy: candidate.policy,
            credentials: vec![],
        },
    )
    .unwrap();
    assert!(saved.changed);
    let reopened = config_file::snapshot(target.path(), &target_home).unwrap();
    assert_eq!(reopened.policy.channels.len(), 1);
    assert_eq!(reopened.policy.channels[0].model_order, ["site/custom-id"]);
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(target_home.join("config.json")).unwrap()).unwrap();
    assert_eq!(raw["unrelated"], json!({"keep":true}));
    assert_eq!(
        raw["providers"][&reopened.policy.channels[0].connection.provider_id]["note"],
        "real TLS roundtrip"
    );
    assert!(std::fs::read(target.path().join(DRAFT_PATH)).unwrap() == b"broken legacy draft");
    println!("HTTPS current-file transfer passed: actual reqwest TLS + conditional publication + download, damaged legacy copies ignored, adopt did not write config, explicit Save/reopen preserved exact model and unrelated data; no external service or user file");
}
#[tokio::test]
#[ignore = "needs explicit owned TLS WebDAV fixture and OCXD_CONFIG_WEBDAV_CA for its local CA"]
async fn conditional_publish_preserves_remote_objects_and_device_scoped_reception() {
    let config = endpoint();
    let ca = std::fs::read(
        std::env::var_os("OCXD_CONFIG_WEBDAV_CA").expect("set local fixture CA file"),
    )
    .unwrap();
    let transport = ReqwestWebDavTransport::with_test_root_certificate(&ca).unwrap();
    let first = tempfile::tempdir().unwrap();
    let a = template(first.path(), "fixture-a");
    PreferencesStore::new(first.path())
        .save(&Preferences {
            interface_scale: 125,
            ..Default::default()
        })
        .unwrap();
    transport
        .request(
            &config,
            WebDavMethod::Put,
            "latest.txt",
            Some(b"old-client-fixture\n".to_vec()),
        )
        .await
        .unwrap();
    let scope = config_remote::save_scope(
        first.path(),
        Scope {
            template_ids: vec![a.clone()],
            preferences: true,
            extensions: false,
            ..Default::default()
        },
    )
    .unwrap();
    let plan = config_remote::prepare(
        first.path(),
        &transport,
        &config,
        "test-binding",
        scope,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    let competing = tempfile::tempdir().unwrap();
    let concurrent_scope = config_remote::save_scope(competing.path(), Scope::default()).unwrap();
    let competitor = config_remote::prepare(
        competing.path(),
        &transport,
        &config,
        "test-binding",
        concurrent_scope,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    assert_eq!(
        config_remote::publish(
            first.path(),
            &transport,
            &config,
            &plan.id,
            "test-binding",
            || Ok(())
        )
        .await
        .unwrap()
        .phase,
        "published"
    );
    assert!(
        matches!(config_remote::publish(competing.path(),&transport,&config,&competitor.id,"test-binding",||Ok(())).await,Err(error) if error.code=="remote_changed")
    );
    let second = tempfile::tempdir().unwrap();
    let b = template(second.path(), "fixture-b");
    let scope = config_remote::save_scope(
        second.path(),
        Scope {
            template_ids: vec![b.clone()],
            preferences: false,
            extensions: true,
            ..Default::default()
        },
    )
    .unwrap();
    let next = config_remote::prepare(
        second.path(),
        &transport,
        &config,
        "test-binding",
        scope,
        None,
        || Ok(()),
    )
    .await
    .unwrap();
    assert!(
        next.package.payload.templates.iter().any(|t| t.id == a)
            && next.package.payload.templates.iter().any(|t| t.id == b)
    );
    assert_eq!(
        next.manager.preferences.as_ref().unwrap().interface_scale,
        125
    );
    config_remote::publish(
        second.path(),
        &transport,
        &config,
        &next.id,
        "test-binding",
        || Ok(()),
    )
    .await
    .unwrap();
    let receiver = tempfile::tempdir().unwrap();
    let receiver_scope = config_remote::save_scope(
        receiver.path(),
        Scope {
            template_ids: vec![a.clone()],
            preferences: true,
            extensions: false,
            ..Default::default()
        },
    )
    .unwrap();
    let before = ModelStore::new(receiver.path()).load().unwrap();
    let incoming = config_remote::download(receiver.path(), &transport, &config, None, || Ok(()))
        .await
        .unwrap();
    assert_eq!(ModelStore::new(receiver.path()).load().unwrap(), before);
    let saved = file_objects::accept(receiver.path(), &incoming.id, false, None).unwrap();
    assert!(saved.library.templates.iter().any(|t| t.id == a));
    assert!(!saved.library.templates.iter().any(|t| t.id == b));
    assert_eq!(
        PreferencesStore::new(receiver.path())
            .load()
            .unwrap()
            .interface_scale,
        125
    );
    assert_eq!(
        config_remote::load_scope(receiver.path()).unwrap(),
        receiver_scope
    );
    assert_eq!(
        transport
            .request(&config, WebDavMethod::Get, "latest.txt", None)
            .await
            .unwrap()
            .body,
        b"old-client-fixture\n"
    );
    println!("TLS WebDAV config domain: conditional create/update/readback passed; competing plan refused; partial union preserved; device-scoped review/adoption passed; old domain intact; no remote deletion");
}
