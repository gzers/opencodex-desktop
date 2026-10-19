//! Opt-in installed-official contract test. All user/client/runtime paths are
//! temporary, upstream traffic is loopback, and credentials use an in-memory
//! test backend. No package download, real account, or paid generation.
#![cfg(unix)]
use opencodex_desktop_lib::{
    modules::model_config::{self, compiler, discovery, runtime, schema::*, upstream},
    state::ProcessContext,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::io::AsyncReadExt;

type Secrets = Arc<Mutex<BTreeMap<String, Vec<u8>>>>;
struct MemoryBuilder(Secrets);
struct MemorySecret {
    data: Secrets,
    id: String,
}
impl keyring::credential::CredentialBuilderApi for MemoryBuilder {
    fn build(
        &self,
        _: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<keyring::credential::Credential>> {
        Ok(Box::new(MemorySecret {
            data: self.0.clone(),
            id: format!("{service}/{user}"),
        }))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
impl keyring::credential::CredentialApi for MemorySecret {
    fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
        self.data
            .lock()
            .unwrap()
            .insert(self.id.clone(), value.to_vec());
        Ok(())
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.data
            .lock()
            .unwrap()
            .get(&self.id)
            .cloned()
            .ok_or(keyring::Error::NoEntry)
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        self.data
            .lock()
            .unwrap()
            .remove(&self.id)
            .map(|_| ())
            .ok_or(keyring::Error::NoEntry)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
struct ProcessGroup(u32);
impl Drop for ProcessGroup {
    fn drop(&mut self) {
        unsafe {
            libc::kill(-(self.0 as i32), libc::SIGKILL);
        }
    }
}
fn quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}
fn free_port() -> u16 {
    std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

async fn history_plan(
    manager: &Path,
    context: &ProcessContext,
    source_id: &str,
) -> runtime::SavePlanView {
    let store = model_config::store::ModelStore::new(manager);
    let current = store.load().unwrap();
    let recovery = runtime::prepare_recovery(
        manager,
        model_config::recovery::Request {
            source_id: source_id.into(),
            side: model_config::recovery::Side::Before,
            scope: model_config::recovery::Scope::Affected,
            restore_defaults: false,
            expected_library: current.library_digest,
            expected_policy: current.policy_digest,
            expected_workspace: current.workspace_digest,
        },
    )
    .unwrap();
    let staged = runtime::accept_recovery(manager, &recovery.id).unwrap();
    runtime::prepare(
        manager,
        context,
        staged.workspace.unwrap().policy,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap()
}
#[tokio::test]
#[ignore = "requires explicit OCXD_TEST_OFFICIAL_ENTRY and OCXD_TEST_OFFICIAL_NODE; runs installed official in temporary HOME"]
async fn isolated_official_discovery_save_and_protocol_mapping_readback() {
    use std::os::unix::fs::PermissionsExt;
    let entry = std::path::PathBuf::from(
        std::env::var_os("OCXD_TEST_OFFICIAL_ENTRY")
            .expect("set the installed official bin/ocx.mjs path"),
    );
    let node = std::path::PathBuf::from(
        std::env::var_os("OCXD_TEST_OFFICIAL_NODE").expect("set an absolute Node executable"),
    );
    assert!(entry.is_absolute() && entry.is_file() && node.is_absolute() && node.is_file());
    keyring::set_default_credential_builder(Box::new(MemoryBuilder(Default::default())));
    let root = tempfile::tempdir().unwrap();
    let user = root.path().join("isolated-user");
    let upstream_home = root.path().join("official-home");
    let manager = root.path().join("manager");
    let codex = user.join(".codex");
    let xdg = user.join(".config");
    for path in [&user, &upstream_home, &manager, &codex, &xdg] {
        std::fs::create_dir_all(path).unwrap();
    }
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let model_port = listener.local_addr().unwrap().port();
    let generation = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = generation.clone();
    let service=axum::Router::new().route("/v1/models",axum::routing::get(||async {axum::Json(json!({"object":"list","data":[{"id":"fixture-base","object":"model","owned_by":"fixture"},{"id":"fixture-anthropic","object":"model","owned_by":"fixture"}]}))})).fallback(move |request:axum::extract::Request|{let count=count.clone();async move {if request.method()==axum::http::Method::POST{count.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}axum::http::StatusCode::NOT_FOUND}});
    let model_server = tokio::spawn(async move {
        axum::serve(listener, service).await.unwrap();
    });
    let port = free_port();
    let base = format!("http://127.0.0.1:{model_port}/v1");
    let config = json!({"port":port,"hostname":"127.0.0.1","defaultProvider":"fixture","providers":{"fixture":{"adapter":"openai-responses","authMode":"key","baseUrl":base,"apiKey":"fixture-local-key","models":["fixture-base","fixture-anthropic"],"liveModels":false,"allowPrivateNetwork":true,"newModelPolicy":"off","selectedModels":["fixture-base","fixture-anthropic"]}},"claudeCode":{"systemEnv":false,"enabled":false},"clientIntegrations":{"codex":true,"grok":false,"claude":false},"syncResumeHistory":false,"serviceHealthRecovery":{"enabled":false},"oauthOpenBrowser":false});
    std::fs::write(
        upstream_home.join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();
    let wrapper = root.path().join("ocx");
    let fail_sync = root.path().join("fail-sync");
    std::fs::write(&wrapper,format!("#!/bin/sh\nexport HOME={}\nexport CODEX_HOME={}\nexport XDG_CONFIG_HOME={}\nif [ \"$1\" = sync ] && [ -f {} ]; then exit 17; fi\nexec {} {} \"$@\"\n",quote(&user),quote(&codex),quote(&xdg),quote(&fail_sync),quote(&node),quote(&entry))).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut command = tokio::process::Command::new(&wrapper);
    command
        .args(["start", "--port", &port.to_string()])
        .env_clear()
        .env("OPENCODEX_HOME", &upstream_home)
        .env("PATH", node.parent().unwrap())
        .env("CI", "1")
        .current_dir(root.path())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .process_group(0);
    let mut child = command.spawn().unwrap();
    let _group = ProcessGroup(child.id().unwrap());
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let capture = tokio::spawn(async move {
        let mut stdout = stdout.take(1024 * 1024);
        let mut stderr = stderr.take(1024 * 1024);
        let mut out = vec![];
        let mut err = vec![];
        let _ = tokio::join!(stdout.read_to_end(&mut out), stderr.read_to_end(&mut err));
        (out, err)
    });
    let context=ProcessContext {runtime:opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(wrapper),working_directory:root.path().into(),opencodex_home:upstream_home.clone()};
    let deadline = std::time::Instant::now() + Duration::from_secs(45);
    let mut ready = false;
    while std::time::Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if upstream_home.join("runtime-port.json").exists()
            && model_config::management::Management::bind(&context)
                .await
                .is_ok()
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    if !ready {
        let _ = child.kill().await;
        let _ = child.wait().await;
        let (out, err) = capture.await.unwrap();
        panic!(
            "isolated official did not become ready: {} {}",
            String::from_utf8_lossy(&out),
            String::from_utf8_lossy(&err)
        );
    }
    let initialized = upstream::write(&context, upstream::WriteOperation::Sync)
        .await
        .unwrap();
    if !initialized.success {
        let output = tokio::process::Command::new(context.executable().unwrap())
            .args(["inspect", "catalog", "--json"])
            .env_clear()
            .env("OPENCODEX_HOME", &upstream_home)
            .env("PATH", node.parent().unwrap())
            .current_dir(root.path())
            .output()
            .await
            .unwrap();
        panic!(
            "isolated initial sync/catalog: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let snapshot = upstream::snapshot(&context).await.unwrap();
    assert!(
        snapshot.version.contains("2.84.0"),
        "record actual version before adapting this fixture"
    );
    let discovered = discovery::discover(discovery::DiscoveryRequest {
        protocol: "openai-responses".into(),
        url: base.clone(),
        api_key: Some("fixture-local-key".into()),
        credential_ref: None,
        allow_private_network: true,
    })
    .await
    .unwrap();
    assert_eq!(discovered.models.len(), 2);
    let node_id = uuid::Uuid::new_v4().to_string();
    let channel_id = uuid::Uuid::new_v4().to_string();
    let mut policy = Policy {
        pending_removals: vec![],
        recovery_id: None,
        default_channel_id: None,
        schema_version: SCHEMA_VERSION,
        template_refs: vec![],
        channels: vec![ChannelPolicy {
            id: channel_id,
            label: "本机协议分组fixture".into(),
            connection: ChannelConnection {
                auth_mode: None,
                source_read_id: None,
                imported_config: None,
                provider_id: "fixture".into(),
                protocol: "openai-responses".into(),
                nodes: vec![ServiceNode {
                    id: node_id.clone(),
                    name: "main".into(),
                    url: base,
                }],
                active_node_id: node_id,
                credential_ref: discovered.credential_ref,
                enabled: true,
                allow_private_network: true,
            },
            selection: ModelSelection::Whitelist(vec![
                "fixture-base".into(),
                "fixture-anthropic".into(),
            ]),
            model_order: vec!["fixture-anthropic".into(), "fixture-base".into()],
            discovered: discovered.models,
            template_refs: vec![],
            model_refs: BTreeMap::new(),
            defaults: ModelRules::default(),
            overrides: BTreeMap::from([(
                "fixture-anthropic".into(),
                ModelRules {
                    protocol: Field::Custom("anthropic".into()),
                    ..Default::default()
                },
            )]),
            family_overrides: BTreeMap::new(),
        }],
    };
    policy.channels[0].overrides.insert(
        "fixture-base".into(),
        ModelRules {
            reasoning_efforts: Field::Custom(vec!["low".into(), "high".into()]),
            default_effort: Field::Custom("low".into()),
            pinned_effort: Field::Custom("high".into()),
            effort_map: Field::Custom(BTreeMap::from([("high".into(), "__omit__".into())])),
            ..Default::default()
        },
    );
    for model in &mut policy.channels[0].discovered {
        model.label = if model.id == "fixture-base" {
            "基础型号"
        } else {
            "协议型号"
        }
        .into();
    }
    let store = model_config::ModelStore::new(&manager);
    let local = store.load().unwrap();
    let staged = store
        .stage_policy(
            &local.policy_digest,
            &local.workspace_digest,
            policy.clone(),
        )
        .unwrap();
    compiler::project(&staged.library, &policy).unwrap();
    let plan = runtime::prepare(
        &manager,
        &context,
        policy.clone(),
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert_eq!(plan.protocol_groups.len(), 1);
    assert_eq!(plan.route_changes.len(), 1);
    let saved = runtime::apply(&manager, &context, &plan.id).await.unwrap();
    if saved.phase != "committed" {
        let stored: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                manager
                    .join("manager-state/model-save-plans")
                    .join(format!("{}.json", plan.id)),
            )
            .unwrap(),
        )
        .unwrap();
        let actual = upstream::snapshot(&context).await.unwrap();
        eprintln!(
            "isolated-order-diagnostic {}",
            json!({"intended":stored["nextOrder"],"actual":actual.order,"phase":saved.phase,"error":saved.error})
        );
    }
    assert_eq!(
        saved.phase, "committed",
        "official save phase={} error={:?}",
        saved.phase, saved.error
    );
    assert_eq!(store.load().unwrap().policy, policy);
    let after = upstream::snapshot(&context).await.unwrap();
    assert_eq!(
        after.providers["providers"]["fixture"]["modelPinnedReasoningEfforts"]["fixture-base"],
        "high"
    );
    assert_eq!(
        after.providers["providers"]["fixture"]["modelReasoningEffortMap"]["fixture-base"]["high"],
        "__omit__"
    );
    assert_eq!(
        after.providers["providers"]["fixture"]["modelDefaultReasoningEfforts"]["fixture-base"],
        "low"
    );
    let order = after.order["pickerOrder"].as_array().unwrap();
    let mapped = plan.protocol_groups[0].split(" → ").last().unwrap();
    assert!(order
        .iter()
        .any(|slug| slug.as_str() == Some(&format!("{mapped}/fixture-anthropic"))));
    assert_eq!(
        after.providers["providers"]["fixture"]["modelDisplayNames"]["fixture-base"],
        "基础型号"
    );
    assert_eq!(
        after.providers["providers"][mapped]["modelDisplayNames"]["fixture-anthropic"],
        "协议型号"
    );
    let client = model_config::client_state::check(
        &manager,
        &context,
        model_config::client_state::Request {
            plan_id: plan.id.clone(),
            confirmation_check: None,
        },
    )
    .await
    .unwrap();
    assert!(client.runtime_matches);
    assert!(matches!(
        client.status,
        model_config::client_state::Status::PendingVerification
    ));
    assert!(!client.automatic_client_verification);
    let stable_group = plan.protocol_groups[0].clone();
    let mut hidden_policy = policy.clone();
    hidden_policy.channels[0].selection = ModelSelection::None;
    let current = store.load().unwrap();
    let staged = store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            hidden_policy.clone(),
        )
        .unwrap();
    let hidden_plan = runtime::prepare(
        &manager,
        &context,
        hidden_policy.clone(),
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert_eq!(hidden_plan.protocol_groups[0], stable_group);
    assert_eq!(hidden_plan.catalog_count, 0);
    let hidden = runtime::apply(&manager, &context, &hidden_plan.id)
        .await
        .unwrap();
    assert_eq!(hidden.phase, "committed", "hide result {:?}", hidden.error);
    let hidden_snapshot = upstream::snapshot(&context).await.unwrap();
    assert_eq!(
        hidden_snapshot.catalog["models"].as_array().unwrap().len(),
        0
    );
    let mut restored_policy = policy;
    restored_policy.channels[0]
        .discovered
        .iter_mut()
        .find(|m| m.id == "fixture-base")
        .unwrap()
        .label
        .clear();
    restored_policy.channels[0].selection = ModelSelection::Whitelist(vec!["fixture-base".into()]);
    let current = store.load().unwrap();
    let staged = store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            restored_policy.clone(),
        )
        .unwrap();
    let restore_plan = runtime::prepare(
        &manager,
        &context,
        restored_policy,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert_eq!(restore_plan.protocol_groups[0], stable_group);
    assert_eq!(restore_plan.catalog_count, 1);
    let restored = runtime::apply(&manager, &context, &restore_plan.id)
        .await
        .unwrap();
    assert_eq!(
        restored.phase, "committed",
        "restore result {:?}",
        restored.error
    );
    let restored_snapshot = upstream::snapshot(&context).await.unwrap();
    assert!(
        restored_snapshot.providers["providers"]["fixture"]["modelDisplayNames"]
            .get("fixture-base")
            .is_none()
    );
    // Register an independent default, then remove the former channel and owned partition.
    let current = store.load().unwrap();
    let mut next = current.policy.clone();
    let mut copy = next.channels[0].clone();
    copy.id = uuid::Uuid::new_v4().to_string();
    copy.label = "删除后保留fixture".into();
    copy.connection.provider_id = "fixture-keep".into();
    for node in &mut copy.connection.nodes {
        node.id = uuid::Uuid::new_v4().to_string();
    }
    copy.connection.active_node_id = copy.connection.nodes[0].id.clone();
    copy.overrides.clear();
    copy.discovered.retain(|m| m.id == "fixture-base");
    copy.model_order = vec!["fixture-base".into()];
    copy.template_refs.clear();
    copy.model_refs.clear();
    copy.family_overrides.clear();
    next.default_channel_id = Some(copy.id.clone());
    next.channels.push(copy.clone());
    let staged = store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            next.clone(),
        )
        .unwrap();
    let add = runtime::prepare(
        &manager,
        &context,
        next,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    let added = runtime::apply(&manager, &context, &add.id).await.unwrap();
    assert_eq!(added.phase, "committed", "{:?}", added.error);
    let management = model_config::management::Management::bind(&context)
        .await
        .unwrap();
    management
        .add_key_labeled("fixture", "fixture-second-local-key", Some("secondary"))
        .await
        .unwrap();
    management
        .patch_headers(
            "fixture",
            &BTreeMap::from([("X-Fixture-Auth".into(), "fixture-private-header".into())]),
        )
        .await
        .unwrap();
    let key_pool_before = management.key_state("fixture").await.unwrap();
    let historical_private: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    assert_eq!(key_pool_before.ids.len(), 2);
    let current = store.load().unwrap();
    let next = Policy {
        channels: vec![copy.clone()],
        default_channel_id: Some(copy.id),
        ..Default::default()
    };
    let staged = store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            next.clone(),
        )
        .unwrap();
    let removal = runtime::prepare(
        &manager,
        &context,
        next,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert!(removal.removed_providers.contains(&"fixture".to_string()));
    assert_eq!(removal.removed_providers.len(), 2);
    let removed = runtime::apply(&manager, &context, &removal.id)
        .await
        .unwrap();
    assert_eq!(removed.phase, "committed", "{:?}", removed.error);
    let actual = upstream::snapshot(&context).await.unwrap();
    assert!(actual.providers["providers"].get("fixture").is_none());
    assert!(actual.providers["providers"].get(mapped).is_none());
    assert_eq!(actual.providers["defaultProvider"], "fixture-keep");
    assert_eq!(actual.catalog["models"].as_array().unwrap().len(), 1);
    // Exercise the public successful-history → draft → normal save → readback path.
    let current = store.load().unwrap();
    let recovery = runtime::prepare_recovery(
        &manager,
        model_config::recovery::Request {
            source_id: removal.id.clone(),
            side: model_config::recovery::Side::Before,
            scope: model_config::recovery::Scope::Affected,
            restore_defaults: false,
            expected_library: current.library_digest,
            expected_policy: current.policy_digest,
            expected_workspace: current.workspace_digest,
        },
    )
    .unwrap();
    let staged = runtime::accept_recovery(&manager, &recovery.id).unwrap();
    let historical = runtime::prepare(
        &manager,
        &context,
        staged.workspace.unwrap().policy,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert_eq!(
        historical.history_source.as_deref(),
        Some(removal.id.as_str())
    );
    let historical_result = runtime::apply(&manager, &context, &historical.id)
        .await
        .unwrap();
    assert_eq!(
        historical_result.phase, "committed",
        "{:?}",
        historical_result.error
    );
    let actual = upstream::snapshot(&context).await.unwrap();
    let stored: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            manager
                .join("manager-state/model-save-plans")
                .join(format!("{}.json", removal.id)),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual.providers, stored["before"]["providers"]);
    assert_eq!(
        management.key_state("fixture").await.unwrap(),
        key_pool_before
    );
    for backup in stored["removedKeys"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r.get("backup"))
        .filter(|v| !v.is_null())
    {
        let backup: model_config::provider_auth::Backup =
            serde_json::from_value(backup.clone()).unwrap();
        model_config::provider_auth::verify(&manager, &context, &backup)
            .await
            .unwrap();
    }
    assert_eq!(
        runtime::apply(&manager, &context, &historical.id)
            .await
            .unwrap()
            .phase,
        "committed"
    );

    // Replace an existing authorization generation through the actual public
    // history -> draft -> normal Save path, and compensate an injected final-sync failure.
    management
        .add_key_labeled("fixture", "fixture-query-delete-key", Some("delete check"))
        .await
        .unwrap();
    let with_extra = management.key_state("fixture").await.unwrap();
    let disposable = with_extra
        .ids
        .difference(&key_pool_before.ids)
        .next()
        .unwrap()
        .clone();
    management.remove_key("fixture", &disposable).await.unwrap();
    assert_eq!(
        management.key_state("fixture").await.unwrap().ids,
        key_pool_before.ids
    );
    management
        .add_key_labeled("fixture", "fixture-third-local-key", Some("new generation"))
        .await
        .unwrap();
    for id in &key_pool_before.ids {
        management.remove_key("fixture", id).await.unwrap();
    }
    management
        .patch_headers(
            "fixture",
            &BTreeMap::from([("X-Fixture-Auth".into(), "fixture-third-header".into())]),
        )
        .await
        .unwrap();
    let third_state = management.key_state("fixture").await.unwrap();
    assert_eq!(third_state.ids.len(), 1);
    let third = model_config::provider_auth::capture(&manager, &context, "fixture", &third_state)
        .await
        .unwrap()
        .unwrap();
    let replacement = history_plan(&manager, &context, &removal.id).await;
    assert!(replacement
        .runtime_changes
        .iter()
        .any(|c| c.field == "授权选择/fixture"));
    std::fs::write(&fail_sync, b"owned failure injection").unwrap();
    let partial = runtime::apply(&manager, &context, &replacement.id)
        .await
        .unwrap();
    assert_ne!(partial.phase, "committed");
    assert!(partial.error.is_some());
    assert_eq!(
        management.key_state("fixture").await.unwrap(),
        key_pool_before
    );
    std::fs::remove_file(&fail_sync).unwrap();
    // Later private changes must reject compensation before any public write.
    management
        .patch_headers(
            "fixture",
            &BTreeMap::from([("X-Fixture-Auth".into(), "fixture-later-header".into())]),
        )
        .await
        .unwrap();
    assert_eq!(
        runtime::compensate(&manager, &context, &replacement.id)
            .await
            .unwrap_err()
            .code,
        "provider_auth_drift"
    );
    management
        .patch_headers(
            "fixture",
            &BTreeMap::from([("X-Fixture-Auth".into(), "fixture-private-header".into())]),
        )
        .await
        .unwrap();
    let compensated = runtime::compensate(&manager, &context, &replacement.id)
        .await
        .unwrap();
    assert_eq!(compensated.phase, "compensated", "{:?}", compensated.error);
    model_config::provider_auth::verify(&manager, &context, &third)
        .await
        .unwrap();
    assert_eq!(
        runtime::compensate(&manager, &context, &replacement.id)
            .await
            .unwrap()
            .phase,
        "compensated"
    );
    let replacement = history_plan(&manager, &context, &removal.id).await;
    let result = runtime::apply(&manager, &context, &replacement.id)
        .await
        .unwrap();
    if result.phase != "committed" {
        let actual: serde_json::Value =
            serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap())
                .unwrap();
        for provider in ["fixture", mapped] {
            let changed: Vec<_> = [
                "baseUrl",
                "adapter",
                "authMode",
                "apiKeyTransport",
                "apiKey",
                "apiKeyPool",
                "headers",
            ]
            .into_iter()
            .filter(|key| {
                actual["providers"][provider][key] != historical_private["providers"][provider][key]
            })
            .collect();
            eprintln!("private readback differs: provider={provider} fields={changed:?}");
        }
        for entry in
            std::fs::read_dir(manager.join("manager-state/provider-auth-transitions")).unwrap()
        {
            let file = entry.unwrap().path();
            if file.extension().and_then(|s| s.to_str()) == Some("json") {
                eprintln!(
                    "nonsecret transition: {}",
                    std::fs::read_to_string(file).unwrap()
                );
            }
        }
    }
    assert_eq!(result.phase, "committed", "{:?}", result.error);
    assert_eq!(
        management.key_state("fixture").await.unwrap(),
        key_pool_before
    );
    assert_eq!(
        runtime::apply(&manager, &context, &replacement.id)
            .await
            .unwrap()
            .phase,
        "committed"
    );
    // A successful existing->existing auth-only save must itself be restorable;
    // its public before/after rows are equal, so private scope cannot be omitted.
    let auth_only = history_plan(&manager, &context, &replacement.id).await;
    assert!(auth_only
        .runtime_changes
        .iter()
        .any(|c| c.field == "授权选择/fixture"));
    let result = runtime::apply(&manager, &context, &auth_only.id)
        .await
        .unwrap();
    assert_eq!(result.phase, "committed", "{:?}", result.error);
    model_config::provider_auth::verify(&manager, &context, &third)
        .await
        .unwrap();
    let client = model_config::client_state::status(&manager).unwrap();
    assert_eq!(client.plan_id.as_deref(), Some(auth_only.id.as_str()));
    let replacement = history_plan(&manager, &context, &removal.id).await;
    assert_eq!(
        runtime::apply(&manager, &context, &replacement.id)
            .await
            .unwrap()
            .phase,
        "committed"
    );

    // Simulate official Keychain migration with the same service/account contract
    // in the owned in-memory keyring; this is not a native Keychain availability test.
    let config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    let mut pool = config["providers"]["fixture"]["apiKeyPool"]
        .as_array()
        .unwrap()
        .clone();
    for slot in &mut pool {
        let id = slot["id"].as_str().unwrap().to_owned();
        let account = format!("fixture/{id}");
        keyring::Entry::new("opencodex.provider-api-key.v1", &account)
            .unwrap()
            .set_password(slot["key"].as_str().unwrap())
            .unwrap();
        slot["key"] = json!(format!("keychain:{account}"));
    }
    management.reload_provider("fixture").await.unwrap();
    let public = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    management
        .restore_authorization_fields(
            "fixture",
            &public["providers"]["fixture"],
            json!({"apiKeyPool":pool,"headers":config["providers"]["fixture"]["headers"]}),
        )
        .await
        .unwrap();
    management
        .activate_key("fixture", key_pool_before.active_id.as_ref().unwrap())
        .await
        .unwrap();
    assert_eq!(
        management.key_state("fixture").await.unwrap(),
        key_pool_before
    );
    let referenced =
        model_config::provider_auth::capture(&manager, &context, "fixture", &key_pool_before)
            .await
            .unwrap()
            .unwrap();
    let operation = uuid::Uuid::new_v4().to_string();
    model_config::provider_auth::change(
        &manager,
        &context,
        &operation,
        "fixture",
        Some(&referenced),
        Some(&third),
    )
    .await
    .unwrap();
    model_config::provider_auth::change(
        &manager,
        &context,
        &operation,
        "fixture",
        Some(&referenced),
        Some(&third),
    )
    .await
    .unwrap();
    model_config::provider_auth::rollback_change(
        &manager,
        &context,
        &operation,
        "fixture",
        Some(&referenced),
        Some(&third),
    )
    .await
    .unwrap();
    model_config::provider_auth::rollback_change(
        &manager,
        &context,
        &operation,
        "fixture",
        Some(&referenced),
        Some(&third),
    )
    .await
    .unwrap();
    model_config::provider_auth::verify(&manager, &context, &referenced)
        .await
        .unwrap();
    let restored_config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        restored_config["providers"]["fixture"]["apiKeyPool"],
        json!(pool)
    );
    println!("existing-auth-generation=true query-delete=true history-auth-only=true injected-failure-compensated=true later-private-drift-rejected=true keychain-reference-identities-preserved=true native-keychain-not-tested=true");

    // An original official-only channel participates without first being saved
    // as a Desktop policy. The source and receiving manager roots are independent.
    let original_name = "fixture-original";
    let public_before = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    let mut public_next = public_before.clone();
    let mut original = public_before["providers"]["fixture"].clone();
    original["alias"] = json!("fixture-original-alias");
    original["modelAliases"] = json!({"fixture-base":"friendly-base"});
    original["modelDisplayNames"] = json!({"fixture-base":"Portable base"});
    original["models"] = json!(["fixture-base"]);
    original["selectedModels"] = json!(["fixture-base"]);
    original["modelAdapters"] = json!({"fixture-base":"openai-chat"});
    original["apiKeyPoolStrategy"] = json!("fill-first");
    original["adapter"] = json!("anthropic");
    original["apiKeyTransport"] = json!("bearer");
    original["note"] = json!("original configuration overlay");
    original["liveModels"] = json!(false);
    public_next["providers"][original_name] = original;
    let baseline = root.path().join("original-provider-baseline.json");
    std::fs::write(&baseline, serde_json::to_vec(&public_before).unwrap()).unwrap();
    let original_write = upstream::write(
        &context,
        upstream::WriteOperation::Providers {
            baseline: &baseline,
            next: &public_next,
        },
    )
    .await
    .unwrap();
    if !original_write.success {
        use tokio::io::AsyncWriteExt;
        let mut diagnostic = tokio::process::Command::new(context.executable().unwrap());
        diagnostic
            .args([
                "provider",
                "apply",
                "--baseline",
                baseline.to_str().unwrap(),
                "--file",
                "-",
                "--yes",
                "--json",
            ])
            .env_clear()
            .env("OPENCODEX_HOME", &upstream_home)
            .env("PATH", node.parent().unwrap())
            .current_dir(root.path())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut process = diagnostic.spawn().unwrap();
        process
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&public_next).unwrap())
            .await
            .unwrap();
        let output = process.wait_with_output().await.unwrap();
        panic!(
            "owned public-only original registration diagnostic: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    management
        .add_key_labeled(original_name, "portable-key-one", Some("first portable"))
        .await
        .unwrap();
    management
        .add_key_labeled(original_name, "portable-key-two", Some("second portable"))
        .await
        .unwrap();
    management
        .patch_headers(
            original_name,
            &BTreeMap::from([("X-Portable-Auth".into(), "portable-private-header".into())]),
        )
        .await
        .unwrap();
    assert!(
        upstream::write(&context, upstream::WriteOperation::Sync)
            .await
            .unwrap()
            .success
    );
    let original_snapshot = upstream::snapshot(&context).await.unwrap();
    let original_keys = management.key_state(original_name).await.unwrap();
    let original_raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    let inventory = model_config::portable_channels::catalog(&manager, &original_snapshot).unwrap();
    let original_id = inventory
        .iter()
        .find(|c| c.connection.provider_id == original_name)
        .unwrap()
        .id
        .clone();
    assert!(!store
        .load()
        .unwrap()
        .policy
        .channels
        .iter()
        .any(|c| c.id == original_id));
    assert_eq!(
        model_config::portable_channels::catalog(&manager, &original_snapshot)
            .unwrap()
            .iter()
            .find(|c| c.connection.provider_id == original_name)
            .unwrap()
            .id,
        original_id
    );
    let export = model_config::transfer::prepare_file_scope(
        &manager,
        model_config::transfer::ExportScope {
            channel_models: Default::default(),
            include_policy: true,
            channel_ids: vec![original_id.clone()],
            template_refs: vec![],
            include_global: false,
        },
        Some("fixture-portable-password"),
        true,
        &opencodex_desktop_lib::modules::sync::portable_manager::Scope {
            upstream_root: Some(upstream_home.clone()),
            upstream_snapshot: Some(original_snapshot.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(export.package.format_version, 3);
    let bytes = model_config::transfer::file_bytes(&manager, &export.id).unwrap();
    assert!(!String::from_utf8_lossy(&bytes).contains("portable-key-one"));
    assert!(!serde_json::to_string(&export)
        .unwrap()
        .contains("portable-private-header"));
    let mut removed_original = original_snapshot.providers.clone();
    removed_original["providers"]
        .as_object_mut()
        .unwrap()
        .remove(original_name);
    std::fs::write(
        &baseline,
        serde_json::to_vec(&original_snapshot.providers).unwrap(),
    )
    .unwrap();
    assert!(
        upstream::write(
            &context,
            upstream::WriteOperation::Providers {
                baseline: &baseline,
                next: &removed_original
            }
        )
        .await
        .unwrap()
        .success
    );
    let recipient = root.path().join("receiving-manager");
    std::fs::create_dir_all(&recipient).unwrap();
    let incoming =
        model_config::file_objects::receive(&recipient, &bytes, Some("fixture-portable-password"))
            .unwrap();
    assert_eq!(incoming.available_keys, vec![original_id.clone()]);
    let adopted = model_config::file_objects::accept(
        &recipient,
        &incoming.id,
        true,
        Some("fixture-portable-password"),
    )
    .unwrap();
    let mut policy = adopted.workspace.unwrap().policy;
    let imported = policy
        .channels
        .iter_mut()
        .find(|c| c.id == original_id)
        .unwrap();
    assert!(!imported.connection.enabled && imported.connection.credential_ref.is_none());
    assert!(imported.connection.imported_config.is_some());
    imported.connection.allow_private_network = true; // explicit loopback permission for this owned fixture
    let receiving_provider = imported.connection.provider_id.clone();
    let receiving_store = model_config::store::ModelStore::new(&recipient);
    let current = receiving_store.load().unwrap();
    let staged = receiving_store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            policy.clone(),
        )
        .unwrap();
    let save = runtime::prepare(
        &recipient,
        &context,
        policy,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    let saved_import = runtime::apply(&recipient, &context, &save.id)
        .await
        .unwrap();
    assert_eq!(saved_import.phase, "committed", "{:?}", saved_import.error);
    let received_public = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    let mut expected = original_snapshot.providers["providers"][original_name].clone();
    expected["disabled"] = json!(true);
    assert_eq!(received_public["providers"][&receiving_provider], expected);
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        original_keys
    );
    let received_raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        received_raw["providers"][&receiving_provider]["apiKeyPool"],
        original_raw["providers"][original_name]["apiKeyPool"]
    );
    assert_eq!(
        received_raw["providers"][&receiving_provider]["headers"],
        original_raw["providers"][original_name]["headers"]
    );
    let current = receiving_store.load().unwrap();
    let mut enabled = current.policy.clone();
    enabled.channels[0].connection.enabled = true;
    let staged = receiving_store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            enabled.clone(),
        )
        .unwrap();
    let enable = runtime::prepare(
        &recipient,
        &context,
        enabled,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    let enabled = runtime::apply(&recipient, &context, &enable.id)
        .await
        .unwrap();
    assert_eq!(enabled.phase, "committed", "{:?}", enabled.error);
    assert!(
        upstream::snapshot(&context).await.unwrap().catalog["models"]
            .as_array()
            .unwrap()
            .len()
            > original_snapshot.catalog["models"]
                .as_array()
                .unwrap()
                .len()
                - 1
    );
    assert!(
        model_config::file_objects::accept(
            &recipient,
            &incoming.id,
            true,
            Some("fixture-portable-password")
        )
        .unwrap()
        .policy
        .channels[0]
            .connection
            .enabled
    );
    management
        .add_key_labeled(
            &receiving_provider,
            "recipient-later-key",
            Some("later local credential"),
        )
        .await
        .unwrap();
    management
        .patch_headers(
            &receiving_provider,
            &BTreeMap::from([("X-Portable-Auth".into(), "recipient-later-header".into())]),
        )
        .await
        .unwrap();
    let later_keys = management.key_state(&receiving_provider).await.unwrap();
    assert_eq!(later_keys.ids.len(), 3);
    let incoming_again =
        model_config::file_objects::receive(&recipient, &bytes, Some("fixture-portable-password"))
            .unwrap();
    let adopted_again = model_config::file_objects::accept(
        &recipient,
        &incoming_again.id,
        true,
        Some("fixture-portable-password"),
    )
    .unwrap();
    let policy_again = adopted_again.workspace.unwrap().policy;
    assert!(!policy_again.channels[0].connection.enabled);
    let current = receiving_store.load().unwrap();
    let replace = runtime::prepare(
        &recipient,
        &context,
        policy_again,
        current.library_digest,
        current.policy_digest,
        current.workspace_digest,
    )
    .await
    .unwrap();
    std::fs::write(&fail_sync, b"owned import failure injection").unwrap();
    let partial_import = runtime::apply(&recipient, &context, &replace.id)
        .await
        .unwrap();
    assert_ne!(partial_import.phase, "committed");
    assert!(partial_import.error.is_some());
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        original_keys
    );
    std::fs::remove_file(&fail_sync).unwrap();
    let compensated_import = runtime::compensate(&recipient, &context, &replace.id)
        .await
        .unwrap();
    assert_eq!(
        compensated_import.phase, "compensated",
        "{:?}",
        compensated_import.error
    );
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        later_keys
    );
    let current = receiving_store.load().unwrap();
    let retry = runtime::prepare(
        &recipient,
        &context,
        current.workspace.unwrap().policy,
        current.library_digest,
        current.policy_digest,
        current.workspace_digest,
    )
    .await
    .unwrap();
    let replaced = runtime::apply(&recipient, &context, &retry.id)
        .await
        .unwrap();
    assert_eq!(replaced.phase, "committed", "{:?}", replaced.error);
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        original_keys
    );
    let repeated = model_config::file_objects::accept(
        &recipient,
        &incoming_again.id,
        true,
        Some("fixture-portable-password"),
    )
    .unwrap();
    assert_eq!(
        repeated.policy.channels[0].connection.imported_config,
        receiving_store.load().unwrap().policy.channels[0]
            .connection
            .imported_config
    );
    println!("original-official-inventory=true protected-full-pool-transfer=true original-public-fields-exact=true imported-disabled=true separate-enable-with-alias-budget=true import-replay-preserves-later-save=true existing-import-replaces-explicit-private-generation=true imported-private-generation-compensated=true");

    let current = receiving_store.load().unwrap();
    let mut make_default = current.policy.clone();
    make_default.channels[0].connection.enabled = true;
    make_default.default_channel_id = Some(make_default.channels[0].id.clone());
    let staged = receiving_store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            make_default.clone(),
        )
        .unwrap();
    let default_plan = runtime::prepare(
        &recipient,
        &context,
        make_default,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    assert_eq!(
        runtime::apply(&recipient, &context, &default_plan.id)
            .await
            .unwrap()
            .phase,
        "committed"
    );
    // A successful managed node change records both exact service directions.
    // Restoring it must replace endpoint and matching auth together, including
    // compensation after the final sync fails.
    let current = receiving_store.load().unwrap();
    let mut other_direction = current.policy.clone();
    let node = &mut other_direction.channels[0].connection.nodes[0];
    let original_url = node.url.clone();
    let alternate_url = format!("{}/alternate", original_url.trim_end_matches('/'));
    node.url = alternate_url.clone();
    let staged = receiving_store
        .stage_policy(
            &current.policy_digest,
            &current.workspace_digest,
            other_direction.clone(),
        )
        .unwrap();
    let direction_save = runtime::prepare(
        &recipient,
        &context,
        other_direction,
        staged.library_digest,
        staged.policy_digest,
        staged.workspace_digest,
    )
    .await
    .unwrap();
    let changed = runtime::apply(&recipient, &context, &direction_save.id)
        .await
        .unwrap();
    assert_eq!(changed.phase, "committed", "{:?}", changed.error);
    let recovery_direction = history_plan(&recipient, &context, &direction_save.id).await;
    assert!(recovery_direction
        .runtime_changes
        .iter()
        .any(|fact| fact.field.contains("baseUrl")));
    std::fs::write(&fail_sync, b"owned direction failure injection").unwrap();
    let partial = runtime::apply(&recipient, &context, &recovery_direction.id)
        .await
        .unwrap();
    assert_ne!(partial.phase, "committed");
    assert!(partial.error.is_some());
    let actual = upstream::snapshot(&context).await.unwrap();
    assert_eq!(
        actual.providers["providers"][&receiving_provider]["baseUrl"],
        original_url
    );
    std::fs::remove_file(&fail_sync).unwrap();
    let compensated = runtime::compensate(&recipient, &context, &recovery_direction.id)
        .await
        .unwrap();
    assert_eq!(compensated.phase, "compensated", "{:?}", compensated.error);
    let actual = upstream::snapshot(&context).await.unwrap();
    assert_eq!(
        actual.providers["providers"][&receiving_provider]["baseUrl"],
        alternate_url
    );
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        original_keys
    );
    assert_eq!(
        runtime::compensate(&recipient, &context, &recovery_direction.id)
            .await
            .unwrap()
            .phase,
        "compensated"
    );
    let recovery_direction = history_plan(&recipient, &context, &direction_save.id).await;
    let restored_direction = runtime::apply(&recipient, &context, &recovery_direction.id)
        .await
        .unwrap();
    assert_eq!(
        restored_direction.phase, "committed",
        "{:?}",
        restored_direction.error
    );
    let actual = upstream::snapshot(&context).await.unwrap();
    assert_eq!(
        actual.providers["providers"][&receiving_provider]["baseUrl"],
        original_url
    );
    assert_eq!(
        management.key_state(&receiving_provider).await.unwrap(),
        original_keys
    );
    let current_public = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    let public_before_wire = current_public["providers"][&receiving_provider].clone();
    let keys_before_wire = management.key_state(&receiving_provider).await.unwrap();
    let auth_before_wire = model_config::provider_auth::capture(
        &recipient,
        &context,
        &receiving_provider,
        &keys_before_wire,
    )
    .await
    .unwrap()
    .unwrap();
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(upstream_home.join("config.json")).unwrap()).unwrap();
    let mut target_wire = public_before_wire.clone();
    target_wire["adapter"] = json!("openai-responses");
    target_wire["disabled"] = json!(true);
    target_wire["newModelPolicy"] = json!("off");
    target_wire
        .as_object_mut()
        .unwrap()
        .remove("apiKeyTransport");
    let authorization:model_config::provider_auth::PortableAuthorization=serde_json::from_value(json!({"activeId":keys_before_wire.active_id,"fields":{"apiKeyPool":raw["providers"][&receiving_provider]["apiKeyPool"],"headers":raw["providers"][&receiving_provider]["headers"]}})).unwrap();
    let mut wire_channel = receiving_store.load().unwrap().policy.channels[0].clone();
    wire_channel.connection.protocol = "openai-responses".into();
    let wire_reference = model_config::provider_import::stage(
        &recipient,
        &wire_channel,
        &target_wire,
        Some(&authorization),
    )
    .unwrap();
    let wire_material =
        model_config::provider_import::load(&recipient, &wire_channel, &wire_reference).unwrap();
    let auth_wire =
        model_config::provider_import::authorization(&recipient, &context, &wire_material)
            .unwrap()
            .unwrap();
    let wire_direction = model_config::provider_auth::Direction {
        from_public: public_before_wire.clone(),
        to_public: target_wire.clone(),
    };
    model_config::provider_auth::review_direction(
        &recipient,
        &context,
        &receiving_provider,
        Some(&auth_before_wire),
        Some(&auth_wire),
        &wire_direction,
    )
    .unwrap();
    let wire_operation = uuid::Uuid::new_v4().to_string();
    model_config::provider_auth::change_direction(
        &recipient,
        &context,
        &wire_operation,
        &receiving_provider,
        Some(&auth_before_wire),
        Some(&auth_wire),
        &wire_direction,
    )
    .await
    .unwrap();
    model_config::provider_auth::verify(&recipient, &context, &auth_wire)
        .await
        .unwrap();
    let wire_public = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    assert_eq!(wire_public["providers"][&receiving_provider], target_wire);
    let mut restore_wire = public_before_wire.clone();
    restore_wire["disabled"] = json!(true);
    model_config::provider_auth::rollback_direction(
        &recipient,
        &context,
        &wire_operation,
        &receiving_provider,
        Some(&auth_before_wire),
        Some(&auth_wire),
        &wire_direction,
        &restore_wire,
    )
    .await
    .unwrap();
    model_config::provider_auth::verify(&recipient, &context, &auth_before_wire)
        .await
        .unwrap();
    let baseline_public = upstream::read(&context, upstream::ReadOperation::ProviderSnapshot)
        .await
        .unwrap();
    std::fs::write(&baseline, serde_json::to_vec(&baseline_public).unwrap()).unwrap();
    assert!(
        upstream::write(
            &context,
            upstream::WriteOperation::Providers {
                baseline: &baseline,
                next: &current_public
            }
        )
        .await
        .unwrap()
        .success
    );
    println!("history-service-direction-restored=true paired-authorization-compensation=true direction-replay=true default-provider-direction=true adapter-and-key-transport-restored-together=true");

    assert_eq!(
        generation.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "discovery/save cannot generate text"
    );
    println!("official={} models={} catalog={}→{}→{}→{} phases={}/{}/{} protocol-groups={} stable-group=true new-default=true removal-including-owned-group=true history-to-normal-save=true multi-key-header-restored=true no-generation=true",snapshot.version,discovered_count(&after.models),plan.current_catalog_count,plan.catalog_count,hidden_plan.catalog_count,restore_plan.catalog_count,saved.phase,hidden.phase,restored.phase,plan.protocol_groups.len());
    let _ = child.kill().await;
    let _ = child.wait().await;
    capture.abort();
    model_server.abort();
}
fn discovered_count(value: &serde_json::Value) -> usize {
    value.as_array().map(Vec::len).unwrap_or_default()
}
