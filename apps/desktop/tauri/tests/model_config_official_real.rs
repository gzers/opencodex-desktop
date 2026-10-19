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
    std::fs::write(&wrapper,format!("#!/bin/sh\nexport HOME={}\nexport CODEX_HOME={}\nexport XDG_CONFIG_HOME={}\nexec {} {} \"$@\"\n",quote(&user),quote(&codex),quote(&xdg),quote(&node),quote(&entry))).unwrap();
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
        schema_version: SCHEMA_VERSION,
        template_refs: vec![],
        channels: vec![ChannelPolicy {
            id: channel_id,
            label: "本机协议分组fixture".into(),
            connection: ChannelConnection {
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
    assert_eq!(
        generation.load(std::sync::atomic::Ordering::SeqCst),
        0,
        "discovery/save cannot generate text"
    );
    println!("official={} models={} catalog={}→{}→{}→{} phases={}/{}/{} protocol-groups={} stable-group=true no-generation=true",snapshot.version,discovered_count(&after.models),plan.current_catalog_count,plan.catalog_count,hidden_plan.catalog_count,restore_plan.catalog_count,saved.phase,hidden.phase,restored.phase,plan.protocol_groups.len());
    let _ = child.kill().await;
    let _ = child.wait().await;
    capture.abort();
    model_server.abort();
}
fn discovered_count(value: &serde_json::Value) -> usize {
    value.as_array().map(Vec::len).unwrap_or_default()
}
