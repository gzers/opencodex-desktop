//! External smoke helper: links the current production library without changing it.
use opencodex_desktop_lib::{
    infrastructure::runtime_executable::FixedRuntimeExecutable,
    modules::{model_config::{cache_refresh, config_file, management::Management}, test_sandbox},
    state::ProcessContext,
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf, sync::{Arc, Mutex}};

#[path = "production-commit-simulation.rs"]
mod safe_owned_restart_commit;

fn pending(root: &std::path::Path) -> Option<Value> {
    let path = root.join("manager-state/provider-client-renames.json");
    path.is_file().then(|| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap())
}

async fn observed(context: &ProcessContext) -> Result<Value, Value> {
    let management = Management::bind(context).await.map_err(|e| json!(e))?;
    let settings = management.public_settings().await.map_err(|e| json!(e))?;
    let integration = management.codex_integration().await.map_err(|e| json!(e))?;
    let live = management.public_config().await.map_err(|e| json!(e))?;
    management.verify_binding().map_err(|e| json!(e))?;
    Ok(json!({"attested":true,"codexRuntime":settings["codexRuntime"],
        "codexIntegration":integration,"publicConfig":live}))
}

type Secrets = Arc<Mutex<BTreeMap<String, Vec<u8>>>>;
struct MemoryBuilder(Secrets);
struct MemorySecret { values: Secrets, account: String }
impl keyring::credential::CredentialBuilderApi for MemoryBuilder {
    fn build(&self, _: Option<&str>, service: &str, user: &str) -> keyring::Result<Box<keyring::credential::Credential>> {
        Ok(Box::new(MemorySecret { values: self.0.clone(), account: format!("{service}/{user}") }))
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
impl keyring::credential::CredentialApi for MemorySecret {
    fn set_secret(&self, value: &[u8]) -> keyring::Result<()> {
        self.values.lock().unwrap().insert(self.account.clone(), value.to_vec()); Ok(())
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.values.lock().unwrap().get(&self.account).cloned().ok_or(keyring::Error::NoEntry)
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        self.values.lock().unwrap().remove(&self.account).map(|_| ()).ok_or(keyring::Error::NoEntry)
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
fn main() {
    keyring::set_default_credential_builder(Box::new(MemoryBuilder(Default::default())));
    let container = PathBuf::from(std::env::var_os("OCX021_CONTAINER").expect("explicit owned container"));
    assert!(container.is_absolute() && container.is_dir());
    assert!(test_sandbox::enabled());
    let manager = test_sandbox::resolve_data_root(&container.join("never-use-daily")).unwrap();
    assert_eq!(manager, container.join("manager"));
    let boundary = test_sandbox::resolve_boundary(&manager).unwrap();
    let home = container.join("official");
    let work = container.join("work");
    for p in [&manager, &home, &work] { test_sandbox::check_boundary(boundary.as_deref(), p).unwrap(); }
    let context = ProcessContext {
        runtime: FixedRuntimeExecutable::resolved(container.join("ocx")),
        working_directory: work,
        opencodex_home: home.clone(),
    };
    assert!(context.executable().unwrap().is_file());
    let mode = std::env::args().nth(1).unwrap_or_else(|| "save".into());
    if mode == "observe-production-bindings" || mode == "production-prepare" || mode == "simulate-owned-restart-commit" {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        if mode == "simulate-owned-restart-commit" {
            let id = std::env::args().nth(2).expect("actual production prepare pending id");
            let result = runtime.block_on(safe_owned_restart_commit::run(&container, &manager, &home, &id));
            println!("{}", result);
            if result["simulationPassed"] != true { std::process::exit(25); }
            return;
        }
        if mode == "observe-production-bindings" {
            match runtime.block_on(observed(&context)) {
                Ok(bindings) => println!("{}", json!({"entry":"production Management read-only observation", "bindings":bindings,"pending":pending(&manager)})),
                Err(error) => { println!("{}", json!({"error":error})); std::process::exit(22); }
            }
            return;
        }
        let before = pending(&manager).expect("actual save must create durable pending intent");
        let result = runtime.block_on(cache_refresh::prepare_saved_rename_restart(&manager, &context));
        match result {
            Ok(id) => {
                let id = id.expect("pending intent must remain prepared until restart commit");
                let after = pending(&manager).expect("prepare cannot acknowledge pending");
                assert_eq!(before, after, "prepare mutated/cleared durable pending intent");
                assert_eq!(before["id"], id);
                println!("{}", json!({"entry":"cache_refresh::prepare_saved_rename_restart",
                    "success":true,"pendingId":id,"pendingRetained":true,"pending":after,
                    "globalRestartInvoked":false}));
            },
            Err(error) => {
                println!("{}", json!({"entry":"cache_refresh::prepare_saved_rename_restart",
                    "success":false,"error":error,"pending":pending(&manager),"globalRestartInvoked":false}));
                std::process::exit(23);
            }
        }
        return;
    }
    assert_eq!(mode, "save", "unsupported smoke helper mode");
    let before = config_file::snapshot(&manager, &home).expect("production snapshot");
    let mut policy = before.policy.clone();
    let channel = policy.channels.iter_mut().find(|c| c.connection.provider_id == "Krill-1").expect("old channel");
    let stable_id = channel.id.clone();
    let models_before = json!([channel.model_order, channel.discovered, channel.model_refs]);
    channel.label = "Krill".into();
    channel.connection.provider_id = "Krill".into();
    let request = config_file::SaveRequest {
        expected_file: before.policy_digest, policy, credentials: vec![],
        scope: Some(config_file::SaveScope { channel_id: stable_id.clone() }),
    };
    let result = config_file::save_with_context(&manager, &context, request);
    if let Err(error) = &result {
        println!("{}", json!({"entry":"config_file::save_with_context","success":false,"error":error}));
        std::process::exit(20);
    }
    let result = result.unwrap();
    assert!(result.changed);
    let after = config_file::snapshot(&manager, &home).unwrap();
    let channel = after.policy.channels.iter().find(|c| c.id == stable_id).unwrap();
    assert_eq!(channel.label, "Krill");
    assert_eq!(channel.connection.provider_id, "Krill");
    assert_eq!(json!([channel.model_order, channel.discovered, channel.model_refs]), models_before);
    let raw: Value = serde_json::from_slice(&std::fs::read(home.join("config.json")).unwrap()).unwrap();
    assert!(raw["providers"].get("Krill-1").is_none());
    assert_eq!(raw["defaultProvider"], "Krill");
    assert_eq!(raw["modelPickerOrder"], json!(["Krill/owned-routing-model", "Krill/owned-second-model"]));
    assert_eq!(raw["disabledModels"], json!(["Krill/owned-hidden-model"]));
    println!("{}", json!({
        "entry":"config_file::save_with_context", "success":true, "changed":result.changed,
        "stableChannelId":stable_id, "stableChannelIdPreserved":true, "modelIdentityPreserved":true,
        "providerBefore":"Krill-1", "providerAfter":channel.connection.provider_id,
        "sandboxRoot":manager, "sandboxBoundary":boundary,
        "processContext":{"executable":context.executable().unwrap(),"workingDirectory":context.working_directory,"opencodexHome":context.opencodex_home},
        "credentialBackend":"process-local memory; synthetic inline provider key",
        "clientRenamePending":pending(&manager)
    }));
}

// Compile the existing production guard test file unchanged against the actual
// linked library. The generated module declaration contains its absolute path.
// All restart HTTP responses in that file come from its attested local fixture;
// it never calls the official process-enumerating/global restart endpoint.
#[cfg(test)]
pub use opencodex_desktop_lib::{infrastructure, modules};
#[cfg(test)]
mod production_true_route_guards {
    use opencodex_desktop_lib::{
        modules::model_config::{
            cache_refresh::{sync_cache, CacheRefresh, RESTART_WARNING},
            management::Management,
        },
        state::ProcessContext,
    };
    use std::path::Path;
    include!("production-cache-refresh-guards.rs");
}
